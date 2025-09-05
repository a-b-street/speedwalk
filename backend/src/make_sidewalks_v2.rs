use std::collections::HashMap;

use geo::buffer::{BufferStyle, LineJoin};
use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{BooleanOps, Buffer, Coord, LineString, MultiLineString};
use osm_reader::WayID;
use utils::split_polygon;

use crate::{Kind, Speedwalk};

pub struct NewSidewalkResults {
    pub new_sidewalks: Vec<LineString>,
    // Everywhere existing some new sidewalk crosses, find the index in the existing way where this
    // crossed point needs to be inserted
    pub modify_existing: HashMap<WayID, Vec<(Coord, usize)>>,
}

impl Speedwalk {
    pub fn make_all_sidewalks_v2(&self, _assume_both_for_missing: bool) -> NewSidewalkResults {
        let mut splitters = Vec::new();
        for way in self.derived_ways.values() {
            if way.kind == Kind::Sidewalk || way.kind == Kind::Other {
                continue;
            }
            if way.tags.is("highway", "service") {
                continue;
            }
            if !way.is_severance() {
                continue;
            }

            splitters.push(way.linestring.clone());
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

        info!(
            "Finding existing roads these {} new sidewalks cross",
            new_sidewalks.len()
        );
        let mut modify_existing = HashMap::new();
        // TODO Could rstar and prune for sure
        for (way_id, way) in &self.derived_ways {
            for new_sidewalk in &mut new_sidewalks {
                if let Some((pt, idx1, idx2)) =
                    find_single_intersection(new_sidewalk, &way.linestring)
                {
                    // Modify the new sidewalk immediately
                    new_sidewalk.0.insert(idx1, pt);

                    // Remember to modify the existing way
                    modify_existing
                        .entry(*way_id)
                        .or_insert_with(Vec::new)
                        .push((pt, idx2));
                }
            }
        }

        NewSidewalkResults {
            new_sidewalks,
            modify_existing,
        }
    }
}

/// Returns the only point of intersection between ls1 and ls2, and the index to insert that point
/// into ls1 and ls2. Bails if ls1 hits ls2 multiple times.
// TODO If the intersection is very close to an existing node on that way, ESPECIALLY a
// crossing, snap to it
fn find_single_intersection(ls1: &LineString, ls2: &LineString) -> Option<(Coord, usize, usize)> {
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
    if hits.len() != 1 {
        return None;
    }
    hits.pop()
}
