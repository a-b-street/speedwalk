use anyhow::Result;
use geo::line_intersection::{line_intersection, LineIntersection};
use geo::{Coord, LineString, Simplify};
use osm_reader::{NodeID, WayID};
use rstar::{primitives::GeomWithData, RTree};
use utils::OffsetCurve;

use crate::{Kind, Speedwalk};

pub struct NewSidewalk {
    pub linestring: LineString,
    // Everywhere existing this linestring crosses, find the index in the existing way where this
    // crossed point needs to be inserted
    pub crossing_points: Vec<(WayID, Coord, usize)>,
    pub connect_start_node: Option<NodeID>,
    pub connect_end_node: Option<NodeID>,
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
        // TODO Maintain this all the time
        let closest_sidewalk_endpoints = RTree::bulk_load(
            self.derived_ways
                .values()
                .flat_map(|w| {
                    if w.kind == Kind::Sidewalk {
                        let node1 = w.node_ids[0];
                        let node2 = *w.node_ids.last().unwrap();
                        vec![
                            GeomWithData::new(self.derived_nodes[&node1].pt, node1),
                            GeomWithData::new(self.derived_nodes[&node2].pt, node2),
                        ]
                    } else {
                        Vec::new()
                    }
                })
                .collect(),
        );

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

                    // Connect the start or end point to another sidewalk?
                    let mut connect_start_node = None;
                    if let Some((obj, dist)) = closest_sidewalk_endpoints
                        .nearest_neighbor_iter_with_distance_2(&linestring.0[0])
                        .next()
                    {
                        if dist <= 1.0 {
                            connect_start_node = Some(obj.data);
                            linestring.0[0] = *obj.geom();
                        }
                    }

                    let mut connect_end_node = None;
                    if let Some((obj, dist)) = closest_sidewalk_endpoints
                        .nearest_neighbor_iter_with_distance_2(linestring.0.last().unwrap())
                        .next()
                    {
                        if dist <= 1.0 {
                            connect_end_node = Some(obj.data);
                            linestring.0.pop();
                            linestring.0.push(*obj.geom());
                        }
                    }

                    *result = Some(NewSidewalk {
                        crossing_points,
                        linestring,
                        connect_start_node,
                        connect_end_node,
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
