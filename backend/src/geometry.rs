use anyhow::{bail, Result};
use geojson::GeoJson;
use osm_reader::WayID;
use utils::OffsetCurve;

use crate::Speedwalk;

impl Speedwalk {
    pub fn make_sidewalk(
        &self,
        base: WayID,
        left_meters: f64,
        right_meters: f64,
    ) -> Result<String> {
        let mut features = Vec::new();

        if left_meters > 0.0 {
            let Some(linestring) = self.ways[&base].linestring.offset_curve(-left_meters) else {
                bail!("offset_curve failed");
            };
            features.push(self.mercator.to_wgs84_gj(&linestring));
        }

        if right_meters > 0.0 {
            let Some(linestring) = self.ways[&base].linestring.offset_curve(right_meters) else {
                bail!("offset_curve failed");
            };
            features.push(self.mercator.to_wgs84_gj(&linestring));
        }

        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }
}
