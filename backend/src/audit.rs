use std::collections::BTreeSet;

use anyhow::Result;
use geo::Point;
use geojson::GeoJson;
use osm_reader::NodeID;

use crate::Speedwalk;

impl Speedwalk {
    pub fn audit_crossings(&self, ignore_service_roads: bool) -> Result<String> {
        let mut features = Vec::new();

        for node in self.find_junctions(ignore_service_roads) {
            let f = self
                .mercator
                .to_wgs84_gj(&Point::from(self.derived_nodes[&node].pt));
            features.push(f);
        }

        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }

    /// Find all junctions along severances
    fn find_junctions(&self, ignore_service_roads: bool) -> BTreeSet<NodeID> {
        let mut nodes = BTreeSet::new();
        for (way_id, way) in &self.derived_ways {
            if !way.is_severance() {
                continue;
            }

            for node_id in &way.node_ids {
                // TODO This counting is completely wrong; we're not working with a graph
                if self.derived_nodes[node_id].way_ids.len() <= 2 {
                    continue;
                }
                if ignore_service_roads
                    && self.derived_nodes[node_id]
                        .way_ids
                        .iter()
                        .filter(|w| !self.derived_ways[w].tags.is("highway", "service"))
                        .count()
                        <= 2
                {
                    continue;
                }

                nodes.insert(*node_id);
            }
        }
        nodes
    }
}
