#[allow(unused)]
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod audit;
mod classify;
mod crossings;
mod disconnected;
mod edits;
mod export;
mod graph;
mod make_sidewalks;
mod problems;
mod scrape;
mod wasm;

use std::collections::HashMap;

use anyhow::Result;
use geo::{Coord, Intersects, LineString, Polygon, Rect};
use geojson::Feature;
use osm_reader::{NodeID, WayID};
use rstar::RTree;
use serde::Serialize;
use utils::{Mercator, Tags};
use wasm_bindgen::prelude::*;

pub use crate::classify::Kind;
pub use crate::edits::{Edits, UserCmd};

#[wasm_bindgen]
pub struct Speedwalk {
    original_nodes: HashMap<NodeID, Node>,
    original_ways: HashMap<WayID, Way>,
    mercator: Mercator,
    boundary_wgs84: Polygon,
    pub timestamp: Option<i64>,
    closest_building: RTree<Polygon>,

    edits: Option<Edits>,

    derived_nodes: HashMap<NodeID, Node>,
    derived_ways: HashMap<WayID, Way>,
}

impl Speedwalk {
    pub fn new_from_osm(input_bytes: &[u8], boundary_wgs84: Option<Polygon>) -> Result<Speedwalk> {
        crate::scrape::scrape_osm(input_bytes, boundary_wgs84)
    }

    // TODO Workaround wasm stuff
    pub fn take_edits(&mut self) -> Edits {
        self.edits.take().unwrap()
    }

    pub fn set_edits(&mut self, edits: Edits) {
        self.edits = Some(edits);
    }

    /// Export original OSM data filtered by bounding box as OSM XML
    /// bbox: [min_lon, min_lat, max_lon, max_lat]
    pub fn export_osm_for_viewport(&self, bbox: [f64; 4]) -> Result<String> {
        let [min_lon, min_lat, max_lon, max_lat] = bbox;
        let bbox_rect = Rect::new(
            Coord { x: min_lon, y: min_lat },
            Coord { x: max_lon, y: max_lat },
        );

        // Find ways that intersect the bbox
        let mut ways_in_bbox = std::collections::HashSet::new();
        let mut nodes_in_bbox = std::collections::HashSet::new();

        for (way_id, way) in &self.original_ways {
            // Convert way linestring to WGS84 for bbox check
            let mut way_wgs84 = way.linestring.clone();
            self.mercator.to_wgs84_in_place(&mut way_wgs84);

            // Check if way intersects bbox
            if way_wgs84.intersects(&bbox_rect) {
                ways_in_bbox.insert(*way_id);
                // Include all nodes referenced by this way
                for node_id in &way.node_ids {
                    nodes_in_bbox.insert(*node_id);
                }
            }
        }

        // Also include nodes that are within the bbox (even if not part of ways)
        for (node_id, node) in &self.original_nodes {
            let pt_wgs84 = self.mercator.pt_to_wgs84(node.pt);
            let pt = geo::Point::new(pt_wgs84.x, pt_wgs84.y);
            if bbox_rect.intersects(&pt) {
                nodes_in_bbox.insert(*node_id);
            }
        }

        // Build OSM XML
        let mut out = vec![r#"<osm version="0.6" generator="Speedwalk">"#.to_string()];

        // Export nodes
        for node_id in &nodes_in_bbox {
            if let Some(node) = self.original_nodes.get(node_id) {
                out.push(format_node_xml(node_id, node, &self.mercator, 2));
            }
        }

        // Export ways
        for way_id in &ways_in_bbox {
            if let Some(way) = self.original_ways.get(way_id) {
                out.push(format_way_xml(way_id, way, 2));
            }
        }

        out.push("</osm>".to_string());

        Ok(out.join("\n"))
    }
}

/// Format a node as OSM XML
/// indent: number of spaces for indentation (2 for <osm>, 4 for <osmChange>)
pub(crate) fn format_node_xml(node_id: &osm_reader::NodeID, node: &Node, mercator: &Mercator, indent: usize) -> String {
    let pt_wgs84 = mercator.pt_to_wgs84(node.pt);
    let indent_str = " ".repeat(indent);
    let tag_indent = " ".repeat(indent + 2);
    let mut out = vec![format!(
        r#"{indent_str}<node id="{}" version="{}" lon="{}" lat="{}">"#,
        node_id.0, node.version, pt_wgs84.x, pt_wgs84.y
    )];
    for (k, v) in &node.tags.0 {
        out.push(format!(
            r#"{tag_indent}<tag k="{k}" v="{}" />"#,
            crate::edits::escape(v)
        ));
    }
    out.push(format!(r#"{indent_str}</node>"#));
    out.join("\n")
}

/// Format a way as OSM XML
/// indent: number of spaces for indentation (2 for <osm>, 4 for <osmChange>)
pub(crate) fn format_way_xml(way_id: &osm_reader::WayID, way: &Way, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    let tag_indent = " ".repeat(indent + 2);
    let mut out = vec![format!(
        r#"{indent_str}<way id="{}" version="{}">"#,
        way_id.0, way.version
    )];
    for node_id in &way.node_ids {
        out.push(format!(r#"{tag_indent}<nd ref="{}" />"#, node_id.0));
    }
    for (k, v) in &way.tags.0 {
        out.push(format!(
            r#"{tag_indent}<tag k="{k}" v="{}" />"#,
            crate::edits::escape(v)
        ));
    }
    out.push(format!(r#"{indent_str}</way>"#));
    out.join("\n")
}

#[derive(Clone)]
pub struct Node {
    pub pt: Coord,
    pub tags: Tags,
    pub version: i32,

    // Derived state, recalculated
    pub way_ids: Vec<WayID>,
    // Only used in the UI. TODO might be wrong.
    pub modified: bool,
    pub problems: Vec<Problem>,
}

#[derive(Clone)]
pub struct Way {
    pub node_ids: Vec<NodeID>,
    pub linestring: LineString,
    pub tags: Tags,
    pub version: i32,

    // TODO Manage derived state better. Everything below is suspect.
    pub kind: Kind,
    pub modified: bool,
    pub problems: Vec<Problem>,
}

impl Node {
    pub fn is_crossing(&self) -> bool {
        // Include any node with a crossing tag value != "no" (informal, traffic_signals, yes, etc.)
        if let Some(value) = self.tags.get("crossing") {
            return value != "no";
        }
        self.tags.is("highway", "crossing")
            || (self.tags.is("highway", "traffic_signals")
                && self.tags.is("crossing", "traffic_signals"))
    }

    pub fn is_explicit_crossing_no(&self) -> bool {
        self.tags.is("crossing", "no")
    }
}

impl Way {
    pub fn is_severance(&self) -> bool {
        self.tags.is_any(
            "highway",
            vec![
                "motorway",
                "motorway_link",
                "trunk",
                "trunk_link",
                "primary",
                "primary_link",
                "secondary",
                "secondary_link",
                "tertiary",
                "tertiary_link",
            ],
        )
    }

    /// For Kind::Other cases (often cycleways or paths), is the way usable for walking?
    pub fn is_walkable_other(&self) -> bool {
        if self.kind != Kind::Other {
            return false;
        }
        if self.tags.is("highway", "cycleway") {
            self.tags.is_any("foot", vec!["yes", "designated"])
        } else {
            // All other cases are routeable
            true
        }
    }
}

#[derive(Clone, Serialize)]
pub struct Problem {
    pub note: String,
    pub details: Vec<Feature>,
}
