use std::collections::HashMap;

use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{BoundingRect, Coord, Distance, Euclidean, Line, LineString, Point};
use osm_reader::WayID;
use rstar::{AABB, RTree, primitives::GeomWithData};
use utils::Tags;

use crate::{Kind, Speedwalk, edits::CreateNewGeometry};

impl Speedwalk {
    pub fn connect_all_crossings(&self) -> CreateNewGeometry {
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
            if node.is_crossing() {
                if node.way_ids.len() == 1 {
                    crossings.push(*id);
                } else if node.way_ids.len() == 2
                    && nearly_parallel(
                        &self.derived_ways[&node.way_ids[0]].linestring,
                        &self.derived_ways[&node.way_ids[1]].linestring,
                        10.0,
                    )
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
                .filter(|(_, way)| way.kind == Kind::Sidewalk)
                .map(|(id, way)| GeomWithData::new(way.linestring.clone(), *id))
                .collect(),
        );

        info!("Generating {} crossings", crossings.len());
        let mut new_crossings = Vec::new();
        let mut modify_existing = HashMap::new();
        for crossing_node in crossings {
            let project_away_meters = 10.0;

            let crossing_node = &self.derived_nodes[&crossing_node];
            let crossing_pt = crossing_node.pt;

            // Make a perpendicular line at the node
            let severance_linestring = &self.derived_ways[&crossing_node.way_ids[0]].linestring;
            let angle = angle_of_pt_on_line(severance_linestring, crossing_pt);

            let test_line1 = Line::new(
                crossing_pt,
                project_away(crossing_pt, angle + 90.0, project_away_meters),
            );
            let Some((sidewalk1, endpt1, insert_idx1)) =
                find_sidewalk_hit(&closest_sidewalk, test_line1)
            else {
                continue;
            };

            let test_line2 = Line::new(
                crossing_pt,
                project_away(crossing_pt, angle - 90.0, project_away_meters),
            );
            let Some((sidewalk2, endpt2, insert_idx2)) =
                find_sidewalk_hit(&closest_sidewalk, test_line2)
            else {
                continue;
            };

            let mut new_tags = Tags::empty();
            new_tags.insert("highway", "footway");
            new_tags.insert("footway", "crossing");
            // Copy one tag from the crossing node to the new crossing way
            if let Some(value) = crossing_node.tags.get("crossing") {
                new_tags.insert("crossing", value);
            }

            new_crossings.push((LineString::new(vec![endpt1, crossing_pt, endpt2]), new_tags));

            modify_existing
                .entry(sidewalk1)
                .or_insert_with(Vec::new)
                .push((endpt1, insert_idx1));
            modify_existing
                .entry(sidewalk2)
                .or_insert_with(Vec::new)
                .push((endpt2, insert_idx2));
        }

        CreateNewGeometry {
            new_objects: new_crossings,
            new_kind: Kind::Crossing,
            modify_existing,
        }
    }

    // Find the one road this crossing should be on
    pub fn add_one_crossing(&self, pt: Point) -> Option<(WayID, usize)> {
        info!(
            "Building rtree for up to {} existing sidewalks",
            self.derived_ways.len()
        );
        let closest_road = RTree::bulk_load(
            self.derived_ways
                .iter()
                // TODO and not Crossing or Other?
                .filter(|(_, way)| way.kind != Kind::Sidewalk)
                .map(|(id, way)| GeomWithData::new(way.linestring.clone(), *id))
                .collect(),
        );

        let obj = closest_road.nearest_neighbor(&pt)?;

        let (idx, _) = obj
            .geom()
            .lines()
            .enumerate()
            .min_by_key(|(_, line)| (Euclidean.distance(line, &pt) * 10e6) as usize)?;
        Some((obj.data, idx + 1))
    }
}

fn find_sidewalk_hit(
    closest_sidewalk: &RTree<GeomWithData<LineString, WayID>>,
    line1: Line,
) -> Option<(WayID, Coord, usize)> {
    // TODO Still cursed
    //let bbox = aabb(&line1);
    let bbox = line1.bounding_rect();
    let aabb = AABB::from_corners(
        Point::new(bbox.min().x, bbox.min().y),
        Point::new(bbox.max().x, bbox.max().y),
    );

    for obj in closest_sidewalk.locate_in_envelope_intersecting(&aabb) {
        for (idx, line2) in obj.geom().lines().enumerate() {
            if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                line_intersection(line1, line2)
            {
                return Some((obj.data, intersection, idx + 1));
            }
        }
    }
    None
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
fn shortest_rotation(angle1: f64, angle2: f64) -> f64 {
    ((angle1 - angle2 + 540.0) % 360.0) - 180.0
}

fn nearly_parallel(ls1: &LineString, ls2: &LineString, epsilon_degrees: f64) -> bool {
    shortest_rotation(average_angle(ls1), average_angle(ls2)).abs() < epsilon_degrees
}
