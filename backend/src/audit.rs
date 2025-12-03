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
    pub fn audit_crossings(&self, ignore_service_roads: bool) -> Result<String> {
        let mut features = Vec::new();

        let graph = Graph::new(self);

        for junction in self.find_junctions(ignore_service_roads, &graph) {
            let mut f = self
                .mercator
                .to_wgs84_gj(&graph.intersections[&junction.i].point);

            let mut debug_arms = Vec::new();
            for e in junction.arms {
                let edge = &graph.edges[&e];
                debug_arms.push(self.mercator.to_wgs84_gj(&edge.linestring));
            }
            let mut debug_crossings = Vec::new();
            for n in junction.crossings {
                debug_crossings.push(
                    self.mercator
                        .to_wgs84_gj(&Point::from(self.derived_nodes[&n].pt)),
                );
            }
            f.set_property("complete", debug_arms.len() == debug_crossings.len());
            f.set_property("arms", GeoJson::from(debug_arms));
            f.set_property("crossings", GeoJson::from(debug_crossings));

            features.push(f);
        }

        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }

    /// Find all junctions along severances
    fn find_junctions(&self, ignore_service_roads: bool, graph: &Graph) -> Vec<Junction> {
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
            for e in &intersection.edges {
                let edge = &graph.edges[e];
                let way = &self.derived_ways[&edge.osm_way];
                if way.is_severance() {
                    any_severances = true;
                }
                if ignore_service_roads && way.tags.is("highway", "service") {
                    continue;
                }
                arms.push(*e);

                // TODO Simple definition of "nearby crossings", with both false positives and
                // negatives
                for n in &edge.node_ids {
                    if self.derived_nodes[n].is_crossing() {
                        crossings.insert(*n);
                    }
                }
            }

            if any_severances && arms.len() > 2 {
                junctions.push(Junction {
                    i: *i,
                    arms,
                    crossings,
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
}
