use geo::LineString;
use osm_reader::WayID;
use utils::OffsetCurve;

use crate::Speedwalk;

impl Speedwalk {
    pub fn make_sidewalk(
        &self,
        base: WayID,
        left_meters: f64,
        right_meters: f64,
    ) -> (Option<LineString>, Option<LineString>) {
        let mut left = None;
        if left_meters > 0.0 {
            left = self.derived_ways[&base]
                .linestring
                .offset_curve(-left_meters);
        }

        let mut right = None;
        if right_meters > 0.0 {
            right = self.derived_ways[&base]
                .linestring
                .offset_curve(right_meters);
        }

        (left, right)
    }
}
