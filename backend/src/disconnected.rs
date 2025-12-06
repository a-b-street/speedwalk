use std::collections::BTreeSet;

use geo::{BoundingRect, Euclidean, Geometry, GeometryCollection, Length, Rect};
use geojson::FeatureCollection;
use osm_reader::WayID;
use petgraph::graphmap::UnGraphMap;

use crate::{
    Kind, Speedwalk,
    graph::{EdgeID, Graph, IntersectionID},
};

impl Speedwalk {
    pub fn find_connected_components(&self, split_graph: &Graph) -> FeatureCollection {
        let mut graph: UnGraphMap<IntersectionID, EdgeID> = UnGraphMap::new();
        for edge in split_graph.edges.values() {
            if matches!(
                self.derived_ways[&edge.osm_way].kind,
                Kind::Sidewalk | Kind::Crossing | Kind::Other
            ) {
                graph.add_edge(edge.src, edge.dst, edge.id);
            }
        }

        // (Ways, total length)
        let mut components: Vec<(BTreeSet<WayID>, usize)> = Vec::new();
        for nodes in petgraph::algo::kosaraju_scc(&graph) {
            let mut ways = nodes_to_ways(split_graph, nodes);
            ways.retain(|w| {
                matches!(
                    self.derived_ways[w].kind,
                    Kind::Sidewalk | Kind::Crossing | Kind::Other
                )
            });
            // There might be a component only with Kind::Other. Ignore these; we need sidewalks.
            if ways
                .iter()
                .all(|w| self.derived_ways[w].kind != Kind::Sidewalk)
            {
                continue;
            }

            let length = ways
                .iter()
                .map(|w| Euclidean.length(&self.derived_ways[w].linestring))
                .sum::<f64>()
                .round() as usize;
            components.push((ways, length));
        }
        components.sort_by_key(|(_, len)| *len);
        components.reverse();

        let mut features = Vec::new();
        let mut component_lengths = Vec::new();
        let mut component_bboxes = Vec::new();
        for (ways, length) in components {
            let component = component_lengths.len();
            component_lengths.push(length);

            let mut collection = Vec::new();
            for w in ways {
                let mut f = self.mercator.to_wgs84_gj(&self.derived_ways[&w].linestring);
                f.set_property("component", component);
                features.push(f);

                // TODO Expensive, make a bbox accumulator
                collection.push(Geometry::LineString(
                    self.derived_ways[&w].linestring.clone(),
                ));
            }
            let mut bbox: Rect = GeometryCollection(collection)
                .bounding_rect()
                .unwrap()
                .into();
            self.mercator.to_wgs84_in_place(&mut bbox);
            component_bboxes.push(vec![bbox.min().x, bbox.min().y, bbox.max().x, bbox.max().y]);
        }

        FeatureCollection {
            features,
            bbox: None,
            foreign_members: Some(into_object_value(serde_json::json!({
                "component_lengths": component_lengths,
                "component_bboxes": component_bboxes,
            }))),
        }
    }
}

// Note this only works for connected components of nodes!
fn nodes_to_ways(graph: &Graph, nodes: Vec<IntersectionID>) -> BTreeSet<WayID> {
    let mut ways = BTreeSet::new();
    for i in nodes {
        for e in &graph.intersections[&i].edges {
            ways.insert(graph.edges[e].osm_way);
        }
    }
    ways
}

// TODO Upstream
fn into_object_value(value: serde_json::Value) -> serde_json::Map<String, serde_json::Value> {
    value.as_object().unwrap().clone()
}
