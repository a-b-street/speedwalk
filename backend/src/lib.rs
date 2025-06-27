#[macro_use]
extern crate anyhow;
#[macro_use]
extern crate log;

mod classify;
mod crossings;
mod edits;
mod geometry;
mod scrape;

use std::collections::HashMap;
use std::sync::Once;

use anyhow::Result;
use geo::{Coord, Euclidean, Length, LineString, Point};
use geojson::GeoJson;
use osm_reader::{NodeID, WayID};
use serde::Serialize;
use utils::{Mercator, OffsetCurve, Tags};
use wasm_bindgen::prelude::*;

use crate::classify::{Kind, Quickfix};
use crate::edits::{Edits, UserCmd};

static START: Once = Once::new();

#[wasm_bindgen]
pub struct Speedwalk {
    original_nodes: HashMap<NodeID, Node>,
    original_ways: HashMap<WayID, Way>,
    mercator: Mercator,

    edits: Option<Edits>,

    derived_nodes: HashMap<NodeID, Node>,
    derived_ways: HashMap<WayID, Way>,
}

#[derive(Clone)]
pub struct Node {
    pub pt: Coord,
    pub tags: Tags,
    pub version: i32,

    // TODO This is managed state, right?
    pub way_ids: Vec<WayID>,
    pub modified: bool,
}

#[derive(Clone)]
pub struct Way {
    pub node_ids: Vec<NodeID>,
    pub linestring: LineString,
    pub tags: Tags,
    pub version: i32,

    // TODO Manage derived state better
    pub kind: Kind,
    pub num_crossings: usize,
    pub is_main_road: bool,
    // From the start of the line. first entry is 0, last entry is linestring length
    pub distance_per_node: Vec<f64>,
    pub modified: bool,
}

