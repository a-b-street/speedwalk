use std::collections::HashMap;

use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{
    BoundingRect, Closest, ClosestPoint, Coord, Distance, Euclidean, Line, LineString, Point,
};
use osm_reader::WayID;
use rstar::{AABB, RTree, primitives::GeomWithData};
use utils::Tags;

use crate::{Kind, Speedwalk, edits::CreateNewGeometry};

impl Speedwalk {
    pub fn connect_all_crossings(&self, include_crossing_no: bool) -> CreateNewGeometry {
        info!("Finding crossings to connect");
        let mut crossings = Vec::new();
        for (id, node) in &self.derived_nodes {
            // When do we generate a crossing way from a node? Have tried a few heuristics here:
            //
            // - when it's not part of a crossing way already (but then nodes on driveways are
            //   falsely included)
            // - when the road it's on has newly generated sidewalks (but then crossing nodes on
            //   roads with existing sidewalks but no crossing way are skipped)
            //
            // Simpler:
            //
            // - if the node is only attached to one way (in the middle), it needs a crossing
            // - if the node is attached to two ways AND those ways are nearly
            //   parallel/anti-parallel, then it needs a crossing
            if node.is_crossing() || (include_crossing_no && node.is_explicit_crossing_no()) {
                let ways = node
                    .way_ids
                    .iter()
                    .map(|w| &self.derived_ways[w])
                    .collect::<Vec<_>>();
                if ways.len() == 1 && ways[0].kind.is_road() {
                    crossings.push(*id);
                } else if ways.len() == 2
                    && ways[0].kind.is_road()
                    && ways[1].kind.is_road()
                    && nearly_parallel(&ways[0].linestring, &ways[1].linestring, 10.0)
                {
                    crossings.push(*id);
                }
            }
        }

        info!(
            "Building rtree for up to {} existing sidewalks",
            self.derived_ways.len()
        );
        let closest_sidewalk = RTree::bulk_load(
            self.derived_ways
                .iter()
                .filter(|(_, way)| way.kind == Kind::Sidewalk || way.is_walkable_other())
                .map(|(id, way)| GeomWithData::new(way.linestring.clone(), *id))
                .collect(),
        );

        info!("Generating {} crossings", crossings.len());
        let mut new_crossings = Vec::new();
        let mut insert_new_nodes = HashMap::new();
        for crossing_node_id in crossings {
            let crossing_node = &self.derived_nodes[&crossing_node_id];
            let crossing_pt = crossing_node.pt;

            // Make a perpendicular line at the node
            let road_way_id = crossing_node.way_ids[0];
            let road_linestring = &self.derived_ways[&road_way_id].linestring;
            let angle = angle_of_pt_on_line(road_linestring, crossing_pt);

            let Some((sidewalk1, endpt1)) =
                find_sidewalk_hit(&closest_sidewalk, crossing_pt, angle + 90.0)
            else {
                continue;
            };

            let Some((sidewalk2, endpt2)) =
                find_sidewalk_hit(&closest_sidewalk, crossing_pt, angle - 90.0)
            else {
                continue;
            };

            // If both sides snapped to the same place, skip it. The same sidewalk way could
            // stretch very far to both sides, so check distance of the snapped points instead.
            if Euclidean.distance(endpt1, endpt2) < 1.0 {
                continue;
            }

            let mut new_tags = Tags::empty();
            new_tags.insert("highway", "footway");
            new_tags.insert("footway", "crossing");
            // Store OSM reference: use crossing node ID (primary) and road way ID (always available as fallback)
            new_tags.insert("tmp:osm_node_id", format!("node/{}", crossing_node_id.0));
            new_tags.insert("tmp:osm_way_id", format!("way/{}", road_way_id.0));
            // Copy one tag from the crossing node to the new crossing way
            if let Some(value) = crossing_node.tags.get("crossing") {
                new_tags.insert("crossing", value);
            }

            new_crossings.push((LineString::new(vec![endpt1, crossing_pt, endpt2]), new_tags));

            insert_new_nodes
                .entry(sidewalk1)
                .or_insert_with(Vec::new)
                .push((endpt1, Tags::empty()));
            insert_new_nodes
                .entry(sidewalk2)
                .or_insert_with(Vec::new)
                .push((endpt2, Tags::empty()));
        }
        info!("Successfully made {} crossings", new_crossings.len());

        CreateNewGeometry {
            new_ways: new_crossings,
            new_kind: Kind::Crossing,
            insert_new_nodes,
            modify_existing_way_tags: HashMap::new(),
        }
    }
}

