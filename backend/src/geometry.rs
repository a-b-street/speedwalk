use anyhow::Result;
use geo::line_intersection::{line_intersection, LineIntersection};
use geo::{Coord, LineString, Simplify};
use osm_reader::WayID;
use utils::OffsetCurve;

use crate::Speedwalk;

pub struct NewSidewalk {
    pub linestring: LineString,
    // Everywhere existing this linestring crosses, find the index in the existing way where this
    // crossed point needs to be inserted
    pub crossing_points: Vec<(WayID, Coord, usize)>,
}

impl Speedwalk {
    // TODO Check crossing_points of each side don't involve the same ways. If so, the indices will
    // be wrong
    pub fn make_sidewalk(
        &self,
        base: WayID,
        left_meters: f64,
        right_meters: f64,
    ) -> Result<(Option<NewSidewalk>, Option<NewSidewalk>)> {
        let mut left = None;
        let mut right = None;

        for (result, offset) in vec![(&mut left, -left_meters), (&mut right, right_meters)] {
            if offset != 0.0 {
                if let Some(mut linestring) =
                    self.derived_ways[&base].linestring.offset_curve(offset)
                {
                    // The original way might have excessive detail
                    linestring = linestring.simplify(&1.0);
                    let crossing_points = self.make_crossing_points(&mut linestring)?;

                    *result = Some(NewSidewalk {
                        crossing_points,
                        linestring,
                    });
                }
            }
        }

        Ok((left, right))
    }

    fn make_crossing_points(
        &self,
        new_sidewalk: &mut LineString,
    ) -> Result<Vec<(WayID, Coord, usize)>> {
        let mut crossings = Vec::new();
        // TODO Could rstar
        for (id, way) in &self.derived_ways {
            // TODO If the intersection is very close to an existing node on that way, ESPECIALLY a
            // crossing, snap to it?
            if let Some((pt, idx1, idx2)) = find_single_intersection(new_sidewalk, &way.linestring)?
            {
                crossings.push((*id, pt, idx2));

                // Modify the new_sidewalk geometry
                new_sidewalk.0.insert(idx1, pt);
            }
        }
        Ok(crossings)
    }
}

/// Returns the only point of intersection between ls1 and ls2, and the index to insert that point
/// into ls1 and ls2. Bails if ls1 hits ls2 multiple times.
fn find_single_intersection(
    ls1: &LineString,
    ls2: &LineString,
) -> Result<Option<(Coord, usize, usize)>> {
    // TODO Consider https://docs.rs/geo/latest/geo/algorithm/sweep/struct.Intersections.html, but
    // handle the endpoint thing better
    let mut hits = Vec::new();
    for (idx1, line1) in ls1.lines().enumerate() {
        for (idx2, line2) in ls2.lines().enumerate() {
            if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                line_intersection(line1, line2)
            {
                // TODO Does is_proper matter?
                hits.push((intersection, idx1 + 1, idx2 + 1));
            }
        }
    }
    if hits.len() > 1 {
        bail!("New sidewalk hits an existing way multiple times, not handled yet");
    }
    Ok(hits.pop())
}
