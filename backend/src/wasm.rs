use std::collections::BTreeMap;
use std::sync::Once;

use anyhow::Result;
use geo::{Euclidean, Length, Point, Polygon};
use geojson::{Feature, GeoJson, Geometry};
use osm_reader::WayID;
use serde::Serialize;
use utils::{OffsetCurve, Tags};
use wasm_bindgen::prelude::*;

use crate::{Edits, Kind, Speedwalk, UserCmd};

static START: Once = Once::new();

#[wasm_bindgen]
impl Speedwalk {
    #[wasm_bindgen(constructor)]
    pub fn new(input_bytes: &[u8], boundary: JsValue) -> Result<Speedwalk, JsValue> {
        // Panics shouldn't happen, but if they do, console.log them.
        console_error_panic_hook::set_once();
        START.call_once(|| {
            console_log::init_with_level(log::Level::Info).unwrap();
        });

        let boundary_f: Option<Feature> = serde_wasm_bindgen::from_value(boundary)?;
        let boundary_wgs84: Option<Polygon> = match boundary_f {
            Some(f) => Some(f.try_into().map_err(err_to_js)?),
            None => None,
        };

        Speedwalk::new_from_osm(input_bytes, boundary_wgs84).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = getOsmTimestamp)]
    pub fn get_osm_timestamp(&self) -> Option<i64> {
        self.timestamp
    }

    #[wasm_bindgen(js_name = getBoundary)]
    pub fn get_boundary(&self) -> Result<String, JsValue> {
        Ok(
            serde_json::to_string(&Feature::from(Geometry::from(&self.boundary_wgs84)))
                .map_err(err_to_js)?,
        )
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
            f.set_property("is_explicit_crossing_no", node.is_explicit_crossing_no());
            f.set_property(
                "is_generated_crossing",
                node.tags.is("crossing", "generated"),
            );
            f.set_property("modified", node.modified);
            f.set_property(
                "way_ids",
                node.way_ids.iter().map(|w| w.0).collect::<Vec<_>>(),
            );
            f.set_property(
                "problems",
                serde_json::to_value(&node.problems).map_err(err_to_js)?,
            );
            features.push(f);
        }
        serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = getWays)]
    pub fn get_ways(&self) -> Result<String, JsValue> {
        let mut features = Vec::new();
        let mut crossings = Vec::new();
        // TODO HashMap nondet order, but it matters less since we sort later
        for (id, way) in &self.derived_ways {
            let mut f = self.mercator.to_wgs84_gj(&way.linestring);
            f.set_property("id", id.0);
            f.set_property("tags", serde_json::to_value(&way.tags).map_err(err_to_js)?);
            f.set_property("kind", format!("{:?}", way.kind));
            f.set_property("modified", way.modified);
            f.set_property(
                "node_ids",
                way.node_ids.iter().map(|n| n.0).collect::<Vec<_>>(),
            );
            f.set_property("is_severance", way.is_severance());
            f.set_property("is_service", way.tags.is("highway", "service"));
            f.set_property(
                "problems",
                serde_json::to_value(&way.problems).map_err(err_to_js)?,
            );
            if way.kind == Kind::Crossing {
                crossings.push(f);
            } else {
                features.push(f);
            }
        }

        // Partially sort, putting crossings last
        features.extend(crossings);

        for (idx, f) in features.iter_mut().enumerate() {
            f.id = Some(geojson::feature::Id::Number(idx.into()));
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

    #[wasm_bindgen(js_name = getRoadSides)]
    pub fn get_road_sides(&self) -> Result<String, JsValue> {
        let mut features = Vec::new();

        let offset_distance = 3.0;

        for way in self.derived_ways.values() {
            if !matches!(way.kind, Kind::RoadWithSeparate | Kind::RoadWithTags) {
                continue;
            }

            if let Some(ls) = way.linestring.offset_curve(offset_distance) {
                let mut f = self.mercator.to_wgs84_gj(&ls);
                let sidewalks = if way.tags.is_any("sidewalk:right", vec!["yes", "separate"])
                    || way
                        .tags
                        .is_any("sidewalk", vec!["both", "separate", "right"])
                    || way.tags.is("sidewalk:both", "separate")
                {
                    "✓"
                } else if way.tags.is("sidewalk:right", "no") || way.tags.is("sidewalk", "left") {
                    "X"
                } else {
                    "?"
                };
                f.set_property("sidewalks", sidewalks);
                features.push(f);
            }

            if let Some(ls) = way.linestring.offset_curve(-offset_distance) {
                let mut f = self.mercator.to_wgs84_gj(&ls);
                let sidewalks = if way.tags.is_any("sidewalk:left", vec!["yes", "separate"])
                    || way
                        .tags
                        .is_any("sidewalk", vec!["both", "separate", "left"])
                    || way.tags.is("sidewalk:both", "separate")
                {
                    "✓"
                } else if way.tags.is("sidewalk:left", "no") || way.tags.is("sidewalk", "right") {
                    "X"
                } else {
                    "?"
                };
                f.set_property("sidewalks", sidewalks);
                features.push(f);
            }
        }

        Ok(serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = editMakeAllSidewalks)]
    pub fn edit_make_all_sidewalks(&mut self, only_severances: bool) -> Result<(), JsValue> {
        let mut edits = self.edits.take().unwrap();
        // Ignore failure?
        let _ = edits.apply_cmd(UserCmd::MakeAllSidewalks(only_severances), self);
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    #[wasm_bindgen(js_name = editConnectAllCrossings)]
    pub fn edit_connect_all_crossings(&mut self, include_crossing_no: bool) -> Result<(), JsValue> {
        let mut edits = self.edits.take().unwrap();
        // Ignore failure?
        let _ = edits.apply_cmd(UserCmd::ConnectAllCrossings(include_crossing_no), self);
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

    // TODO Unused now, was just for debug
    #[wasm_bindgen(js_name = editAddNewCrossing)]
    pub fn edit_add_new_crossing(&mut self, x: f64, y: f64) -> Result<(), JsValue> {
        let mut edits = self.edits.take().unwrap();
        let mut tags = Tags::empty();
        tags.insert("highway", "crossing");
        tags.insert("crossing", "traffic_signals");
        edits
            .apply_cmd(UserCmd::AddCrossings(vec![Point::new(x, y)], tags), self)
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

    #[wasm_bindgen(js_name = auditCrossings)]
    pub fn audit_crossings_wasm(&self, options: JsValue) -> Result<String, JsValue> {
        let options: crate::audit::Options = serde_wasm_bindgen::from_value(options)?;
        self.audit_crossings(options).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = editGenerateMissingCrossings)]
    pub fn edit_generate_missing_crossings_wasm(
        &mut self,
        options: JsValue,
    ) -> Result<(), JsValue> {
        let options: crate::audit::Options = serde_wasm_bindgen::from_value(options)?;
        self.generate_missing_crossings(options).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = findConnectedComponents)]
    pub fn find_connected_components_wasm(&self, filter: JsValue) -> Result<String, JsValue> {
        let graph = crate::graph::Graph::new(self);
        let filter: crate::export::NetworkFilter = serde_wasm_bindgen::from_value(filter)?;
        Ok(
            serde_json::to_string(&self.find_connected_components(&graph, &filter))
                .map_err(err_to_js)?,
        )
    }

    #[wasm_bindgen(js_name = exportNetwork)]
    pub fn export_network_wasm(&self, filter: JsValue) -> Result<String, JsValue> {
        let filter: crate::export::NetworkFilter = serde_wasm_bindgen::from_value(filter)?;
        self.export_network(filter).map_err(err_to_js)
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
        for kind in Kind::all() {
            metrics.total_length_meters.insert(kind, 0.0);
        }
        for way in model.derived_ways.values() {
            *metrics.total_length_meters.get_mut(&way.kind).unwrap() +=
                Euclidean.length(&way.linestring);
        }
        metrics
    }
}
