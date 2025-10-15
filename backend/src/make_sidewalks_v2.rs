use std::collections::HashMap;

use geo::buffer::{BufferStyle, LineJoin};
use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{Buffer, Coord, Euclidean, InterpolatableLine, LineString, MultiLineString, Point};
use osm_reader::WayID;
use rstar::{RTree, primitives::GeomWithData};
use utils::{Tags, aabb};

use crate::{Kind, Speedwalk, edits::CreateNewGeometry};

impl Speedwalk {
    // TODO Plumb through options:
    // - retain disconnected islands
    // - all roads, not just severances
    pub fn make_all_sidewalks_v2(&self) -> CreateNewGeometry {
        let mut splitters = Vec::new();
        let mut splitters_with_ways = Vec::new();
        for (id, way) in &self.derived_ways {
            if way.kind == Kind::Sidewalk || way.kind == Kind::Other {
                continue;
            }
            if !way.is_severance() {
                continue;
            }
            // There are already separate sidewalks here
            if way.kind == Kind::GoodRoadway {
                continue;
            }
            // Even if the lack of sidewalks is tagged in the old style, skip
            if way.tags.is("sidewalk", "no") {
                continue;
            }

            splitters.push(way.linestring.clone());

            splitters_with_ways.push(GeomWithData::new(
                way.linestring.clone(),
                *id
            ));
        }
        let splitters_mls = MultiLineString(splitters);

        // Buffer these all in one batch; it's much cleaner
        info!("Creating one big buffered blob");
        let width = 3.0;
        let subtract_polygons = splitters_mls
            .buffer_with_style(BufferStyle::new(width).line_join(LineJoin::Round(width)));

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
        let closest_splitter = RTree::bulk_load(splitters_with_ways);
        let mut new_sidewalks = Vec::new();
        for sidewalk in raw_new_sidewalks {
            for (ls, way) in split_new_sidewalks(sidewalk, &closest_splitter) {
                let mut tags = new_tags.clone();
                tags.insert("tmp:closest_way", way.0.to_string());
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

// For each point, find the closest splitter road that contributed to it. Chunk by that.
fn split_new_sidewalks(
    full: LineString,
    rtree: &RTree<GeomWithData<LineString, WayID>>,
) -> Vec<(LineString, WayID)> {
    let mut output = Vec::new();
    let mut pts = Vec::new();
    for chunk in full.0.chunk_by(|a, b| {
        rtree
            .nearest_neighbor(&Point::from(*a))
            .map(|obj| &obj.data)
            == rtree
                .nearest_neighbor(&Point::from(*b))
                .map(|obj| &obj.data)
    }) {
        pts.extend(chunk);
        if pts.len() >= 2 {
            let last = *pts.last().unwrap();
            // To decide the closest road, use the midpoint of the entire line. Funny stuff happens
            // near the ends.
            let new_line = LineString::new(std::mem::take(&mut pts));
            let midpt = new_line.point_at_ratio_from_start(&Euclidean, 0.5).expect("new sidewalk has no midpt");
            let road = rtree.nearest_neighbor(&midpt).expect("no closest road to a new sidewalk").data;
            output.push((new_line, road));
            pts = vec![last];
        }
    }
    // If there's a last point, add it to the last line
    if pts.len() == 1 {
        output.last_mut().unwrap().0.0.push(pts[0]);
        // TODO But dedupe in the case there was just one?
    }
    output
}
