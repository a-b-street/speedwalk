use std::collections::HashMap;

use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{BoundingRect, Coord, Distance, Euclidean, Line, LineString, Point};
use osm_reader::WayID;
use rstar::{AABB, RTree, primitives::GeomWithData};
use utils::Tags;

use crate::{Kind, Speedwalk, edits::CreateNewGeometry};

impl Speedwalk {
    pub fn connect_all_crossings(&self, include_crossing_no: bool) -> CreateNewGeometry {
        info!("Finding crossings to connect");
        let mut crossings = Vec::new();
        let mut half_crossings = Vec::new();
        for (id, node) in &self.derived_nodes {
            if !(node.is_crossing() || (include_crossing_no && node.is_explicit_crossing_no())) {
                continue;
            }
            let ways = node
                .way_ids
                .iter()
                .map(|w| &self.derived_ways[w])
                .collect::<Vec<_>>();
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
            if ways.len() == 1 && ways[0].kind.is_road() {
                crossings.push(*id);
                continue;
            } else if ways.len() == 2
                && ways[0].kind.is_road()
                && ways[1].kind.is_road()
                && nearly_parallel(&ways[0].linestring, &ways[1].linestring, 10.0)
            {
                crossings.push(*id);
                continue;
            }

            // There are also cases where the crossing node is already connected to a sidewalk or
            // footway on one side, and we only need to generate a new way on the other side.
            let mut roads = Vec::new();
            let mut crossings = Vec::new();
            let mut other = Vec::new();
            for way in ways {
                if way.kind.is_road() {
                    roads.push(way);
                } else if way.kind == Kind::Crossing {
                    crossings.push(way);
                } else if way.kind == Kind::Other {
                    other.push(way);
                }
            }
            // TODO This may be too restrictive, but it's a start for cases like
            // https://www.openstreetmap.org/node/12270311650
            if crossings.is_empty() && roads.len() >= 1 && other.len() == 1 {
                half_crossings.push((*id, roads[0], other[0]));
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
            let project_away_meters = 10.0;

            let crossing_node = &self.derived_nodes[&crossing_node_id];
            let crossing_pt = crossing_node.pt;

            // Make a perpendicular line at the node
            let road_way_id = crossing_node.way_ids[0];
            let road_linestring = &self.derived_ways[&road_way_id].linestring;
            let angle = angle_of_pt_on_line(road_linestring, crossing_pt);

            let test_line1 = Line::new(
                crossing_pt,
                project_away(crossing_pt, angle + 90.0, project_away_meters),
            );
            let Some((sidewalk1, endpt1)) = find_sidewalk_hit(&closest_sidewalk, test_line1) else {
                continue;
            };

            let test_line2 = Line::new(
                crossing_pt,
                project_away(crossing_pt, angle - 90.0, project_away_meters),
            );
            let Some((sidewalk2, endpt2)) = find_sidewalk_hit(&closest_sidewalk, test_line2) else {
                continue;
            };

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

        info!("Generating {} half-crossings", half_crossings.len());
        for (crossing_node_id, road, existing_footway) in half_crossings {
            let project_away_meters = 10.0;

            let crossing_node = &self.derived_nodes[&crossing_node_id];
            let crossing_pt = crossing_node.pt;

            // Make two perpendicular lines at the node
            // TODO It might be more straightforward to figure out the incoming angle of
            // existing_footway and continue in that direction
            let angle = angle_of_pt_on_line(&road.linestring, crossing_pt);
            let mut candidates = Vec::new();
            for offset in [-90.0, 90.0] {
                let test_line = Line::new(
                    crossing_pt,
                    project_away(crossing_pt, angle + offset, project_away_meters),
                );
                if let Some((sidewalk, endpt)) = find_sidewalk_hit(&closest_sidewalk, test_line) {
                    candidates.push((sidewalk, endpt));
                }
            }

            // Which one is the missing side? Check the distance between this candidate endpoint
            // and the existing footway
            if let Some((sidewalk, endpt)) = candidates.into_iter().max_by_key(|(_, endpt)| {
                to_cm(Euclidean.distance(&existing_footway.linestring, &Point::from(*endpt)))
            }) {
                let mut new_tags = Tags::empty();
                new_tags.insert("highway", "footway");
                new_tags.insert("footway", "crossing");
                // Store OSM reference: use crossing node ID (primary) and road way ID (always available as fallback)
                new_tags.insert("tmp:osm_node_id", format!("node/{}", crossing_node_id.0));
                // TODO Plumb road id
                //new_tags.insert("tmp:osm_way_id", format!("way/{}", road_way_id.0));
                // Copy one tag from the crossing node to the new crossing way
                if let Some(value) = crossing_node.tags.get("crossing") {
                    new_tags.insert("crossing", value);
                }

                new_crossings.push((LineString::new(vec![endpt, crossing_pt]), new_tags));

                insert_new_nodes
                    .entry(sidewalk)
                    .or_insert_with(Vec::new)
                    .push((endpt, Tags::empty()));
            }
        }

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
    line1: Line,
) -> Option<(WayID, Coord)> {
    // TODO Still cursed
    //let bbox = aabb(&line1);
    let bbox = line1.bounding_rect();
    let aabb = AABB::from_corners(
        Point::new(bbox.min().x, bbox.min().y),
        Point::new(bbox.max().x, bbox.max().y),
    );

    for obj in closest_sidewalk.locate_in_envelope_intersecting(&aabb) {
        for line2 in obj.geom().lines() {
            if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                line_intersection(line1, line2)
            {
                return Some((obj.data, intersection));
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
pub fn shortest_rotation(angle1: f64, angle2: f64) -> f64 {
    ((angle1 - angle2 + 540.0) % 360.0) - 180.0
}

fn nearly_parallel(ls1: &LineString, ls2: &LineString, epsilon_degrees: f64) -> bool {
    shortest_rotation(average_angle(ls1), average_angle(ls2)).abs() < epsilon_degrees
}

fn to_cm(x: f64) -> usize {
    (x * 100.0).round() as usize
}
