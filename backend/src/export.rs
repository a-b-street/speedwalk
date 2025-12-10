use anyhow::Result;
use geo::{Euclidean, Length};
use geojson::GeoJson;
use serde::Deserialize;

use crate::{Kind, Speedwalk, graph::Graph};

#[derive(Deserialize)]
pub struct Options {
    ignore_deadends: bool,
}

const MINIMUM_DEADEND_LENGTH: f64 = 10.0;

impl Speedwalk {
    pub fn export_network(&self, options: Options) -> Result<String> {
        let graph = Graph::new(self);
        let mut features = Vec::new();
        for edge in graph.edges.values() {
            let way = &self.derived_ways[&edge.osm_way];
            let include = match way.kind {
                // Use the separate footways to route
                // TODO Even if it's only separate on one side, but tagged for the other?
                Kind::RoadWithSeparate => false,
                Kind::RoadWithTags => true,
                // Small streets with no sidewalks are routeable
                Kind::RoadWithoutSidewalksExplicit | Kind::RoadWithoutSidewalksImplicit => {
                    way.tags.is_any(
                        "highway",
                        vec!["living_street", "pedestrian", "residential", "service"],
                    )
                }
                // We have to assume yes
                Kind::RoadUnknown => true,
                Kind::Sidewalk | Kind::Crossing => true,
                Kind::Other => {
                    if way.tags.is("highway", "cycleway") {
                        way.tags.is_any("foot", vec!["yes", "designated"])
                    } else {
                        // All other cases are routeable
                        true
                    }
                }
            };
            if !include {
                continue;
            }

            // highway=proposed is filtered out upfront from Speedwalk, but construction is kept
            // for mapping, but isn't routeable.
            if way.tags.is("highway", "construction") {
                continue;
            }

            if options.ignore_deadends
                && (graph.intersections[&edge.src].edges.len() == 1
                    || graph.intersections[&edge.dst].edges.len() == 1)
                && Euclidean.length(&edge.linestring) < MINIMUM_DEADEND_LENGTH
            {
                continue;
            }

            let mut f = self.mercator.to_wgs84_gj(&edge.linestring);

            f.set_property("node1", edge.osm_node1.0);
            f.set_property("node2", edge.osm_node2.0);
            f.set_property("way", edge.osm_way.0);

            f.set_property("kind", format!("{:?}", way.kind));

            for (k, v) in &way.tags.0 {
                f.set_property(k.to_string(), v.to_string());
            }

            features.push(f);
        }
        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }
}
