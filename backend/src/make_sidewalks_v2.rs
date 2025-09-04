use geo::buffer::{BufferStyle, LineJoin};
use geo::{BooleanOps, Buffer, LineString, MultiLineString};
use utils::split_polygon;

use crate::{Kind, Speedwalk};

impl Speedwalk {
    pub fn make_all_sidewalks_v2(&self, _assume_both_for_missing: bool) -> Vec<LineString> {
        let mut splitters = Vec::new();
        for way in self.derived_ways.values() {
            if way.kind != Kind::Sidewalk && way.kind != Kind::Other {
                // TODO More granular, not every single road
                splitters.push(way.linestring.clone());
            }
        }
        let splitters_mls = MultiLineString(splitters);

        // Buffer these all in one batch; it's much cleaner
        info!("Creating one big buffered blob");
        let width = 3.0;
        let subtract_polygons = splitters_mls
            .buffer_with_style(BufferStyle::new(width).line_join(LineJoin::Round(width)));

        info!("Splitting {} edges into faces", splitters_mls.0.len());
        let study_area_bbox = self
            .mercator
            .to_mercator(&self.mercator.wgs84_bounds.to_polygon());
        let faces = split_polygon(&study_area_bbox, splitters_mls.iter());

        info!("Creating sidewalks for {} faces", faces.len());
        let mut new_sidewalks = Vec::new();
        for face in faces {
            // TODO Faster to find just the relevant edges and buffer repeatedly? Or prepare
            // subtract_polygons?
            let combo = face.difference(&subtract_polygons);
            for polygon in combo {
                new_sidewalks.push(polygon.into_inner().0);
            }
        }

        new_sidewalks
    }
}