fn find_sidewalk_hit(
    closest_sidewalk: &RTree<GeomWithData<LineString, WayID>>,
    crossing_pt: Coord,
    angle: f64,
) -> Option<(WayID, Coord)> {
    // First try to project a perpendicular line from the crossing out 10m (far away), and find the
    // first sidewalk we hit.
    let line1 = Line::new(crossing_pt, project_away(crossing_pt, angle, 10.0));

    // TODO Still cursed
    //let bbox = aabb(&line1);
    let bbox = line1.bounding_rect();
    let aabb = AABB::from_corners(
        Point::new(bbox.min().x, bbox.min().y),
        Point::new(bbox.max().x, bbox.max().y),
    );

    let mut candidates = Vec::new();
    for obj in closest_sidewalk.locate_in_envelope_intersecting(&aabb) {
        for line2 in obj.geom().lines() {
            if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                line_intersection(line1, line2)
            {
                candidates.push((obj.data, intersection));
            }
        }
    }
    // There could be multiple hits. Pick the one closest to the specified crossing_pt
    if let Some(pair) = candidates.into_iter().min_by_key(|(_, end_pt)| {
        to_cm(Euclidean.distance(Point::from(crossing_pt), Point::from(*end_pt)))
    }) {
        return Some(pair);
    }

    // If the perpendicular line didn't hit anything, try a second strategy. Project a point just a
    // little bit (5m) to one side of the crossing, then find the closest point on any sidewalk to
    // that.
    //
    // (It's tempting to just use this strategy always, instead of first trying perpendicular
    // lines. But when two sidewalks meet at a corner, sometimes we incorrectly pick one of them
    // based on how much we project away from the crossing_pt.)
    let one_side_pt = project_away(crossing_pt, angle, 5.0);
    let obj = closest_sidewalk.nearest_neighbor(&Point::from(one_side_pt))?;
    // Then find the straight line to the crossing_pt using that matching sidewalk. Don't find the
    // closest point to one_side_pt, because that'll make a slightly angled crossing.
    match obj.geom().closest_point(&Point::from(crossing_pt)) {
        Closest::Intersection(pt) | Closest::SinglePoint(pt) => Some((obj.data, pt.into())),
        Closest::Indeterminate => None,
    }
}

// TODO Use new geo euclidean destination
fn project_away(pt: Coord, angle_degrees: f64, distance: f64) -> Coord {
    let (sin, cos) = angle_degrees.to_radians().sin_cos();
    Coord {
        x: pt.x + distance * cos,
        y: pt.y + distance * sin,
    }
}

// TODO Move more of these to utils
fn angle_of_pt_on_line(linestring: &LineString, pt: Coord) -> f64 {
    let line = linestring
        .lines()
        .min_by_key(|line| (Euclidean.distance(line, pt) * 10e9) as usize)
        .unwrap();
    angle_of_line(line)
}

fn angle_of_line(line: Line) -> f64 {
    line.dy().atan2(line.dx()).to_degrees()
}

fn average_angle(linestring: &LineString) -> f64 {
    let angles: Vec<f64> = linestring.lines().map(angle_of_line).collect();
    angles.iter().sum::<f64>() / (angles.len() as f64)
}

/// Degrees for input/output. Returns [-180, 180]. See  //
/// https://math.stackexchange.com/questions/110080/shortest-way-to-achieve-target-angle
pub fn shortest_rotation(angle1: f64, angle2: f64) -> f64 {
    ((angle1 - angle2 + 540.0) % 360.0) - 180.0
}

fn nearly_parallel(ls1: &LineString, ls2: &LineString, epsilon_degrees: f64) -> bool {
    shortest_rotation(average_angle(ls1), average_angle(ls2)).abs() < epsilon_degrees
}

fn to_cm(x: f64) -> usize {
    (x * 100.0).round() as usize
}
