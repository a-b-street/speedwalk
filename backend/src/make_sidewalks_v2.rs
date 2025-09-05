use std::collections::HashMap;

use geo::buffer::{BufferStyle, LineJoin};
use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{BoundingRect, Buffer, Coord, LineString, MultiLineString, Point, Rect};
use osm_reader::WayID;
use rstar::{AABB, RTree, primitives::GeomWithData};

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

        let mut new_sidewalks = Vec::new();
        for polygon in subtract_polygons {
            let (exterior, holes) = polygon.into_inner();
            new_sidewalks.push(exterior);
            new_sidewalks.extend(holes);
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
        for new_sidewalk in &mut new_sidewalks {
            // TODO Since new_sidewalk usually covers a very big block, its bbox is huge. rstar
            // pruning will help much more if we can split the new linestring into meaningful
            // chunks first.
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

        NewSidewalkResults {
            new_sidewalks,
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

// TODO Upstream
fn aabb<G: BoundingRect<f64, Output = Option<Rect<f64>>>>(geom: &G) -> AABB<Point> {
    let bbox: Rect = geom.bounding_rect().unwrap().into();
    AABB::from_corners(
        Point::new(bbox.min().x, bbox.min().y),
        Point::new(bbox.max().x, bbox.max().y),
    )
}
