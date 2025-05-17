#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::sync::Once;
use std::time::Duration;

use anyhow::Result;
use geo::{BoundingRect, Coord, Euclidean, Length, LineString, MapCoordsInPlace, Point, Polygon};
use osm_reader::OsmID;
use serde::Serialize;
use utils::Tags;
use wasm_bindgen::prelude::*;

static START: Once = Once::new();

#[wasm_bindgen]
pub struct Speedwalk {
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

        Ok(Speedwalk {})
    }

    #[wasm_bindgen(js_name = getWays)]
    pub fn get_ways(&self) -> Result<String, JsValue> {
        let mut features = Vec::new();
        serde_json::to_string(&features).map_err(err_to_js)
    }
}

fn err_to_js<E: std::fmt::Display>(err: E) -> JsValue {
    JsValue::from_str(&err.to_string())
}
