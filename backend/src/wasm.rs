use std::collections::{BTreeMap, HashMap};
use std::sync::Once;

use anyhow::Result;
use geo::{Euclidean, Length, Point, Polygon};
use geojson::{Feature, GeoJson, Geometry};
use osm_reader::{NodeID, WayID};
use serde::{Deserialize, Serialize};
use utils::{OffsetCurve, Tags};
use wasm_bindgen::prelude::*;

use crate::{Edits, Kind, Speedwalk, UserCmd};

static START: Once = Once::new();

#[derive(Deserialize)]
struct BatchPoint {
    lat: f64,
    lng: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchResolvedCrossingInput {
    start_way: i64,
    end_way: i64,
    start: BatchPoint,
    end: BatchPoint,
}

#[derive(Deserialize)]
struct BatchCrossingInput {
    start: BatchPoint,
    end: BatchPoint,
    #[serde(default)]
    tags: BTreeMap<String, String>,
    #[serde(default)]
    resolved: Option<BatchResolvedCrossingInput>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchDeletionInput {
    way_id: i64,
    node1: i64,
    node2: i64,
}

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
            f.set_property("is_manual_crossing", node.tags.is("crossing", "manual"));
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
            f.set_property("is_manual_crossing", way.tags.is("crossing", "manual"));
            f.set_property(
                "problems",
                serde_json::to_value(&way.problems).map_err(err_to_js)?,
            );
            f.set_property("length_m", Euclidean.length(&way.linestring));
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

    #[wasm_bindgen(js_name = normalizeSidewalkTags)]
    pub fn normalize_sidewalk_tags(&self, id: i64) -> Result<String, JsValue> {
        let tags = &self.derived_ways[&WayID(id)].tags;
        let normalized = normalize_sidewalk_tags(tags);
        let mut result = serde_json::Map::new();
        if let Some(left) = normalized.left {
            result.insert("left".to_string(), left.into());
        }
        if let Some(right) = normalized.right {
            result.insert("right".to_string(), right.into());
        }
        if let Some(both) = normalized.both {
            result.insert("both".to_string(), both.into());
        }
        Ok(serde_json::to_string(&result).map_err(err_to_js)?)
    }

