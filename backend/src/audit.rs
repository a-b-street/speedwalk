use std::collections::BTreeSet;

use anyhow::Result;
use geo::{LineString, Point};
use geojson::GeoJson;
use osm_reader::NodeID;
use serde::Deserialize;

use crate::{
    Speedwalk,
    crossings::shortest_rotation,
    graph::{EdgeID, Graph, IntersectionID},
};

#[derive(Deserialize)]
pub struct Options {
    only_major_roads: bool,
    ignore_utility_roads: bool,
    ignore_cycleways: bool,
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
            for e in junction.arms {
                let edge = &graph.edges[&e];
                arms.push(self.mercator.to_wgs84_gj(&edge.linestring));
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
                arms.push(*e);

                // Iterate along the edge away from the intersection, stopping at crossing=no
                let node_iter: Box<dyn Iterator<Item = &NodeID>> = if edge.src == *i {
                    // Iterate forward from src to dst
                    Box::new(edge.node_ids.iter().skip(1))
                } else if edge.dst == *i {
                    // Iterate backward from dst to src
                    Box::new(edge.node_ids.iter().rev().skip(1))
                } else {
                    unreachable!()
                };

                for n in node_iter {
                    let node = &self.derived_nodes[n];
                    if node.is_explicit_crossing_no() {
                        explicit_non_crossings.insert(*n);
                        // Stop iterating along this edge when we hit crossing=no
                        break;
                    }
                    if node.is_crossing() {
                        crossings.insert(*n);
                        // Stop iterating along this edge when we hit the first crossing
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
        arms: &Vec<EdgeID>,
    ) -> usize {
        // For each one-way road, track its (name, whether it points at the intersection (true)
        // or away from it (false), angle)
        let mut oneway_roads: Vec<(String, bool, f64)> = Vec::new();
        let mut num_splits = 0;
        for e in arms {
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
}

struct Junction {
    i: IntersectionID,
    arms: Vec<EdgeID>,
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
