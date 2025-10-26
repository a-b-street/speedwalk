use anyhow::Result;
use geo::Point;
use geojson::GeoJson;
use rstar::{RTree, primitives::GeomWithData};
use utils::{aabb, buffer_aabb};

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

        // Look for footways involving crossing nodes that aren't marked footway=crossing
        for (way_id, way) in &self.derived_ways {
            if !matches!(self.derived_ways[way_id].kind, Kind::Sidewalk | Kind::Other) {
                continue;
            }
            if way.node_ids.iter().any(|n| {
                let node = &self.derived_nodes[n];
                // TODO This one is debatable
                // Only crossing nodes over severances -- it's normal for a sidewalk to have
                // side road crossings in the middle
                node.is_crossing()
                    && node
                        .way_ids
                        .iter()
                        .any(|w| self.derived_ways[w].is_severance())
            }) {
                let mut f = self.mercator.to_wgs84_gj(&way.linestring);
                f.set_property("osm", way_id.to_string());
                f.set_property("problem", "missing footway=crossing");
                features.push(f);
            }
        }

        // Look for roads without separate sidewalks marked, but that seem to be geometrically
        // close to separate sidewalks
        let closest_sidewalk = RTree::bulk_load(
            self.derived_ways
                .iter()
                .filter(|(_, way)| way.kind == Kind::Sidewalk)
                .map(|(id, way)| GeomWithData::new(way.linestring.clone(), *id))
                .collect(),
        );
        for (id, way) in &self.derived_ways {
            if way.kind != Kind::Road {
                continue;
            }
            let mut any_matches = false;
            for obj in closest_sidewalk
                .locate_in_envelope_intersecting(&buffer_aabb(aabb(&way.linestring), 15.0))
            {
                // TODO See if it's parallelish -- walknet's defn
                any_matches = true;
                break;
            }

            if any_matches {
                let mut f = self.mercator.to_wgs84_gj(&way.linestring);
                f.set_property("osm", id.to_string());
                f.set_property(
                    "problem",
                    "possible separate sidewalk near way without it tagged",
                );
                features.push(f);
            }
        }

        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }
}
