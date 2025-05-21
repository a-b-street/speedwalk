use anyhow::Result;
use geo::line_intersection::{line_intersection, LineIntersection};
use geo::{Coord, LineString};
use osm_reader::WayID;
use utils::OffsetCurve;

use crate::Speedwalk;

pub struct NewSidewalk {
    pub linestring: LineString,
    // Everywhere existing this linestring crosses, find the index where this crossed point needs
    // to be inserted
    pub crossing_points: Vec<(WayID, Coord, usize)>,
}

impl Speedwalk {
    pub fn make_sidewalk(
        &self,
        base: WayID,
        left_meters: f64,
        right_meters: f64,
    ) -> Result<(Option<NewSidewalk>, Option<NewSidewalk>)> {
        let mut left = None;
        if left_meters > 0.0 {
            if let Some(linestring) = self.derived_ways[&base]
                .linestring
                .offset_curve(-left_meters)
            {
                left = Some(NewSidewalk {
                    crossing_points: self.find_crossing_points(&linestring)?,
                    linestring,
                });
            }
        }

        let mut right = None;
        if right_meters > 0.0 {
            if let Some(linestring) = self.derived_ways[&base]
                .linestring
                .offset_curve(right_meters)
            {
                right = Some(NewSidewalk {
                    crossing_points: self.find_crossing_points(&linestring)?,
                    linestring,
                });
            }
        }

        Ok((left, right))
    }

    fn find_crossing_points(
        &self,
        new_sidewalk: &LineString,
    ) -> Result<Vec<(WayID, Coord, usize)>> {
        let mut crossings = Vec::new();
        // TODO Could rstar
        for (id, way) in &self.derived_ways {
            // TODO If the intersection is very close to an existing node on that way, ESPECIALLY a
            // crossing, snap to it?
            if let Some((pt, idx)) = find_single_intersection(new_sidewalk, &way.linestring)? {
                crossings.push((*id, pt, idx));
            }
        }
        Ok(crossings)
    }
}

/// Returns the only point of intersection between ls1 and ls2, and the index to insert that point
/// into ls2. Bails if ls1 hits ls2 multiple times.
fn find_single_intersection(ls1: &LineString, ls2: &LineString) -> Result<Option<(Coord, usize)>> {
    // TODO Consider https://docs.rs/geo/latest/geo/algorithm/sweep/struct.Intersections.html, but
    // handle the endpoint thing better
    let mut hits = Vec::new();
    for line1 in ls1.lines() {
        for (idx, line2) in ls2.lines().enumerate() {
            if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                line_intersection(line1, line2)
            {
                // TODO Does is_proper matter?
                // TODO Is insertion idx correct?
                hits.push((intersection, idx + 1));
            }
        }
    }
    if hits.len() > 1 {
        bail!("New sidewalk hits an existing way multiple times, not handled yet");
    }
    Ok(hits.pop())
}
