use std::collections::BTreeSet;

use anyhow::Result;
use geo::Point;
use geojson::GeoJson;
use osm_reader::NodeID;

use crate::{
    Speedwalk,
    graph::{EdgeID, Graph, IntersectionID},
};

impl Speedwalk {
    pub fn audit_crossings(&self, ignore_utility_roads: bool) -> Result<String> {
        let mut features = Vec::new();

        let graph = Graph::new(self);

        for junction in self.find_junctions(ignore_utility_roads, &graph) {
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
                arms.len() == crossings.len() + explicit_non_crossings.len(),
            );
            f.set_property("arms", GeoJson::from(arms));
            f.set_property("crossings", GeoJson::from(crossings));
            f.set_property(
                "explicit_non_crossings",
                GeoJson::from(explicit_non_crossings),
            );

            features.push(f);
        }

        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }

    /// Find all junctions along severances
    fn find_junctions(&self, ignore_utility_roads: bool, graph: &Graph) -> Vec<Junction> {
        let mut junctions = Vec::new();
        for (i, intersection) in &graph.intersections {
            if self.derived_nodes[&intersection.osm_node]
                .tags
                .is("highway", "crossing")
            {
                continue;
            }

            let mut any_severances = false;
            let mut arms = Vec::new();
            let mut crossings = BTreeSet::new();
            let mut explicit_non_crossings = BTreeSet::new();
            for e in &intersection.edges {
                let edge = &graph.edges[e];
                let way = &self.derived_ways[&edge.osm_way];
                if way.is_severance() {
                    any_severances = true;
                }
                if ignore_utility_roads && way.tags.is_any("highway", vec!["service", "track"]) {
                    continue;
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

            if any_severances && arms.len() > 2 {
                junctions.push(Junction {
                    i: *i,
                    arms,
                    crossings,
                    explicit_non_crossings,
                });
            }
        }
        junctions
    }
}

struct Junction {
    i: IntersectionID,
    arms: Vec<EdgeID>,
    crossings: BTreeSet<NodeID>,
    explicit_non_crossings: BTreeSet<NodeID>,
}
