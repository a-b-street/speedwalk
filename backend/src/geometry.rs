use anyhow::{bail, Result};
use osm_reader::WayID;
use utils::OffsetCurve;

use crate::Speedwalk;

impl Speedwalk {
    pub fn make_sidewalk(&self, base: WayID, project_meters: f64) -> Result<String> {
        let Some(linestring) = self.ways[&base].linestring.offset_curve(project_meters) else {
            bail!("offset_curve failed");
        };

        let f = self.mercator.to_wgs84_gj(&linestring);
        Ok(serde_json::to_string(&f)?)
    }
}
