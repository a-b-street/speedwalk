use std::collections::HashMap;

use geo::buffer::{BufferStyle, LineJoin};
use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{
    Buffer, Coord, Distance, Euclidean, InterpolatableLine, Line, LineLocatePoint, LineString,
    MultiLineString, Point,
};
use osm_reader::WayID;
use rstar::{RTree, primitives::GeomWithData};
use utils::{OffsetCurve, Tags, aabb};

use crate::{Kind, Speedwalk, edits::CreateNewGeometry};

const BUFFER_DISTANCE: f64 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Side {
    Left,
    Right,
}

impl Speedwalk {
    pub fn make_all_sidewalks_v2(&self, only_severances: bool) -> CreateNewGeometry {
        let mut roads = Vec::new();
        let mut roads_with_ways = Vec::new();
        for (id, way) in &self.derived_ways {
            if matches!(way.kind, Kind::Sidewalk | Kind::Crossing) {
                continue;
            }
            if only_severances && !way.is_severance() {
                continue;
            }
            // There are already separate sidewalks here
            if way.tags.has("sidewalk:both")
                || way.tags.has("sidewalk:left")
                || way.tags.has("sidewalk:right")
            {
                continue;
            }
            // Even if the lack of sidewalks is tagged in the old style, skip
            if way.tags.is("sidewalk", "no") {
                continue;
            }
            // This is ambiguous, but generally seems to mean both
            if way.tags.is("sidewalk", "separate") {
                continue;
            }
            // Implies no sidewalks
            if way
                .tags
                .is_any("highway", vec!["motorway", "motorway_link"])
            {
                continue;
            }
            // Ignore things that don't exist
            if way.tags.is("highway", "proposed") {
                continue;
            }

            roads.push(way.linestring.clone());

            roads_with_ways.push(GeomWithData::new(way.linestring.clone(), *id));
        }
        let roads_mls = MultiLineString(roads);

        // Buffer these all in one batch; it's much cleaner
        info!("Creating one big buffered blob");
        let subtract_polygons = roads_mls.buffer_with_style(
            BufferStyle::new(BUFFER_DISTANCE).line_join(LineJoin::Round(BUFFER_DISTANCE)),
        );

        // Debugging
        #[cfg(target_arch = "wasm32")]
        if false {
            utils::download_string(
                &serde_json::to_string(&self.mercator.to_wgs84_gj(&subtract_polygons)).unwrap(),
                "buffered.geojson",
            )
            .unwrap();
        }

        let mut raw_new_sidewalks = Vec::new();
        for polygon in subtract_polygons {
            let (exterior, holes) = polygon.into_inner();
            raw_new_sidewalks.push(exterior);
            raw_new_sidewalks.extend(holes);
        }

        let mut new_tags = Tags::empty();
        new_tags.insert("highway", "footway");
        new_tags.insert("footway", "sidewalk");

        info!(
            "Splitting {} new sidewalks into smaller chunks aligned to roads",
            raw_new_sidewalks.len()
        );
        let closest_road = RTree::bulk_load(roads_with_ways);
        let mut new_sidewalks = Vec::new();
        for sidewalk in raw_new_sidewalks {
            for (ls, way, side) in split_new_sidewalks(sidewalk, &closest_road) {
                // If the road lacks sidewalks on this side, skip
                if self.derived_ways[&way].tags.is("sidewalk", "left") && side == Side::Right {
                    continue;
                }
                if self.derived_ways[&way].tags.is("sidewalk", "right") && side == Side::Left {
                    continue;
                }

                let mut tags = new_tags.clone();
                tags.insert("tmp:closest_way", way.0.to_string());
                tags.insert("tmp:side", format!("{side:?}"));
                new_sidewalks.push((ls, tags));
            }
        }

        info!(
            "Building rtree for {} existing ways",
            self.derived_ways.len()
        );
        let closest_way = RTree::bulk_load(
            self.derived_ways
                .iter()
                .map(|(id, way)| GeomWithData::new(way.linestring.clone(), *id))
                .collect(),
        );

        info!(
            "Finding existing roads these {} new sidewalks cross",
            new_sidewalks.len()
        );
        let mut modify_existing = HashMap::new();
        for (new_sidewalk, _) in &mut new_sidewalks {
            let bbox = aabb(new_sidewalk);
            for obj in closest_way.locate_in_envelope_intersecting(&bbox) {
                for (pt, idx1, idx2) in find_all_intersections(new_sidewalk, obj.geom()) {
                    // Modify the new sidewalk immediately
                    new_sidewalk.0.insert(idx1, pt);

                    // Remember to modify the existing way
                    modify_existing
                        .entry(obj.data)
                        .or_insert_with(Vec::new)
                        .push((pt, idx2));
                }
            }
        }

        CreateNewGeometry {
            new_objects: new_sidewalks,
            new_kind: Kind::Sidewalk,
            modify_existing,
        }
    }
}

/// Returns all points of intersection between ls1 and ls2, and the index to insert those points
/// into ls1 and ls2. Returns highest idx1's first, so inserting repeatedly is safe.
// TODO If the intersection is very close to an existing node on that way, ESPECIALLY a
// crossing, snap to it
fn find_all_intersections(ls1: &LineString, ls2: &LineString) -> Vec<(Coord, usize, usize)> {
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
    // Put idx1's highest first
    hits.reverse();
    hits
}

// For each point, find the closest road road that contributed to it. Chunk by that, including
// the guess on which side of the road.
fn split_new_sidewalks(
    full: LineString,
    rtree: &RTree<GeomWithData<LineString, WayID>>,
) -> Vec<(LineString, WayID, Side)> {
    let mut lines: Vec<(Line, WayID, Side)> = Vec::new();
    for line in full.lines() {
        let midpt = line.point_at_ratio_from_start(&Euclidean, 0.5);
        let road = rtree
            .nearest_neighbor(&midpt)
            .expect("no closest road to a new sidewalk line");
        let side = classify_side(midpt, road.geom());

        lines.push((line, road.data, side));
    }

    let mut output = Vec::new();
    for chunk in lines.chunk_by(|a, b| (a.1, a.2) == (b.1, b.2)) {
        // Combine the lines
        let mut pts = vec![chunk[0].0.start];
        let road = chunk[0].1;
        let side = chunk[0].2;
        for (line, _, _) in chunk {
            pts.push(line.end);
        }
        output.push((LineString::new(pts), road, side));
    }
    output
}

fn classify_side(pt: Point, ls: &LineString) -> Side {
    // TODO There's probably something much easier with Orient, but this works
    let left = ls.offset_curve(-BUFFER_DISTANCE).expect("offset failed");
    let right = ls.offset_curve(BUFFER_DISTANCE).expect("offset failed");
    if distance(&left, pt).expect("snap failed") <= distance(&right, pt).expect("snap failed") {
        Side::Left
    } else {
        Side::Right
    }
}

// TODO Upstream
fn distance(ls: &LineString, pt: Point) -> Option<f64> {
    let fraction = ls.line_locate_point(&pt)?;
    let snapped = ls.point_at_ratio_from_start(&Euclidean, fraction)?;
    Some(Euclidean.distance(pt, snapped))
}
