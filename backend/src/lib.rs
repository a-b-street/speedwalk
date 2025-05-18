#[macro_use]
extern crate log;

mod classify;
mod edits;
mod geometry;

use std::collections::{HashMap, HashSet};
use std::sync::Once;

use anyhow::Result;
use geo::{Coord, Euclidean, GeometryCollection, Length, LineString, Point};
use geojson::GeoJson;
use osm_reader::{Element, NodeID, WayID};
use serde::Serialize;
use utils::{Mercator, Tags};
use wasm_bindgen::prelude::*;

use crate::classify::Kind;

static START: Once = Once::new();

#[wasm_bindgen]
pub struct Speedwalk {
    nodes: HashMap<NodeID, Node>,
    ways: HashMap<WayID, Way>,
    mercator: Mercator,
}

struct Node {
    pt: Coord,
    tags: Tags,
}

pub struct Way {
    pub linestring: LineString,
    pub tags: Tags,
    pub kind: Kind,
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

        scrape_osm(input_bytes).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = getNodes)]
    pub fn get_nodes(&self) -> Result<String, JsValue> {
        let mut features = Vec::new();
        // TODO HashMap nondet order
        for (idx, (id, node)) in self.nodes.iter().enumerate() {
            let mut f = self.mercator.to_wgs84_gj(&Point::from(node.pt));
            f.id = Some(geojson::feature::Id::Number(idx.into()));
            f.set_property("id", id.0);
            f.set_property("tags", serde_json::to_value(&node.tags).map_err(err_to_js)?);
            features.push(f);
        }
        serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = getWays)]
    pub fn get_ways(&self) -> Result<String, JsValue> {
        let mut features = Vec::new();
        // TODO HashMap nondet order
        for (idx, (id, way)) in self.ways.iter().enumerate() {
            let mut f = self.mercator.to_wgs84_gj(&way.linestring);
            f.id = Some(geojson::feature::Id::Number(idx.into()));
            f.set_property("id", id.0);
            f.set_property("tags", serde_json::to_value(&way.tags).map_err(err_to_js)?);
            f.set_property("kind", way.kind.to_simple_string());
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
    ) -> Result<String, JsValue> {
        let (left, right) = self.make_sidewalk(WayID(base), left_meters, right_meters);
        let mut features = Vec::new();
        for x in vec![left, right].into_iter().flatten() {
            features.push(self.mercator.to_wgs84_gj(&x));
        }
        Ok(serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)?)
    }
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

fn scrape_osm(input_bytes: &[u8]) -> Result<Speedwalk> {
    let mut nodes = HashMap::new();
    let mut ways = HashMap::new();
    let mut used_nodes = HashSet::new();
    osm_reader::parse(input_bytes, |elem| match elem {
        Element::Node {
            id, lon, lat, tags, ..
        } => {
            nodes.insert(
                id,
                Node {
                    pt: Coord { x: lon, y: lat },
                    tags: tags.into(),
                },
            );
        }
        Element::Way {
            id, node_ids, tags, ..
        } => {
            let tags: Tags = tags.into();
            if tags.has("highway") {
                let mut pts = Vec::new();
                for node in node_ids {
                    used_nodes.insert(node);
                    pts.push(nodes[&node].pt);
                }
                let linestring = LineString::new(pts);
                let kind = Kind::classify(&tags);
                ways.insert(
                    id,
                    Way {
                        linestring,
                        tags,
                        kind,
                    },
                );
            }
        }
        Element::Relation { .. } => {}
        Element::Bounds { .. } => {}
    })?;

    nodes.retain(|id, _| used_nodes.contains(id));

    let mercator = Mercator::from(GeometryCollection::from(
        ways.values()
            .map(|way| way.linestring.clone())
            .collect::<Vec<_>>(),
    ))
    .unwrap();
    for node in nodes.values_mut() {
        node.pt = mercator.pt_to_mercator(node.pt);
    }
    for way in ways.values_mut() {
        mercator.to_mercator_in_place(&mut way.linestring);
    }
    info!("Found {} ways", ways.len());

    Ok(Speedwalk {
        nodes,
        ways,
        mercator,
    })
}

#[derive(Default, Serialize)]
struct Metrics {
    total_length_meters: HashMap<&'static str, f64>,
}

impl Metrics {
    fn new(model: &Speedwalk) -> Self {
        let mut metrics = Self::default();
        for way in model.ways.values() {
            *metrics
                .total_length_meters
                .entry(way.kind.to_simple_string())
                .or_insert(0.0) += Euclidean.length(&way.linestring);
        }
        metrics
    }
}
