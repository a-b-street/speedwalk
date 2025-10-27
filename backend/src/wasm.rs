use std::collections::BTreeMap;
use std::sync::Once;

use anyhow::Result;
use geo::{Euclidean, Length, Point};
use geojson::GeoJson;
use osm_reader::WayID;
use serde::Serialize;
use utils::{OffsetCurve, Tags};
use wasm_bindgen::prelude::*;

use crate::{Edits, Kind, Speedwalk, UserCmd};

static START: Once = Once::new();

#[wasm_bindgen]
impl Speedwalk {
    #[wasm_bindgen(constructor)]
    pub fn new(input_bytes: &[u8]) -> Result<Speedwalk, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        Speedwalk::new_from_osm(input_bytes).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = getOsmTimestamp)]
    pub fn get_osm_timestamp(&self) -> Option<i64> {
        self.timestamp
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
            f.set_property(
                "way_ids",
                node.way_ids.iter().map(|w| w.0).collect::<Vec<_>>(),
            );
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
            f.set_property("kind", format!("{:?}", way.kind));
            f.set_property("modified", way.modified);
            f.set_property(
                "node_ids",
                way.node_ids.iter().map(|n| n.0).collect::<Vec<_>>(),
            );
            f.set_property("is_severance", way.is_severance());
            features.push(f);
        }
        serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = getMetrics)]
    pub fn get_metrics(&self) -> Result<String, JsValue> {
        serde_json::to_string(&Metrics::new(self)).map_err(err_to_js)
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

    #[wasm_bindgen(js_name = editMakeAllSidewalksV2)]
    pub fn edit_make_all_sidewalks_v2(&mut self, only_severances: bool) -> Result<(), JsValue> {
        let mut edits = self.edits.take().unwrap();
        // Ignore failure?
        let _ = edits.apply_cmd(UserCmd::MakeAllSidewalksV2(only_severances), self);
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = editConnectAllCrossings)]
    pub fn edit_connect_all_crossings(&mut self) -> Result<(), JsValue> {
        let mut edits = self.edits.take().unwrap();
        // Ignore failure?
        let _ = edits.apply_cmd(UserCmd::ConnectAllCrossings, self);
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = editAssumeTags)]
    pub fn edit_assume_tags(&mut self, drive_on_left: bool) -> Result<(), JsValue> {
        let mut edits = self.edits.take().unwrap();
        // Ignore failure?
        let _ = edits.apply_cmd(UserCmd::AssumeTags(drive_on_left), self);
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = editSetTags)]
    pub fn edit_set_tags(&mut self, base: i64, tags: JsValue) -> Result<(), JsValue> {
        let tags: Vec<Vec<String>> = serde_wasm_bindgen::from_value(tags)?;
        let mut edits = self.edits.take().unwrap();
        edits
            .apply_cmd(
                UserCmd::SetTags(
                    WayID(base),
                    tags.into_iter()
                        .map(|mut kv| (kv.remove(0), kv.remove(0)))
                        .collect(),
                ),
                self,
            )
            .map_err(err_to_js)?;
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = editAddNewCrossing)]
    pub fn edit_add_new_crossing(&mut self, x: f64, y: f64) -> Result<(), JsValue> {
        let mut edits = self.edits.take().unwrap();
        let mut tags = Tags::empty();
        tags.insert("highway", "crossing");
        tags.insert("crossing", "traffic_signals");
        edits
            .apply_cmd(UserCmd::AddCrossing(Point::new(x, y), tags), self)
            .map_err(err_to_js)?;
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = editUndo)]
    pub fn edit_undo(&mut self) -> Result<(), JsValue> {
        let mut cmds = self.edits.take().unwrap().user_commands;
        cmds.pop();
        self.edits = Some(Edits::default());
        self.after_edit();

        // We have to start over and replay almost all the commands
        for cmd in cmds {
            let mut edits = self.edits.take().unwrap();
            edits.apply_cmd(cmd, self).map_err(err_to_js)?;
            self.edits = Some(edits);
            self.after_edit();
        }
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

    #[wasm_bindgen(js_name = findProblems)]
    pub fn find_problems_wasm(&self) -> Result<String, JsValue> {
        self.find_problems().map_err(err_to_js)
    }
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

#[derive(Default, Serialize)]
struct Metrics {
    total_length_meters: BTreeMap<Kind, f64>,
}

impl Metrics {
    fn new(model: &Speedwalk) -> Self {
        let mut metrics = Self::default();
        for way in model.derived_ways.values() {
            *metrics.total_length_meters.entry(way.kind).or_insert(0.0) +=
                Euclidean.length(&way.linestring);
        }
        metrics
    }
}
