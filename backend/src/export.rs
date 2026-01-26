use anyhow::Result;
use geo::{Euclidean, Length};
use geojson::GeoJson;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};

use crate::{
    Kind, Speedwalk,
    graph::{Edge, EdgeID, Graph, IntersectionID},
};

#[derive(Deserialize)]
pub struct NetworkFilter {
    include: NetworkFilterType,
    ignore_deadends: bool,
}

impl NetworkFilter {
    pub fn ignore_deadends(&self) -> bool {
        self.ignore_deadends
    }
}

#[derive(Deserialize)]
pub enum NetworkFilterType {
    Everything,
    OnlyExplicitFootways,
    RouteableNetwork,
}

const MINIMUM_DEADEND_LENGTH: f64 = 10.0;
const MINIMUM_DISCONNECTED_LENGTH: f64 = 100.0;

impl Speedwalk {
    /// Filter network without dead end check - used to determine routeable edges
    fn filter_network_without_deadends(
        &self,
        filter: &NetworkFilter,
        _graph: &Graph,
        edge: &Edge,
    ) -> bool {
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

        true
    }

    /// Find all edges that are part of dead end chains (< 10m total length) and disconnected segments (< 100m)
    pub(crate) fn find_dead_end_chains(
        &self,
        filter: &NetworkFilter,
        graph: &Graph,
    ) -> HashSet<EdgeID> {
        // Pre-compute and cache edge lengths
        let mut edge_lengths: HashMap<EdgeID, f64> = HashMap::new();
        for edge in graph.edges.values() {
            edge_lengths.insert(edge.id, Euclidean.length(&edge.linestring));
        }

        // First pass: Determine routeable edges (without dead end filter)
        let mut routeable_edges: HashSet<EdgeID> = HashSet::new();
        for edge in graph.edges.values() {
            if self.filter_network_without_deadends(filter, graph, edge) {
                routeable_edges.insert(edge.id);
            }
        }

        // Early filtering: Only consider edges < 10m as candidates
        let candidate_edges: HashSet<EdgeID> = routeable_edges
            .iter()
            .filter(|&&edge_id| {
                edge_lengths.get(&edge_id).map_or(false, |&len| len < MINIMUM_DEADEND_LENGTH)
            })
            .copied()
            .collect();

        // Count routeable edges per intersection
        let mut intersection_routeable_count: HashMap<IntersectionID, usize> = HashMap::new();
        for &edge_id in &routeable_edges {
            let edge = &graph.edges[&edge_id];
            *intersection_routeable_count.entry(edge.src).or_insert(0) += 1;
            *intersection_routeable_count.entry(edge.dst).or_insert(0) += 1;
        }

        // Find endpoint intersections (only 1 routeable edge) that have candidate edges
        let mut endpoint_intersections: Vec<IntersectionID> = Vec::new();
        for (intersection_id, count) in &intersection_routeable_count {
            if *count == 1 {
                // Check if this intersection has at least one candidate edge
                let intersection = &graph.intersections[intersection_id];
                if intersection.edges.iter().any(|&e| candidate_edges.contains(&e)) {
                    endpoint_intersections.push(*intersection_id);
                }
            }
        }

        let mut dead_end_edges: HashSet<EdgeID> = HashSet::new();
        let mut visited_edges: HashSet<EdgeID> = HashSet::new();

        // Build chains from each endpoint intersection
        for &start_intersection in &endpoint_intersections {
            let intersection = &graph.intersections[&start_intersection];
            
            // Find the routeable edge connected to this endpoint
            let start_edge_id = intersection
                .edges
                .iter()
                .find(|&&e| routeable_edges.contains(&e))
                .copied();

            if let Some(start_edge_id) = start_edge_id {
                if visited_edges.contains(&start_edge_id) {
                    continue;
                }

                // Traverse the chain from this endpoint
                // Build the chain, but stop early if we exceed 10m (we won't remove it anyway)
                let mut chain_edges: Vec<EdgeID> = Vec::new();
                let mut chain_length = 0.0;
                let mut current_edge_id = start_edge_id;
                let mut current_intersection = start_intersection;

                loop {
                    if !routeable_edges.contains(&current_edge_id) {
                        break;
                    }

                    let edge_length = edge_lengths[&current_edge_id];
                    
                    // Early termination: If adding this edge would exceed 10m, stop building
                    // We still mark it as visited but don't add to chain (we won't remove it)
                    if chain_length + edge_length >= MINIMUM_DEADEND_LENGTH {
                        visited_edges.insert(current_edge_id);
                        break;
                    }

                    // Add edge to chain and continue building
                    chain_edges.push(current_edge_id);
                    chain_length += edge_length;
                    visited_edges.insert(current_edge_id);

                    // Find the next intersection (the one we haven't visited yet)
                    let edge = &graph.edges[&current_edge_id];
                    let next_intersection = if edge.src == current_intersection {
                        edge.dst
                    } else {
                        edge.src
                    };

                    // Check if we've reached another endpoint or branching intersection
                    // Count how many OTHER routeable edges exist at this intersection
                    // (excluding the current edge we're traversing)
                    let next_intersection_obj = &graph.intersections[&next_intersection];
                    let other_routeable_edges: Vec<EdgeID> = next_intersection_obj
                        .edges
                        .iter()
                        .filter(|&&e| {
                            routeable_edges.contains(&e)
                                && e != current_edge_id
                        })
                        .copied()
                        .collect();

                    if other_routeable_edges.is_empty() {
                        // Reached another endpoint (no other routeable edges) - chain is complete
                        // Only remove if the entire chain is < 10m
                        if chain_length < MINIMUM_DEADEND_LENGTH {
                            dead_end_edges.extend(&chain_edges);
                        }
                        break;
                    } else if other_routeable_edges.len() >= 2 {
                        // Reached a branching intersection (2+ other routeable edges) - chain ends here
                        // Only remove if the entire chain is < 10m
                        if chain_length < MINIMUM_DEADEND_LENGTH {
                            dead_end_edges.extend(&chain_edges);
                        }
                        break;
                    }

                    // Find the next routeable edge from this intersection (there's exactly 1 other)
                    let next_edge_id = other_routeable_edges
                        .iter()
                        .find(|&&e| !visited_edges.contains(&e))
                        .copied();

                    if let Some(next_id) = next_edge_id {
                        current_edge_id = next_id;
                        current_intersection = next_intersection;
                    } else {
                        // No more routeable edges - chain ends
                        // Only remove if the entire chain is < 10m
                        if chain_length < MINIMUM_DEADEND_LENGTH {
                            dead_end_edges.extend(&chain_edges);
                        }
                        break;
                    }
                }
            }
        }

        // Second pass: After removing dead ends, find disconnected mini-networks in the remaining network
        // Remove dead end edges from routeable set to get the remaining network
        let remaining_routeable_edges: HashSet<EdgeID> = routeable_edges
            .difference(&dead_end_edges)
            .copied()
            .collect();

        if remaining_routeable_edges.is_empty() {
            return dead_end_edges;
        }

        // Build a graph of remaining routeable edges to find connected components
        // Map each intersection to its connected edges
        let mut intersection_to_edges: HashMap<IntersectionID, Vec<EdgeID>> = HashMap::new();
        for &edge_id in &remaining_routeable_edges {
            let edge = &graph.edges[&edge_id];
            intersection_to_edges.entry(edge.src).or_insert_with(Vec::new).push(edge_id);
            intersection_to_edges.entry(edge.dst).or_insert_with(Vec::new).push(edge_id);
        }

        // Find all connected components using DFS
        let mut visited_intersections: HashSet<IntersectionID> = HashSet::new();
        let mut visited_edges_in_components: HashSet<EdgeID> = HashSet::new();

        for &start_edge_id in &remaining_routeable_edges {
            if visited_edges_in_components.contains(&start_edge_id) {
                continue;
            }

            // Start a new component from this edge
            let mut component_edges: Vec<EdgeID> = Vec::new();
            let mut component_intersections: HashSet<IntersectionID> = HashSet::new();
            let mut stack: Vec<IntersectionID> = Vec::new();

            let start_edge = &graph.edges[&start_edge_id];
            stack.push(start_edge.src);
            component_intersections.insert(start_edge.src);
            component_intersections.insert(start_edge.dst);
            component_edges.push(start_edge_id);
            visited_edges_in_components.insert(start_edge_id);

            // DFS to find all connected edges in this component
            while let Some(intersection_id) = stack.pop() {
                if visited_intersections.contains(&intersection_id) {
                    continue;
                }
                visited_intersections.insert(intersection_id);

                if let Some(connected_edges) = intersection_to_edges.get(&intersection_id) {
                    for &edge_id in connected_edges {
                        if visited_edges_in_components.contains(&edge_id) {
                            continue;
                        }

                        let edge = &graph.edges[&edge_id];
                        component_edges.push(edge_id);
                        visited_edges_in_components.insert(edge_id);

                        // Add the other endpoint to the stack
                        let next_intersection = if edge.src == intersection_id {
                            edge.dst
                        } else {
                            edge.src
                        };

                        if !component_intersections.contains(&next_intersection) {
                            component_intersections.insert(next_intersection);
                            stack.push(next_intersection);
                        }
                    }
                }
            }

            // Calculate total length of this component
            let component_length: f64 = component_edges
                .iter()
                .map(|&e| edge_lengths[&e])
                .sum();

            // If component is < 100m, remove all edges in it
            if component_length < MINIMUM_DISCONNECTED_LENGTH {
                dead_end_edges.extend(&component_edges);
            }
        }

        dead_end_edges
    }

