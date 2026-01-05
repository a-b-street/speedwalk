use std::collections::{BTreeMap, HashMap};

use geo::{LineString, Point};
use osm_reader::{NodeID, WayID};
use utils::Tags;

use crate::Speedwalk;

// TODO Adapted from utils::osm2graph. Not sure we need all of this.
pub struct Graph {
    pub edges: BTreeMap<EdgeID, Edge>,
    pub intersections: BTreeMap<IntersectionID, Intersection>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct EdgeID(usize);
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct IntersectionID(usize);

#[allow(unused)]
pub struct Edge {
    pub id: EdgeID,
    pub src: IntersectionID,
    pub dst: IntersectionID,

    pub osm_way: WayID,
    pub osm_node1: NodeID,
    pub osm_node2: NodeID,
    pub osm_tags: Tags,

    pub node_ids: Vec<NodeID>,
    /// Where does this edge begin and end within the way's list of nodes?
    pub idx_of_node1: usize,
    pub idx_of_node2: usize,

    pub linestring: LineString,
}

#[allow(unused)]
pub struct Intersection {
    pub id: IntersectionID,
    pub edges: Vec<EdgeID>,

    pub osm_node: NodeID,

    pub point: Point,
}

impl Graph {
    pub fn new(osm: &Speedwalk) -> Self {
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
            let mut idx_of_node1 = 0;
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
                            idx_of_node1,
                            idx_of_node2: idx,
                            node_ids: std::mem::take(&mut nodes),
                            osm_tags: way.tags.clone(),
                            linestring: LineString::new(std::mem::take(&mut pts)),
                        },
                    );

                    // Start the next edge
                    nodes.push(node);
                    node1 = node;
                    idx_of_node1 = idx;
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