#[wasm_bindgen]
impl Speedwalk {
    #[wasm_bindgen(constructor)]
    pub fn new(input_bytes: &[u8]) -> Result<Speedwalk, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        scrape::scrape_osm(input_bytes).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = getNodes)]
    pub fn get_nodes(&self) -> Result<String, JsValue> {
        let mut features = Vec::new();
        // TODO HashMap nondet order
        for (idx, (id, node)) in self.derived_nodes.iter().enumerate() {
            let mut f = self.mercator.to_wgs84_gj(&Point::from(node.pt));
            f.id = Some(geojson::feature::Id::Number(idx.into()));
            f.set_property("id", id.0);
            if !node.tags.0.is_empty() {
                f.set_property("tags", serde_json::to_value(&node.tags).map_err(err_to_js)?);
            }
            f.set_property("is_crossing", node.is_crossing());
            f.set_property("modified", node.modified);
            features.push(f);
        }
        serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = getWays)]
    pub fn get_ways(&self) -> Result<String, JsValue> {
        let mut features = Vec::new();
        // TODO HashMap nondet order
        for (idx, (id, way)) in self.derived_ways.iter().enumerate() {
            let mut f = self.mercator.to_wgs84_gj(&way.linestring);
            f.id = Some(geojson::feature::Id::Number(idx.into()));
            f.set_property("id", id.0);
            f.set_property("tags", serde_json::to_value(&way.tags).map_err(err_to_js)?);
            f.set_property("kind", way.kind.to_simple_string());
            f.set_property("num_crossings", way.num_crossings);
            f.set_property("modified", way.modified);
            if let Kind::QuickfixRoadway(ref fix) = way.kind {
                f.set_property("fix", serde_json::to_value(&fix).map_err(err_to_js)?);
            }
            if let Kind::BadRoadway(ref problem) = way.kind {
                f.set_property(
                    "problem",
                    serde_json::to_value(&problem).map_err(err_to_js)?,
                );
            }
            features.push(f);
        }
        serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = getMetrics)]
    pub fn get_metrics(&self) -> Result<String, JsValue> {
        serde_json::to_string(&Metrics::new(self)).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = previewSidewalk)]
    pub fn preview_sidewalk(
        &self,
        base: i64,
        left_meters: f64,
        right_meters: f64,
        trim_back_from_crossings: Option<f64>,
    ) -> Result<String, JsValue> {
        let sidewalks = self
            .make_sidewalks(
                WayID(base),
                left_meters,
                right_meters,
                trim_back_from_crossings,
            )
            .map_err(err_to_js)?;

        let mut features = Vec::new();
        for new_sidewalk in sidewalks {
            features.push(self.mercator.to_wgs84_gj(&new_sidewalk.linestring));
            for (_, new_node, _) in new_sidewalk.crossing_points {
                features.push(self.mercator.to_wgs84_gj(&Point::from(new_node)));
            }
        }
        Ok(serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = getSideLocations)]
    pub fn get_side_locations(&self, id: i64) -> Result<String, JsValue> {
        let linestring = &self.derived_ways[&WayID(id)].linestring;
        let mut features = Vec::new();

        if let Some(ls) = linestring.offset_curve(20.0) {
            let mut f = self.mercator.to_wgs84_gj(&ls);
            f.set_property("side", "right");
            features.push(f);
        }
        if let Some(ls) = linestring.offset_curve(-20.0) {
            let mut f = self.mercator.to_wgs84_gj(&ls);
            f.set_property("side", "left");
            features.push(f);
        }

        Ok(serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = editMakeSidewalk)]
    pub fn edit_make_sidewalk(
        &mut self,
        base: i64,
        left_meters: f64,
        right_meters: f64,
        trim_back_from_crossings: Option<f64>,
    ) -> Result<(), JsValue> {
        let mut edits = self.edits.take().unwrap();
        edits
            .apply_cmd(
                UserCmd::MakeSidewalk(
                    WayID(base),
                    left_meters,
                    right_meters,
                    trim_back_from_crossings,
                ),
                self,
            )
            .map_err(err_to_js)?;
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = editMakeAllSidewalks)]
    pub fn edit_make_all_sidewalks(
        &mut self,
        trim_back_from_crossings: Option<f64>,
        assume_both_for_missing: bool,
        only_severances: bool,
    ) -> Result<(), JsValue> {
        let mut cmds = Vec::new();
        for (id, way) in &self.derived_ways {
            if only_severances && !way.is_severance() {
                continue;
            }

            if way.tags.is("sidewalk", "both") {
                cmds.push(UserCmd::MakeSidewalk(
                    *id,
                    3.0,
                    3.0,
                    trim_back_from_crossings,
                ));
            } else if way.tags.is("sidewalk", "left") {
                cmds.push(UserCmd::MakeSidewalk(
                    *id,
                    3.0,
                    0.0,
                    trim_back_from_crossings,
                ));
            } else if way.tags.is("sidewalk", "right") {
                cmds.push(UserCmd::MakeSidewalk(
                    *id,
                    0.0,
                    3.0,
                    trim_back_from_crossings,
                ));
            } else if assume_both_for_missing && !way.tags.is_any("sidewalk", vec!["no", "none"]) {
                cmds.push(UserCmd::MakeSidewalk(
                    *id,
                    3.0,
                    3.0,
                    trim_back_from_crossings,
                ));
            }
        }

        for cmd in cmds {
            let mut edits = self.edits.take().unwrap();
            // Some may fail; that's fine
            let _ = edits.apply_cmd(cmd, self);
            self.edits = Some(edits);
            self.after_edit();
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = editConnectAllCrossings)]
    pub fn edit_connect_all_crossings(&mut self) -> Result<(), JsValue> {
        let mut cmds = Vec::new();
        for node in self.get_all_crossings_on_severances() {
            cmds.push(UserCmd::ConnectCrossing(node));
        }

        for cmd in cmds {
            let mut edits = self.edits.take().unwrap();
            // Some may fail; that's fine
            let _ = edits.apply_cmd(cmd, self);
            self.edits = Some(edits);
            self.after_edit();
        }
        Ok(())
    }

    #[wasm_bindgen(js_name = editApplyQuickfix)]
    pub fn edit_apply_quickfix(&mut self, base: i64, quickfix: JsValue) -> Result<(), JsValue> {
        let quickfix: Quickfix = serde_wasm_bindgen::from_value(quickfix)?;
        let mut edits = self.edits.take().unwrap();
        edits
            .apply_cmd(UserCmd::ApplyQuickfix(WayID(base), quickfix), self)
            .map_err(err_to_js)?;
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = editClear)]
    pub fn edit_clear(&mut self) {
        self.edits = Some(Edits::default());
        self.after_edit();
    }

    /// List of UserCmd
    #[wasm_bindgen(js_name = getEdits)]
    pub fn get_edits(&self) -> Result<String, JsValue> {
        serde_json::to_string(&self.edits.as_ref().unwrap().user_commands).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = toOsc)]
    pub fn to_osc(&self) -> String {
        self.edits.as_ref().unwrap().to_osc(self)
    }

    #[wasm_bindgen(js_name = toOsmChangeJson)]
    pub fn to_osmchange_json(&self) -> Result<String, JsValue> {
        self.edits
            .as_ref()
            .unwrap()
            .to_osmchange_json(self)
            .map_err(err_to_js)
    }
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[derive(Default, Serialize)]
struct Metrics {
    total_length_meters: HashMap<&'static str, f64>,
}

impl Metrics {
    fn new(model: &Speedwalk) -> Self {
        let mut metrics = Self::default();
        for way in model.derived_ways.values() {
            *metrics
                .total_length_meters
                .entry(way.kind.to_simple_string())
                .or_insert(0.0) += Euclidean.length(&way.linestring);
        }
        metrics
    }
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
        !self.tags.is("highway", "residential")
    }
}
