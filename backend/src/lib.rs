#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::sync::Once;

use anyhow::Result;
use geo::{Coord, GeometryCollection, LineString};
use geojson::GeoJson;
use osm_reader::{Element, WayID};
use utils::{Mercator, Tags};
use wasm_bindgen::prelude::*;

static START: Once = Once::new();

#[wasm_bindgen]
pub struct Speedwalk {
    ways: HashMap<WayID, Way>,
    mercator: Mercator,
}

struct Way {
    linestring: LineString,
    tags: Tags,
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

    #[wasm_bindgen(js_name = getWays)]
    pub fn get_ways(&self) -> Result<String, JsValue> {
        let mut features = Vec::new();
        for (id, way) in &self.ways {
            let mut f = self.mercator.to_wgs84_gj(&way.linestring);
            f.set_property("id", id.to_string());
            f.set_property("tags", serde_json::to_value(&way.tags).map_err(err_to_js)?);
            features.push(f);
        }
        serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)
    }
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}

fn scrape_osm(input_bytes: &[u8]) -> Result<Speedwalk> {
    let mut node_mapping = HashMap::new();
    let mut ways = HashMap::new();
    osm_reader::parse(input_bytes, |elem| match elem {
        Element::Node { id, lon, lat, .. } => {
            node_mapping.insert(id, Coord { x: lon, y: lat });
        }
        Element::Way {
            id, node_ids, tags, ..
        } => {
            let tags: Tags = tags.into();
            if tags.has("highway") {
                let linestring =
                    LineString::new(node_ids.into_iter().map(|id| node_mapping[&id]).collect());
                ways.insert(id, Way { linestring, tags });
            }
        }
        Element::Relation { .. } => {}
        Element::Bounds { .. } => {}
    })?;

    let mercator = Mercator::from(GeometryCollection::from(
        ways.values()
            .map(|way| way.linestring.clone())
            .collect::<Vec<_>>(),
    ))
    .unwrap();
    for way in ways.values_mut() {
        mercator.to_mercator_in_place(&mut way.linestring);
    }
    info!("Found {} ways", ways.len());

    Ok(Speedwalk { ways, mercator })
}
