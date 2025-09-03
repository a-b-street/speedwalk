use std::collections::HashSet;

use anyhow::Result;
use geo::line_intersection::{LineIntersection, line_intersection};
use geo::{
    Closest, ClosestPoint, Coord, Distance, Euclidean, Length, LineLocatePoint, LineString, Point,
    Simplify,
};
use osm_reader::{NodeID, WayID};
use rstar::{RTree, primitives::GeomWithData};
use utils::{LineSplit, OffsetCurve};

use crate::{Kind, Side, Speedwalk};

pub struct NewSidewalk {
    pub linestring: LineString,
    // Everywhere existing this linestring crosses, find the index in the existing way where this
    // crossed point needs to be inserted
    pub crossing_points: Vec<(WayID, Coord, usize)>,
    pub connect_start_node: Option<NodeID>,
    pub connect_end_node: Option<NodeID>,
}

impl Speedwalk {
    pub fn make_sidewalks(
        &self,
        base: WayID,
        side: Side,
        offset_meters: f64,
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
        let offset = if side == Side::Left {
            -offset_meters
        } else {
            offset_meters
        };
        if offset == 0.0 {
            return Ok(Vec::new());
        }
        let Some(mut linestring) = self.derived_ways[&base].linestring.offset_curve(offset) else {
            return Ok(Vec::new());
        };

        // The original way might have excessive detail
        linestring = linestring.simplify(1.0);

        // TODO If we're trimming back, we don't need to modify the linestring, but I guess it
        // doesn't hurt
        let crossing_points = self.make_crossing_points(&mut linestring)?;

        lines_with_crossings.push((linestring, crossing_points));

        // TODO Plumb options, or rethink trim_back_from_crossings
        let stop_at_other_sidewalks = true;

        if stop_at_other_sidewalks {
            let mut split = Vec::new();
            for (linestring, crossing_points) in lines_with_crossings.drain(..) {
                split.extend(stop_at_sidewalks(self, linestring, crossing_points)?);
            }
            lines_with_crossings = split;
        } else if let Some(trim_meters) = trim_back_from_crossings {
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

        // TODO Check crossing_points of each side don't involve the same ways. If so, the indices will
        // be wrong
        // ... shouldnt be relevant now, but still seeing some kind of bug like this
        let mut ways_crossed = HashSet::new();
        for sidewalk in &results {
            for (way, _, _) in &sidewalk.crossing_points {
                if ways_crossed.contains(way) {
                    error!("One new sidewalk has multiple crossings of {way}");
                    return Ok(Vec::new());
                }
                ways_crossed.insert(*way);
            }
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
            } else {
                // For the very first and last point in new_sidewalk, see if it's VERY CLOSE to
                // something else
                if let Some((pt, idx)) =
                    find_almost_intersection(new_sidewalk.0[0], &way.linestring)
                {
                    crossings.push((*id, pt, idx));

                    // We shouldnt have to modify new_sidewalk's geometry...
                } else if let Some((pt, idx)) =
                    find_almost_intersection(*new_sidewalk.0.last().unwrap(), &way.linestring)
                {
                    crossings.push((*id, pt, idx));

                    // We shouldnt have to modify new_sidewalk's geometry...
                }
            }
        }
        Ok(crossings)
    }
}

fn find_almost_intersection(endpt: Coord, ls: &LineString) -> Option<(Coord, usize)> {
    match ls.closest_point(&endpt.into()) {
        Closest::Intersection(pt) | Closest::SinglePoint(pt) => {
            if Euclidean.distance(endpt.into(), pt) < 1.0 {
                // Find the line containing this point. Unfortunately have to be fuzzy.
                let (idx, _) = ls
                    .lines()
                    .enumerate()
                    .flat_map(|(idx, line)| {
                        match line.closest_point(&pt) {
                            Closest::Intersection(x) | Closest::SinglePoint(x) => {
                                // Normally this distance should be practically 0 for the line
                                // "containing" pt
                                Some((idx, Euclidean.distance(x, pt)))
                            }
                            Closest::Indeterminate => None,
                        }
                    })
                    .min_by_key(|(_, dist)| (*dist * 10e5) as usize)
                    .unwrap();
                Some((pt.into(), idx + 1))
            } else {
                None
            }
        }
        Closest::Indeterminate => None,
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

fn stop_at_sidewalks(
    model: &Speedwalk,
    input: LineString,
    input_crossing_points: Vec<(WayID, Coord, usize)>,
) -> Result<Vec<(LineString, Vec<(WayID, Coord, usize)>)>> {
    // Just trim the start and end if we cross another sidewalk
    let mut start = 0.0;
    let mut end = 1.0;

    let mut only_sidewalks = input_crossing_points.clone();
    only_sidewalks.retain(|(way, _, _)| model.derived_ways[way].tags.is("highway", "footway"));

    for candidate in vec![only_sidewalks.get(0), only_sidewalks.last()] {
        if let Some((_, pt, _)) = candidate {
            if let Some(fraction) = input.line_locate_point(&Point::from(*pt)) {
                // Depending which end of the linestring it's on, assume it's start or end. Ignore
                // in the middle -- do cross over it.
                if fraction <= 0.2 {
                    start = fraction;
                } else if fraction >= 0.8 {
                    end = fraction;
                }
            }
        }
    }

    if start == 0.0 && end == 1.0 {
        // Nothing to do
        return Ok(vec![(input, input_crossing_points)]);
    }

    let Some(mut trimmed) = input
        .line_split_twice(start, end)
        .and_then(|r| r.into_second())
    else {
        bail!("Couldn't split from {start} to {end}");
    };

    // Then just recalculate crossing points
    let crossing_points = model.make_crossing_points(&mut trimmed)?;
    Ok(vec![(trimmed, crossing_points)])
}
