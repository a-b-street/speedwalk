use std::collections::BTreeSet;

use anyhow::Result;
use geo::{Distance, Euclidean, InterpolatePoint, LineString, Point};
use geojson::GeoJson;
use osm_reader::NodeID;
use serde::Deserialize;

use crate::{
    Speedwalk, Way,
    crossings::shortest_rotation,
    graph::{EdgeID, Graph, IntersectionID},
};

#[derive(Deserialize)]
pub struct Options {
    only_major_roads: bool,
    ignore_utility_roads: bool,
    ignore_cycleways: bool,
    max_distance: f64,
}

impl Speedwalk {
    pub fn audit_crossings(&self, options: Options) -> Result<String> {
        let mut features = Vec::new();

        let graph = Graph::new(self);

        for junction in self.find_junctions(options, &graph) {
            let mut f = self
                .mercator
                .to_wgs84_gj(&graph.intersections[&junction.i].point);

            let mut arms = Vec::new();
            for (_, ls) in junction.arms {
                arms.push(self.mercator.to_wgs84_gj(&ls));
            }

            let mut crossings = Vec::new();
            for n in &junction.crossings {
                crossings.push(
                    self.mercator
                        .to_wgs84_gj(&Point::from(self.derived_nodes[n].pt)),
                );
            }

            let mut explicit_non_crossings = Vec::new();
            for n in &junction.explicit_non_crossings {
                explicit_non_crossings.push(
                    self.mercator
                        .to_wgs84_gj(&Point::from(self.derived_nodes[n].pt)),
                );
            }

            f.set_property(
                "complete",
                crossings.len() + explicit_non_crossings.len()
                    >= arms.len() - junction.number_dual_carriageway_splits,
            );
            f.set_property("arms", GeoJson::from(arms));
            f.set_property(
                "number_dual_carriageway_splits",
                junction.number_dual_carriageway_splits,
            );
            f.set_property("crossings", GeoJson::from(crossings));
            f.set_property(
                "explicit_non_crossings",
                GeoJson::from(explicit_non_crossings),
            );

            features.push(f);
        }

        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }

    /// Find all junctions
    fn find_junctions(&self, options: Options, graph: &Graph) -> Vec<Junction> {
        let mut junctions = Vec::new();
        for (i, intersection) in &graph.intersections {
            if self.derived_nodes[&intersection.osm_node]
                .tags
                .is("highway", "crossing")
            {
                continue;
            }

            let mut any_severances = false;
            let mut any_roads = false;
            let mut arms = Vec::new();
            let mut crossings = BTreeSet::new();
            let mut explicit_non_crossings = BTreeSet::new();
            for e in &intersection.edges {
                let edge = &graph.edges[e];
                let way = &self.derived_ways[&edge.osm_way];
                if way.is_severance() {
                    any_severances = true;
                }
                if options.ignore_utility_roads
                    && way.tags.is_any("highway", vec!["service", "track"])
                {
                    continue;
                }
                if options.ignore_cycleways && way.tags.is("highway", "cycleway") {
                    continue;
                }
                if way.kind.is_road() {
                    any_roads = true;
                }

                // The edge belongs to an OSM way, and is usually just a subset of it. Crossings
                // aren't always located directly on this first edge, so search a bit away. If the
                // OSM way is split before the crossing (uncommon), then we'll miss it.
                let (arm_ls, nodes_reached) = self.slice_way(
                    way,
                    intersection.osm_node,
                    edge.src == *i,
                    options.max_distance,
                );
                arms.push((*e, arm_ls));

                // Look for the first crossing (or crossing=no) along this arm
                for n in nodes_reached {
                    let node = &self.derived_nodes[&n];
                    if node.is_explicit_crossing_no() {
                        explicit_non_crossings.insert(n);
                        break;
                    }
                    if node.is_crossing() {
                        crossings.insert(n);
                        break;
                    }
                }
            }

            let number_dual_carriageway_splits =
                self.count_dual_carriageway_splits(graph, *i, &arms);

            if any_roads
                && (any_severances || !options.only_major_roads)
                && (arms.len() - number_dual_carriageway_splits) > 2
            {
                junctions.push(Junction {
                    i: *i,
                    arms,
                    number_dual_carriageway_splits,
                    crossings,
                    explicit_non_crossings,
                });
            }
        }
        junctions
    }

    fn count_dual_carriageway_splits(
        &self,
        graph: &Graph,
        i: IntersectionID,
        arms: &Vec<(EdgeID, LineString)>,
    ) -> usize {
        // For each one-way road, track its (name, whether it points at the intersection (true)
        // or away from it (false), angle)
        let mut oneway_roads: Vec<(String, bool, f64)> = Vec::new();
        let mut num_splits = 0;
        for (e, _) in arms {
            let edge = &graph.edges[e];
            let way = &self.derived_ways[&edge.osm_way];
            if way.kind.is_road()
                && way.tags.is("oneway", "yes")
                && let Some(name) = way.tags.get("name")
            {
                let dir = edge.dst == i;
                let angle = angle_ls_directional(&edge.linestring);

                if oneway_roads
                    .iter()
                    .any(|(other_name, other_dir, other_angle)| {
                        name == other_name
                            && dir == !other_dir
                            && shortest_rotation(angle, *other_angle).abs() > 45.0
                    })
                {
                    num_splits += 1;
                } else {
                    oneway_roads.push((name.to_string(), dir, angle));
                }
            }
        }
        num_splits
    }

    // Starting from a node on a way, search backwards or forwards up to max_distance, returning
    // the sliced LineString and all of the nodes reached (inclusive).
    fn slice_way(
        &self,
        way: &Way,
        start: NodeID,
        forwards: bool,
        max_distance: f64,
    ) -> (LineString, Vec<NodeID>) {
        let mut pts = Vec::new();
        let mut nodes_reached = Vec::new();
        let mut length_so_far = 0.0;

        let node_iter: Box<dyn Iterator<Item = &NodeID>> = if forwards {
            Box::new(way.node_ids.iter())
        } else {
            Box::new(way.node_ids.iter().rev())
        };

        for n in node_iter {
            let pt = Point::from(self.derived_nodes[n].pt);
            if nodes_reached.is_empty() {
                // We haven't found the start node yet
                if *n == start {
                    nodes_reached.push(*n);
                    pts.push(pt);
                }
            } else {
                let length_this_step = Euclidean.distance(*pts.last().unwrap(), pt);
                if length_so_far + length_this_step <= max_distance {
                    nodes_reached.push(*n);
                    pts.push(pt);
                    length_so_far += length_this_step;
                } else {
                    // Don't add this node -- it's too far away
                    // Stop somewhere between these two nodes
                    pts.push(Euclidean.point_at_distance_between(
                        *pts.last().unwrap(),
                        pt,
                        max_distance - length_so_far,
                    ));
                    break;
                }
            }
        }

        (
            LineString::new(pts.into_iter().map(|pt| pt.into()).collect()),
            nodes_reached,
        )
    }
}

struct Junction {
    i: IntersectionID,
    // The LineString is up to options.max_distance along the edge's way
    arms: Vec<(EdgeID, LineString)>,
    number_dual_carriageway_splits: usize,
    crossings: BTreeSet<NodeID>,
    explicit_non_crossings: BTreeSet<NodeID>,
}

// Angle in degrees from first to last point. Includes the "direction" of the line.
fn angle_ls_directional(ls: &LineString) -> f64 {
    let pt1 = ls.coords().next().unwrap();
    let pt2 = ls.coords().last().unwrap();
    (pt2.y - pt1.y).atan2(pt2.x - pt1.x).to_degrees()
}