    pub fn filter_network(
        &self,
        filter: &NetworkFilter,
        graph: &Graph,
        edge: &Edge,
        dead_end_edges: Option<&HashSet<EdgeID>>,
    ) -> bool {
        // Apply filters without dead end check
        if !self.filter_network_without_deadends(filter, graph, edge) {
            return false;
        }

        // Apply dead end filter if enabled
        if filter.ignore_deadends() {
            if let Some(dead_ends) = dead_end_edges {
                if dead_ends.contains(&edge.id) {
                    return false;
                }
            } else {
                // Fallback to old per-edge check if dead_end_edges not provided
                if (graph.intersections[&edge.src].edges.len() == 1
                    || graph.intersections[&edge.dst].edges.len() == 1)
                    && Euclidean.length(&edge.linestring) < MINIMUM_DEADEND_LENGTH
                {
                    return false;
                }
            }
        }

        true
    }

    pub fn export_network(&self, filter: NetworkFilter) -> Result<String> {
        let graph = Graph::new(self);
        
        // If dead end filtering is enabled, use chain-based approach
        let dead_end_edges = if filter.ignore_deadends() {
            Some(self.find_dead_end_chains(&filter, &graph))
        } else {
            None
        };

        let mut features = Vec::new();
        for edge in graph.edges.values() {
            if self.filter_network(&filter, &graph, edge, dead_end_edges.as_ref()) {
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
