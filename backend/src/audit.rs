use std::collections::{BTreeMap, HashMap};

use anyhow::Result;
use geo::{LineString, Point};
use geojson::GeoJson;
use osm_reader::{NodeID, WayID};
use utils::Tags;

use crate::Speedwalk;

impl Speedwalk {
    pub fn audit_crossings(&self, ignore_service_roads: bool) -> Result<String> {
        let mut features = Vec::new();

        let graph = Graph::new(self);

        for junction in self.find_junctions(ignore_service_roads, &graph) {
            let mut f = self.mercator.to_wgs84_gj(&graph.intersections[&junction.i].point);

            let mut debug_arms = Vec::new();
            for e in junction.arms {
                debug_arms.push(self.mercator.to_wgs84_gj(&graph.edges[&e].linestring));
            }
            f.set_property("arms", GeoJson::from(debug_arms));

            features.push(f);
        }

        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }

    /// Find all junctions along severances
    fn find_junctions(
        &self,
        ignore_service_roads: bool,
        graph: &Graph,
    ) -> Vec<Junction> {
        let mut junctions = Vec::new();
        for (i, intersection) in &graph.intersections {
            if self.derived_nodes[&intersection.osm_node].tags.is("highway", "crossing") {
                continue;
            }

            let mut any_severances = false;
            let mut arms = Vec::new();
            for e in &intersection.edges {
                let way = &self.derived_ways[&graph.edges[e].osm_way];
                if way.is_severance() {
                    any_severances = true;
                }
                if ignore_service_roads && way.tags.is("highway", "service") {
                    continue;
                }
                arms.push(*e);
            }

            if any_severances && arms.len() > 2 {
                junctions.push(Junction {
                    i: *i,
                    arms,
                });
            }
        }
        junctions
    }
}

struct Junction {
    i: IntersectionID,
    arms: Vec<EdgeID>,
}

// TODO Adapted from utils::osm2graph. Not sure we need all of this.
struct Graph {
    edges: BTreeMap<EdgeID, Edge>,
    intersections: BTreeMap<IntersectionID, Intersection>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
struct EdgeID(usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
struct IntersectionID(usize);

#[allow(unused)]
struct Edge {
    id: EdgeID,
    src: IntersectionID,
    dst: IntersectionID,

    osm_way: WayID,
    osm_node1: NodeID,
    osm_node2: NodeID,
    osm_tags: Tags,

    node_ids: Vec<NodeID>,

    linestring: LineString,
}

#[allow(unused)]
struct Intersection {
    id: IntersectionID,
    edges: Vec<EdgeID>,

    osm_node: NodeID,

    point: Point,
}

impl Graph {
    fn new(osm: &Speedwalk) -> Self {
        // Count how many ways reference each node
        let mut node_counter: HashMap<NodeID, usize> = HashMap::new();
        for way in osm.derived_ways.values() {
            for node in &way.node_ids {
                *node_counter.entry(*node).or_insert(0) += 1;
            }
        }

        // Split each way into edges
        let mut id_counter = 0;
        let mut node_to_intersection: HashMap<NodeID, IntersectionID> = HashMap::new();
        let mut intersections = BTreeMap::new();
        let mut edges = BTreeMap::new();
        for (way_id, way) in &osm.derived_ways {
            let mut node1 = way.node_ids[0];
            let mut pts = Vec::new();
            let mut nodes = Vec::new();

            let num_nodes = way.node_ids.len();
            for (idx, node) in way.node_ids.iter().cloned().enumerate() {
                pts.push(osm.derived_nodes[&node].pt);
                nodes.push(node);
                // Edges start/end at intersections between two ways. The endpoints of the way also
                // count as intersections.
                let is_endpoint =
                    idx == 0 || idx == num_nodes - 1 || *node_counter.get(&node).unwrap() > 1;
                if is_endpoint && pts.len() > 1 {
                    let edge_id = EdgeID(id_counter);
                    id_counter += 1;

                    let mut i_ids = Vec::new();
                    for (n, point) in [(node1, pts[0]), (node, *pts.last().unwrap())] {
                        let i = node_to_intersection.get(&n).cloned().unwrap_or_else(|| {
                            let i = IntersectionID(id_counter);
                            id_counter += 1;
                            intersections.insert(
                                i,
                                Intersection {
                                    id: i,
                                    osm_node: n,
                                    point: Point(point),
                                    edges: Vec::new(),
                                },
                            );
                            node_to_intersection.insert(n, i);
                            i
                        });
                        let intersection = intersections.get_mut(&i).unwrap();

                        intersection.edges.push(edge_id);
                        i_ids.push(i);
                    }

                    edges.insert(
                        edge_id,
                        Edge {
                            id: edge_id,
                            src: i_ids[0],
                            dst: i_ids[1],
                            osm_way: *way_id,
                            osm_node1: node1,
                            osm_node2: node,
                            node_ids: nodes.clone(),
                            osm_tags: way.tags.clone(),
                            linestring: LineString::new(std::mem::take(&mut pts)),
                        },
                    );

                    // Start the next edge
                    node1 = node;
                    pts.push(osm.derived_nodes[&node].pt);
                }
            }
        }

        Self {
            edges,
            intersections,
        }
    }
}
