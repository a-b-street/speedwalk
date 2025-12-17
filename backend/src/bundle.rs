use anyhow::Result;
use geo::Contains;
use geojson::GeoJson;

use crate::{Kind, Speedwalk};

impl Speedwalk {
    pub fn bundle_faces(&self) -> Result<String> {
        let study_area = self.mercator.to_mercator(&self.boundary_wgs84);
        let faces = utils::split_polygon(
            &study_area,
            self.derived_ways
                .values()
                .filter(|way| matches!(way.kind, Kind::Sidewalk | Kind::Crossing))
                .map(|way| &way.linestring),
        );

        let mut features = Vec::new();
        for polygon in faces {
            // Ignore faces with buildings in them
            if self
                .closest_building
                .locate_in_envelope_intersecting(&utils::aabb(&polygon))
                .any(|building| polygon.contains(building))
            {
                continue;
            }

            features.push(self.mercator.to_wgs84_gj(&polygon));
        }
        Ok(serde_json::to_string(&GeoJson::from(features))?)
    }
}
