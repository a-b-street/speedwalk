use anyhow::Result;
use geo::Point;
use geojson::GeoJson;

use crate::{Kind, Speedwalk};

impl Speedwalk {
    pub fn find_problems(&self) -> Result<String> {
        let mut features = Vec::new();

        // Look for footway=crossing ways that don't have crossing nodes on the roads
        for (_way_id, way) in &self.derived_ways {
            if !way.tags.is("footway", "crossing") {
                continue;
            }
            for node_id in &way.node_ids {
                let node = &self.derived_nodes[node_id];
                if node.is_crossing() {
                    continue;
                }
                if node.way_ids.iter().any(|other_way_id| {
                    !matches!(
                        self.derived_ways[other_way_id].kind,
                        Kind::Sidewalk | Kind::Other
                    )
                }) {
                    let mut f = self.mercator.to_wgs84_gj(&Point::from(node.pt));
                    f.set_property("osm", node_id.to_string());
                    f.set_property("problem", "missing crossing node");
                    features.push(f);
                }
            }
        }
        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }
}
