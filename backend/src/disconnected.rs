use std::collections::BTreeSet;

use geo::{BoundingRect, Euclidean, Geometry, GeometryCollection, Length, Rect};
use geojson::FeatureCollection;
use osm_reader::WayID;
use petgraph::graphmap::UnGraphMap;

use crate::{
    Speedwalk,
    export::NetworkFilter,
    graph::{EdgeID, Graph, IntersectionID},
};

impl Speedwalk {
    pub fn find_connected_components(
        &self,
        graph: &Graph,
        filter: &NetworkFilter,
    ) -> FeatureCollection {
        let mut scc_graph: UnGraphMap<IntersectionID, EdgeID> = UnGraphMap::new();
        for edge in graph.edges.values() {
            if self.filter_network(filter, graph, edge) {
                scc_graph.add_edge(edge.src, edge.dst, edge.id);
            }
        }

        // (Ways, total length)
        let mut components: Vec<(BTreeSet<WayID>, usize)> = Vec::new();
        for nodes in petgraph::algo::kosaraju_scc(&scc_graph) {
            let mut ways = BTreeSet::new();
            for i in nodes {
                for e in &graph.intersections[&i].edges {
                    if self.filter_network(filter, graph, &graph.edges[e]) {
                        ways.insert(graph.edges[e].osm_way);
                    }
                }
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
            foreign_members: Some(utils::into_object_value(serde_json::json!({
                "component_lengths": component_lengths,
                "component_bboxes": component_bboxes,
            }))),
        }
    }
}
