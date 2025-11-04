use geo::{Euclidean, InterpolatableLine, Intersects, Length, LineLocatePoint, LineString, Point};
use geojson::Feature;
use osm_reader::WayID;
use rstar::{RTree, primitives::GeomWithData};
use utils::{LineSplit, aabb, buffer_aabb};

use crate::{Kind, Problem, Speedwalk};

impl Speedwalk {
    pub fn recalculate_problems(&mut self) {
        let mut problem_nodes = Vec::new();
        let mut problem_ways = Vec::new();

        // Look for footway=crossing ways that don't have crossing nodes on the roads
        for (_way_id, way) in &self.derived_ways {
            if !way.tags.is("footway", "crossing") {
                continue;
            }
            for node_id in &way.node_ids {
                let node = &self.derived_nodes[node_id];
                if node.is_crossing() {
                    continue;
                }
                if node.way_ids.iter().any(|other_way_id| {
                    !matches!(
                        self.derived_ways[other_way_id].kind,
                        Kind::Sidewalk | Kind::Other
                    )
                }) {
                    problem_nodes.push((*node_id, "missing crossing node", Vec::new()));
                }
            }
        }

        // Look for footways involving crossing nodes that aren't marked footway=crossing
        for (way_id, way) in &self.derived_ways {
            if !matches!(self.derived_ways[way_id].kind, Kind::Sidewalk | Kind::Other) {
                continue;
            }
            if let Some(crossing_node) = way.node_ids.iter().find(|n| {
                let node = &self.derived_nodes[n];
                // TODO This one is debatable
                // Only crossing nodes over severances -- it's normal for a sidewalk to have
                // side road crossings in the middle
                node.is_crossing()
                    && node
                        .way_ids
                        .iter()
                        .any(|w| self.derived_ways[w].is_severance())
            }) {
                problem_ways.push((
                    *way_id,
                    "missing footway=crossing",
                    vec![
                        self.mercator
                            .to_wgs84_gj(&Point::from(self.derived_nodes[crossing_node].pt)),
                    ],
                ));
            }
        }

        for (road, _sidewalk, details) in self.find_parallel_sidewalks() {
            problem_ways.push((
                road,
                "possible separate sidewalk near way without it tagged",
                details,
            ));
        }

        // Fill out problems
        for (id, note, details) in problem_nodes {
            self.derived_nodes
                .get_mut(&id)
                .unwrap()
                .problems
                .push(Problem {
                    note: note.to_string(),
                    details,
                });
        }
        for (id, note, details) in problem_ways {
            self.derived_ways
                .get_mut(&id)
                .unwrap()
                .problems
                .push(Problem {
                    note: note.to_string(),
                    details,
                });
        }
    }

    // Returns pairs of (road, nearby matching sidewalk, debug). Only tries to find one nearby sidewalk
    // per road.
    fn find_parallel_sidewalks(&self) -> Vec<(WayID, WayID, Vec<Feature>)> {
        let mut results = Vec::new();

        let closest_sidewalk = RTree::bulk_load(
            self.derived_ways
                .iter()
                .filter(|(_, way)| way.kind == Kind::Sidewalk)
                .map(|(id, way)| GeomWithData::new(way.linestring.clone(), *id))
                .collect(),
        );

        'ROAD: for (road_id, road) in &self.derived_ways {
            if road.kind != Kind::Road {
                continue;
            }
            for sidewalk in closest_sidewalk
                .locate_in_envelope_intersecting(&buffer_aabb(aabb(&road.linestring), 15.0))
            {
                'LINE: for sidewalk_line in sidewalk.geom().lines() {
                    // Slice the candidate road by this one line segment in the sidewalk
                    if let Some((a, b)) =
                        slice_lines_to_match(&sidewalk_line.into(), &road.linestring)
                    {
                        // The slices should be roughly parallel
                        let angle_diff = (angle_ls(&a) - angle_ls(&b)).abs();
                        if angle_diff > 30.0 {
                            continue 'LINE;
                        }

                        // No buildings between the midpoint of the two slices
                        let midpt_line = LineString::new(vec![
                            a.point_at_ratio_from_start(&Euclidean, 0.5).unwrap().into(),
                            b.point_at_ratio_from_start(&Euclidean, 0.5).unwrap().into(),
                        ]);

                        if self
                            .closest_building
                            .locate_in_envelope_intersecting(&aabb(&midpt_line))
                            .any(|polygon| polygon.intersects(&midpt_line))
                        {
                            continue 'LINE;
                        }

                        let details = vec![
                            self.mercator
                                .to_wgs84_gj(&self.derived_ways[&sidewalk.data].linestring),
                            self.mercator
                                .to_wgs84_gj(&midpt_line),
                        ];

                        results.push((*road_id, sidewalk.data, details));
                        continue 'ROAD;
                    }
                }
            }
        }

        results
    }
}

// TODO Diagram of example cases would help
fn slice_lines_to_match(
    source: &LineString,
    target: &LineString,
) -> Option<(LineString, LineString)> {
    if Euclidean.length(source) >= Euclidean.length(target) {
        let smaller_source = slice_line_to_match(source, target)?;
        return Some((smaller_source, target.clone()));
    }

    let smaller_target = slice_line_to_match(target, source)?;
    Some((source.clone(), smaller_target))
}

// Slice `a` to correspond to `b`, by finding the closest point along `a` matching `b`'s start and
// end point.
fn slice_line_to_match(a: &LineString, b: &LineString) -> Option<LineString> {
    let start = a.line_locate_point(&b.points().next().unwrap())?;
    let end = a.line_locate_point(&b.points().last().unwrap())?;
    // Note this uses a copy of an API that hasn't been merged into georust yet. It seems to work
    // fine in practice.
    a.line_split_twice(start, end)?.into_second()
}

// Angle in degrees from first to last point. Ignores the "direction" of the line; returns [0,
// 180].
// TODO Needs unit testing!
fn angle_ls(ls: &LineString) -> f64 {
    let pt1 = ls.coords().next().unwrap();
    let pt2 = ls.coords().last().unwrap();
    let a1 = (pt2.y - pt1.y).atan2(pt2.x - pt1.x).to_degrees();
    // Normalize to [0, 360]
    let a2 = if a1 < 0.0 { a1 + 360.0 } else { a1 };
    // Ignore direction
    if a2 > 180.0 { a2 - 180.0 } else { a2 }
}
