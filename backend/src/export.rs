use anyhow::Result;
use geo::{Euclidean, Length};
use geojson::GeoJson;
use serde::Deserialize;

use crate::{
    Kind, Speedwalk,
    graph::{Edge, Graph},
};

#[derive(Deserialize)]
pub struct NetworkFilter {
    include: NetworkFilterType,
    ignore_deadends: bool,
}

#[derive(Deserialize)]
pub enum NetworkFilterType {
    Everything,
    OnlyExplicitFootways,
    RouteableNetwork,
}

const MINIMUM_DEADEND_LENGTH: f64 = 10.0;

impl Speedwalk {
    pub fn filter_network(&self, filter: &NetworkFilter, graph: &Graph, edge: &Edge) -> bool {
        let way = &self.derived_ways[&edge.osm_way];

        match filter.include {
            NetworkFilterType::Everything => {}
            NetworkFilterType::OnlyExplicitFootways => {
                if way.kind.is_road() {
                    return false;
                }
            }
            NetworkFilterType::RouteableNetwork => {
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
                    Kind::Other => way.is_walkable_other(),
                };
                if !include {
                    return false;
                }

                // highway=proposed is filtered out upfront from Speedwalk, but construction is kept
                // for mapping, but isn't routeable.
                if way.tags.is("highway", "construction") {
                    return false;
                }
            }
        }

        if filter.ignore_deadends
            && (graph.intersections[&edge.src].edges.len() == 1
                || graph.intersections[&edge.dst].edges.len() == 1)
            && Euclidean.length(&edge.linestring) < MINIMUM_DEADEND_LENGTH
        {
            return false;
        }

        true
    }

    pub fn export_network(&self, filter: NetworkFilter) -> Result<String> {
        let graph = Graph::new(self);
        let mut features = Vec::new();
        for edge in graph.edges.values() {
            if self.filter_network(&filter, &graph, edge) {
                let mut f = self.mercator.to_wgs84_gj(&edge.linestring);
                let way = &self.derived_ways[&edge.osm_way];

                f.set_property("node1", edge.osm_node1.0);
                f.set_property("node2", edge.osm_node2.0);
                f.set_property("way", edge.osm_way.0);

                // Determine osm_id from tags stored during generation, or fallback to way ID
                if let Some(osm_node_id) = way.tags.get("tmp:osm_node_id") {
                    // OSM node reference (for crossings)
                    f.set_property("osm_id", osm_node_id.clone());
                } else if let Some(osm_way_id) = way.tags.get("tmp:osm_way_id") {
                    // OSM way reference (for sidewalks and crossings fallback)
                    f.set_property("osm_id", osm_way_id.clone());
                } else {
                    // Regular way - use the way ID (always available as final fallback)
                    f.set_property("osm_id", format!("way/{}", edge.osm_way.0));
                }

                f.set_property("kind", format!("{:?}", way.kind));

                for (k, v) in &way.tags.0 {
                    f.set_property(k.to_string(), v.to_string());
                }

                features.push(f);
            }
        }
        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }
}
