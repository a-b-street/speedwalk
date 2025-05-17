#[macro_use]
extern crate log;

use std::collections::HashMap;
use std::sync::Once;

use anyhow::Result;
use geo::{Coord, Euclidean, GeometryCollection, Length, LineString};
use geojson::GeoJson;
use osm_reader::{Element, WayID};
use serde::Serialize;
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
    kind: Kind,
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
        // TODO HashMap nondet order
        for (idx, (id, way)) in self.ways.iter().enumerate() {
            let mut f = self.mercator.to_wgs84_gj(&way.linestring);
            f.id = Some(geojson::feature::Id::Number(idx.into()));
            f.set_property("id", id.0);
            f.set_property("tags", serde_json::to_value(&way.tags).map_err(err_to_js)?);
            f.set_property("kind", way.kind.to_simple_string());
            if let Kind::QuickfixRoadway(ref fix) = way.kind {
                f.set_property("fix", fix.to_string());
            }
            if let Kind::BadRoadway(ref problem) = way.kind {
                f.set_property("problem", problem.to_string());
            }
            features.push(f);
        }
        serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)
    }

    #[wasm_bindgen(js_name = getMetrics)]
    pub fn get_metrics(&self) -> Result<String, JsValue> {
        serde_json::to_string(&Metrics::new(self)).map_err(err_to_js)
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

enum Kind {
    /// A separately mapped sidewalk
    Sidewalk,
    /// A roadway with bronze-level tags indicating the separate sidewalks
    GoodRoadway,
    /// A roadway not meeting bronze, but with a likely quick-fix
    QuickfixRoadway(String),
    /// A roadway not meeting bronze, with a problem
    BadRoadway(String),
    /// Something else / irrelevant
    Other,
}

// See
// https://wiki.openstreetmap.org/wiki/Draft:Foundation/Local_Chapters/United_States/Pedestrian_Working_Group/Guide
impl Kind {
    fn classify(tags: &Tags) -> Self {
        if tags.is("highway", "footway") && tags.is("footway", "sidewalk") {
            return Self::Sidewalk;
        }

        if tags.is_any("highway", vec!["footway", "path", "steps"]) {
            return Self::Other;
        }

        let left = tags.is_any("sidewalk:left", vec!["separate", "no"]);
        let right = tags.is_any("sidewalk:right", vec!["separate", "no"]);
        let both = tags.is_any("sidewalk:both", vec!["separate", "no"]);

        if left && both {
            return Self::BadRoadway("Double-tagged: sidewalk:left and sidewalk:both".to_string());
        }
        if right && both {
            return Self::BadRoadway("Double-tagged: sidewalk:right and sidewalk:both".to_string());
        }

        if let Some(sidewalk) = tags.get("sidewalk") {
            if !both && !left && !right {
                if sidewalk == "no" || sidewalk == "none" {
                    return Self::QuickfixRoadway(format!(
                        "Replace sidewalk={sidewalk} with sidewalk:both=no"
                    ));
                }
                if sidewalk == "separate" {
                    return Self::QuickfixRoadway(
                        "Replace sidewalk=separate with sidewalk:both=separate".to_string(),
                    );
                }
            }

            return Self::BadRoadway("Old-style sidewalk tag included".to_string());
        }

        if tags.is("highway", "motorway") {
            // No sidewalks implied
            return Self::GoodRoadway;
        }
        if tags.is("highway", "service") && tags.is("service", "driveway") {
            return Self::GoodRoadway;
        }

        if !both && !(left && right) {
            return Self::BadRoadway("Both sides aren't specified".to_string());
        }

        Self::GoodRoadway
    }

    fn to_simple_string(&self) -> &'static str {
        match self {
            Kind::Sidewalk => "sidewalk",
            Kind::GoodRoadway => "good_roadway",
            Kind::QuickfixRoadway(_) => "quickfix_roadway",
            Kind::BadRoadway(_) => "bad_roadway",
            Kind::Other => "other",
        }
    }
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