    #[wasm_bindgen(js_name = getSideLocations)]
    pub fn get_side_locations(&self, id: i64) -> Result<String, JsValue> {
        let way = &self.derived_ways[&WayID(id)];
        let linestring = &way.linestring;
        let normalized = normalize_sidewalk_tags(&way.tags);
        let mut features = Vec::new();

        if let Some(ls) = linestring.offset_curve(20.0) {
            let mut f = self.mercator.to_wgs84_gj(&ls);
            f.set_property("side", "right");
            let right_label = normalized
                .both
                .as_ref()
                .or(normalized.right.as_ref())
                .map(|v| format!("right: {}", v))
                .unwrap_or_else(|| "right".to_string());
            f.set_property("label", right_label);
            features.push(f);
        }
        if let Some(ls) = linestring.offset_curve(-20.0) {
            let mut f = self.mercator.to_wgs84_gj(&ls);
            f.set_property("side", "left");
            let left_label = normalized
                .both
                .as_ref()
                .or(normalized.left.as_ref())
                .map(|v| format!("left: {}", v))
                .unwrap_or_else(|| "left".to_string());
            f.set_property("label", left_label);
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
    pub fn edit_set_tags(
        &mut self,
        base: i64,
        remove_keys: JsValue,
        add_tags: JsValue,
    ) -> Result<(), JsValue> {
        let remove_keys: Vec<String> = serde_wasm_bindgen::from_value(remove_keys)?;
        let add_tags: Vec<Vec<String>> = serde_wasm_bindgen::from_value(add_tags)?;
        let mut edits = self.edits.take().unwrap();
        edits
            .apply_cmd(
                UserCmd::SetTags {
                    way: WayID(base),
                    remove_keys,
                    add_tags: add_tags
                        .into_iter()
                        .map(|mut kv| (kv.remove(0), kv.remove(0)))
                        .collect(),
                },
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

    /// Add a crossing at (x, y) in WGS84 lon/lat with the given tags (e.g. highway=crossing, crossing=manual).
    #[wasm_bindgen(js_name = editAddNewCrossingWithTags)]
    pub fn edit_add_new_crossing_with_tags(
        &mut self,
        x: f64,
        y: f64,
        tags_js: JsValue,
    ) -> Result<(), JsValue> {
        let tag_map: HashMap<String, String> = serde_wasm_bindgen::from_value(tags_js)?;
        let mut tags = Tags::empty();
        for (k, v) in tag_map {
            tags.insert(&k, &v);
        }
        let mut edits = self.edits.take().unwrap();
        edits
            .apply_cmd(UserCmd::AddCrossings(vec![Point::new(x, y)], tags), self)
            .map_err(err_to_js)?;
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    /// Add a crossing as a segment between two points (WGS84 lon/lat). Each point is snapped to the
    /// nearest road or sidewalk (closest line); a new crossing way is created between the snapped points.
    #[wasm_bindgen(js_name = editAddCrossingSegment)]
    pub fn edit_add_crossing_segment(
        &mut self,
        start_lng: f64,
        start_lat: f64,
        end_lng: f64,
        end_lat: f64,
        tags_js: JsValue,
    ) -> Result<(), JsValue> {
        let tag_map: HashMap<String, String> = serde_wasm_bindgen::from_value(tags_js)?;
        let mut tags = Tags::empty();
        for (k, v) in tag_map {
            tags.insert(&k, &v);
        }
        let start = Point::new(start_lng, start_lat);
        let end = Point::new(end_lng, end_lat);
        let mut edits = self.edits.take().unwrap();
        edits
            .apply_cmd(UserCmd::AddCrossingSegment(start, end, tags), self)
            .map_err(err_to_js)?;
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    /// Snap two WGS84 points to the nearest road/sidewalk; returns snapped coords as JSON.
    /// Does not apply any edit. Used so the UI can store snapped geometry for re-apply/import.
    #[wasm_bindgen(js_name = snapCrossingSegment)]
    pub fn snap_crossing_segment_wasm(
        &self,
        start_lng: f64,
        start_lat: f64,
        end_lng: f64,
        end_lat: f64,
    ) -> Result<String, JsValue> {
        let start = Point::new(start_lng, start_lat);
        let end = Point::new(end_lng, end_lat);
        let resolved =
            crate::edits::resolve_crossing_segment(self, start, end).map_err(err_to_js)?;
        let out = serde_json::json!({
            "start": { "lat": resolved.start_lat, "lng": resolved.start_lng },
            "end": { "lat": resolved.end_lat, "lng": resolved.end_lng },
            "startWay": resolved.start_way,
            "endWay": resolved.end_way
        });
        serde_json::to_string(&out).map_err(err_to_js)
    }

    /// Resolve which graph edges lie between two draft points (same snap as crossings). JSON: `{ "edges": [...] }`.
    #[wasm_bindgen(js_name = resolveManualDeletion)]
    pub fn resolve_manual_deletion_wasm(
        &self,
        start_lng: f64,
        start_lat: f64,
        end_lng: f64,
        end_lat: f64,
    ) -> Result<String, JsValue> {
        let start = Point::new(start_lng, start_lat);
        let end = Point::new(end_lng, end_lat);
        let resolved =
            crate::edits::resolve_manual_deletion_edges(self, start, end).map_err(err_to_js)?;
        serde_json::to_string(&serde_json::json!({ "edges": resolved })).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = editManualDeleteEdge)]
    pub fn edit_manual_delete_edge(
        &mut self,
        way_id: i64,
        node1: i64,
        node2: i64,
    ) -> Result<(), JsValue> {
        let mut edits = self.edits.take().unwrap();
        edits
            .apply_cmd(
                UserCmd::ManualDeleteEdge {
                    way: WayID(way_id),
                    node1: NodeID(node1),
                    node2: NodeID(node2),
                },
                self,
            )
            .map_err(err_to_js)?;
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    /// Apply many manual overrides in one call. This batches commands and rebuilds derived state once.
    #[wasm_bindgen(js_name = editApplyManualOverridesBatch)]
    pub fn edit_apply_manual_overrides_batch(
        &mut self,
        crossings_js: JsValue,
        deletions_js: JsValue,
    ) -> Result<(), JsValue> {
        let crossings: Vec<BatchCrossingInput> = serde_wasm_bindgen::from_value(crossings_js)?;
        let deletions: Vec<BatchDeletionInput> = serde_wasm_bindgen::from_value(deletions_js)?;
        let mut cmds = Vec::with_capacity(crossings.len() + deletions.len());

        for crossing in crossings {
            let mut tags = Tags::empty();
            for (k, v) in crossing.tags {
                tags.insert(&k, &v);
            }
            if let Some(resolved) = crossing.resolved {
                cmds.push(UserCmd::AddCrossingSegmentSnapped {
                    start_way: WayID(resolved.start_way),
                    end_way: WayID(resolved.end_way),
                    snapped_start_wgs84: Point::new(resolved.start.lng, resolved.start.lat),
                    snapped_end_wgs84: Point::new(resolved.end.lng, resolved.end.lat),
                    tags,
                });
            } else {
                cmds.push(UserCmd::AddCrossingSegment(
                    Point::new(crossing.start.lng, crossing.start.lat),
                    Point::new(crossing.end.lng, crossing.end.lat),
                    tags,
                ));
            }
        }

        for deletion in deletions {
            cmds.push(UserCmd::ManualDeleteEdge {
                way: WayID(deletion.way_id),
                node1: NodeID(deletion.node1),
                node2: NodeID(deletion.node2),
            });
        }

        let mut edits = self.edits.take().unwrap();
        edits
            .apply_cmds_without_rebuild(cmds, self)
            .map_err(err_to_js)?;
        self.edits = Some(edits);
        self.after_edit();
        Ok(())
    }

    /// Clear manual override commands (manual crossing adds + manual edge deletions), preserving all
    /// other edit commands.
    ///
    /// Replays remaining commands one-by-one with `after_edit()` after each (same as [`edit_undo`]),
    /// so `apply_cmd` always sees a [`Speedwalk`] graph consistent with the command history.
    /// Batching with [`Edits::apply_cmds_without_rebuild`] against a stale derived graph can produce
    /// invalid `change_way_nodes` / `change_way_tags` keys and panic in [`Speedwalk::after_edit`].
    #[wasm_bindgen(js_name = editClearManualOverrides)]
    pub fn edit_clear_manual_overrides(&mut self) -> Result<(), JsValue> {
        let mut cmds = self.edits.take().unwrap().user_commands;
        cmds.retain(|cmd| {
            !matches!(
                cmd,
                UserCmd::AddCrossingSegment(_, _, _)
                    | UserCmd::AddCrossingSegmentSnapped { .. }
                    | UserCmd::ManualDeleteEdge { .. }
            )
        });

        self.edits = Some(Edits::default());
        self.after_edit();

        for cmd in cmds {
            let mut edits = self.edits.take().unwrap();
            edits.apply_cmd(cmd, self).map_err(err_to_js)?;
            self.edits = Some(edits);
            self.after_edit();
        }
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

#[derive(Default)]
struct NormalizedSidewalkTags {
    left: Option<String>,
    right: Option<String>,
    both: Option<String>,
}

fn normalize_sidewalk_tags(tags: &Tags) -> NormalizedSidewalkTags {
    let mut result = NormalizedSidewalkTags::default();

    // Check for sidewalk:both tag (highest priority)
    if let Some(both_value) = tags.get("sidewalk:both") {
        result.both = Some(both_value.clone());
    } else if let Some(sidewalk_value) = tags.get("sidewalk") {
        // Check for sidewalk tag (legacy format)
        match sidewalk_value.as_str() {
            "left" => {
                result.left = Some("yes".to_string());
                result.right = Some("no".to_string());
            }
            "right" => {
                result.left = Some("no".to_string());
                result.right = Some("yes".to_string());
            }
            "both" => {
                result.both = Some("yes".to_string());
            }
            "separate" => {
                result.both = Some("separate".to_string());
            }
            "no" | "none" => {
                result.both = Some("no".to_string());
            }
            _ => {}
        }
    }

    // Direct sidewalk:left and sidewalk:right tags override normalized values
    if let Some(left_value) = tags.get("sidewalk:left") {
        result.left = Some(left_value.clone());
    }
    if let Some(right_value) = tags.get("sidewalk:right") {
        result.right = Some(right_value.clone());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_sidewalk_tags_legacy_left_equivalence() {
        let legacy = Tags::new_from_pairs(&vec!["sidewalk=left"]);
        let explicit = Tags::new_from_pairs(&vec!["sidewalk:left=yes", "sidewalk:right=no"]);

        let legacy_norm = normalize_sidewalk_tags(&legacy);
        let explicit_norm = normalize_sidewalk_tags(&explicit);

        assert_eq!(legacy_norm.left.as_deref(), Some("yes"));
        assert_eq!(legacy_norm.right.as_deref(), Some("no"));
        assert_eq!(legacy_norm.both.as_deref(), None);

        assert_eq!(explicit_norm.left.as_deref(), Some("yes"));
        assert_eq!(explicit_norm.right.as_deref(), Some("no"));
        assert_eq!(explicit_norm.both.as_deref(), None);
    }

    #[test]
    fn test_normalize_sidewalk_tags_precedence() {
        let tags = Tags::new_from_pairs(&vec![
            "sidewalk=left",
            "sidewalk:both=separate",
            "sidewalk:right=no",
        ]);
        let norm = normalize_sidewalk_tags(&tags);

        // sidewalk:both has highest priority over legacy sidewalk=*
        assert_eq!(norm.both.as_deref(), Some("separate"));
        // explicit per-side tags override derived values
        assert_eq!(norm.right.as_deref(), Some("no"));
        assert_eq!(norm.left.as_deref(), None);
    }
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
