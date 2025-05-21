use geo::Point;
use geojson::GeoJson;
use wasm_bindgen::prelude::*;

use crate::{err_to_js, Speedwalk};

#[wasm_bindgen]
impl Speedwalk {
    #[wasm_bindgen(js_name = getSideRoads)]
    pub fn get_side_roads(&self, only_along_main: bool) -> Result<String, JsValue> {
        let mut features = Vec::new();

        for (_, way) in &self.derived_ways {
            if only_along_main && !way.is_main_road {
                continue;
            }

            for node_id in &way.node_ids {
                if self.derived_nodes[node_id].way_ids.len() > 1 {
                    features.push(
                        self.mercator
                            .to_wgs84_gj(&Point::from(self.derived_nodes[node_id].pt)),
                    );
                }
            }
        }

        serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)
    }
}
