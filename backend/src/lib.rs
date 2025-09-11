#[allow(unused)]
#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod classify;
mod crossings;
mod edits;
mod make_sidewalks_v2;
mod problems;
mod scrape;
mod wasm;

use std::collections::HashMap;

use anyhow::Result;
use geo::{Coord, LineString};
use osm_reader::{NodeID, WayID};
use utils::{Mercator, Tags};
use wasm_bindgen::prelude::*;

pub use crate::classify::{Kind, Quickfix};
pub use crate::edits::{Edits, UserCmd};

#[wasm_bindgen]
pub struct Speedwalk {
    original_nodes: HashMap<NodeID, Node>,
    original_ways: HashMap<WayID, Way>,
    mercator: Mercator,
    timestamp: Option<i64>,

    edits: Option<Edits>,

    derived_nodes: HashMap<NodeID, Node>,
    derived_ways: HashMap<WayID, Way>,
}

impl Speedwalk {
    pub fn new_from_osm(input_bytes: &[u8]) -> Result<Speedwalk> {
        crate::scrape::scrape_osm(input_bytes)
    }
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
}

#[derive(Clone)]
pub struct Way {
    pub node_ids: Vec<NodeID>,
    pub linestring: LineString,
    pub tags: Tags,
    pub version: i32,

    // TODO Manage derived state better. Everything below is suspect.
    pub kind: Kind,
    pub is_main_road: bool,
    pub modified: bool,
}

impl Node {
    pub fn is_crossing(&self) -> bool {
        self.tags.is("highway", "crossing")
            || (self.tags.is("highway", "traffic_signals")
                && self.tags.is("crossing", "traffic_signals"))
    }
}

impl Way {
    pub fn is_severance(&self) -> bool {
        // TODO Improve
        !self.tags.is_any(
            "highway",
            vec!["construction", "residential", "service", "unclassified"],
        )
    }
}
