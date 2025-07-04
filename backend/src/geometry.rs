use anyhow::Result;
use geo::line_intersection::{line_intersection, LineIntersection};
use geo::{Coord, Euclidean, Length, LineLocatePoint, LineString, Point, Simplify};
use osm_reader::{NodeID, WayID};
use rstar::{primitives::GeomWithData, RTree};
use utils::{LineSplit, OffsetCurve};

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
    pub fn make_sidewalks(
        &self,
        base: WayID,
        left_meters: f64,
        right_meters: f64,
        trim_back_from_crossings: Option<f64>,
    ) -> Result<Vec<NewSidewalk>> {
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

        let mut lines_with_crossings = Vec::new();

        for offset in vec![-left_meters, right_meters] {
            if offset == 0.0 {
                continue;
            }
            let Some(mut linestring) = self.derived_ways[&base].linestring.offset_curve(offset)
            else {
                continue;
            };

            // The original way might have excessive detail
            linestring = linestring.simplify(&1.0);

            // TODO If we're trimming back, we don't need to modify the linestring, but I guess it
            // doesn't hurt
            let crossing_points = self.make_crossing_points(&mut linestring)?;

            lines_with_crossings.push((linestring, crossing_points));
        }

        if let Some(trim_meters) = trim_back_from_crossings {
            let mut split = Vec::new();
            for (linestring, crossing_points) in lines_with_crossings.drain(..) {
                split.extend(split_at_crossings(
                    self,
                    linestring,
                    crossing_points,
                    trim_meters,
                )?);
            }
            lines_with_crossings = split;
        }

        let mut results = Vec::new();
        for (mut linestring, crossing_points) in lines_with_crossings {
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

            results.push(NewSidewalk {
                crossing_points,
                linestring,
                connect_start_node,
                connect_end_node,
            });
        }

        Ok(results)
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
            if let Some((pt, idx1, idx2)) =
                find_single_intersection(*id, new_sidewalk, &way.linestring)?
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
    way: WayID,
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
        bail!("New sidewalk hits the existing {way} multiple times, not handled yet");
    }
    Ok(hits.pop())
}

fn split_at_crossings(
    model: &Speedwalk,
    input: LineString,
    input_crossing_points: Vec<(WayID, Coord, usize)>,
    trim_meters: f64,
) -> Result<Vec<(LineString, Vec<(WayID, Coord, usize)>)>> {
    // Split before and after every crossing
    let input_length = Euclidean.length(&input);
    let mut fractions = vec![0.0, 1.0];
    for (way, pt, _) in input_crossing_points {
        // Don't split when we cross other footways
        if model.derived_ways[&way].tags.is("highway", "footway") {
            continue;
        }

        if let Some(fraction) = input.line_locate_point(&Point::from(pt)) {
            let base_distance = input_length * fraction;

            let split1 = (base_distance - trim_meters) / input_length;
            let split2 = (base_distance + trim_meters) / input_length;

            if base_distance < trim_meters {
                // Override the first split
                fractions[0] = split2;
            } else if base_distance > input_length - trim_meters {
                // Override the last split
                fractions[1] = split1;
            } else {
                fractions.push(split1);
                fractions.push(split2);
            }
        }
    }
    fractions.sort_by_key(|f| (*f * 10e6) as usize);

    let mut all_split_lines = Vec::new();
    for ls in input.line_split_many(&fractions).unwrap_or_else(Vec::new) {
        all_split_lines.extend(ls);
    }
    // LineSplit forces 0 and 1; remove if needed
    if fractions[0] != 0.0 {
        all_split_lines.remove(0);
    }
    if *fractions.last().unwrap() != 1.0 {
        all_split_lines.pop();
    }

    // Every other line is a crossing way; drop it
    let mut sidewalk_lines = Vec::new();
    for (idx, line) in all_split_lines.into_iter().enumerate() {
        if idx % 2 == 0 {
            sidewalk_lines.push(line);
        }
    }

    // Then just recalculate crossing points for every split line, rather than try to maintain
    // anything
    let mut output = Vec::new();
    for mut linestring in sidewalk_lines {
        let crossing_points = model.make_crossing_points(&mut linestring)?;
        output.push((linestring, crossing_points));
    }
    Ok(output)
}
