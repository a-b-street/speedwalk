use anyhow::Result;
use geo::line_intersection::{line_intersection, LineIntersection};
use geo::{Coord, Distance, Euclidean, Line, LineString, Point};
use geojson::GeoJson;
use osm_reader::{NodeID, WayID};
use wasm_bindgen::prelude::*;

use crate::{err_to_js, Kind, Speedwalk};

#[wasm_bindgen]
impl Speedwalk {
    #[wasm_bindgen(js_name = getSideRoads)]
    pub fn get_side_roads(&self, only_along_main: bool) -> Result<String, JsValue> {
        let mut features = Vec::new();

        for (_, way) in &self.derived_ways {
            if only_along_main && !way.is_main_road {
                continue;
            }

            for node_id in &way.node_ids {
                if self.derived_nodes[node_id].way_ids.len() > 1 {
                    features.push(
                        self.mercator
                            .to_wgs84_gj(&Point::from(self.derived_nodes[node_id].pt)),
                    );
                }
            }
        }

        serde_json::to_string(&GeoJson::from(features)).map_err(err_to_js)
    }
}

impl Speedwalk {
    pub fn get_all_crossings_on_severances(&self) -> Vec<NodeID> {
        let mut crossings = Vec::new();
        for (id, node) in &self.derived_nodes {
            if node.is_crossing()
                && node
                    .way_ids
                    .iter()
                    .all(|way| self.derived_ways[way].is_severance())
            {
                crossings.push(*id);
            }
        }
        crossings
    }

    pub fn connect_crossing(&self, crossing_node: NodeID) -> Result<(LineString, WayID, WayID)> {
        let project_away_meters = 10.0;

        let crossing_node = &self.derived_nodes[&crossing_node];
        let crossing_pt = crossing_node.pt;

        // Make a perpendicular line at the node
        let severance_linestring = &self.derived_ways[&crossing_node.way_ids[0]].linestring;
        let angle = angle_of_pt_on_line(severance_linestring, crossing_pt);

        let test_line1 = Line::new(
            crossing_pt,
            project_away(crossing_pt, angle + 90.0, project_away_meters),
        );
        let Some((sidewalk1, endpt1)) = self.find_sidewalk_hit(test_line1) else {
            bail!("Couldn't find sidewalk on one side of crossing");
        };

        let test_line2 = Line::new(
            crossing_pt,
            project_away(crossing_pt, angle - 90.0, project_away_meters),
        );
        let Some((sidewalk2, endpt2)) = self.find_sidewalk_hit(test_line2) else {
            bail!("Couldn't find sidewalk on one side of crossing");
        };

        let new_linestring = LineString::new(vec![endpt1, crossing_pt, endpt2]);
        Ok((new_linestring, sidewalk1, sidewalk2))
    }

    fn find_sidewalk_hit(&self, line1: Line) -> Option<(WayID, Coord)> {
        // TODO rstar
        for (id, way) in &self.derived_ways {
            if way.kind == Kind::Sidewalk {
                for line2 in way.linestring.lines() {
                    if let Some(LineIntersection::SinglePoint { intersection, .. }) =
                        line_intersection(line1, line2)
                    {
                        return Some((*id, intersection));
                    }
                }
            }
        }
        None
    }
}

// TODO Use new geo euclidean destination
fn project_away(pt: Coord, angle_degrees: f64, distance: f64) -> Coord {
    let (sin, cos) = angle_degrees.to_radians().sin_cos();
    Coord {
        x: pt.x + distance * cos,
        y: pt.y + distance * sin,
    }
}

// TODO Move more of these to utils
fn angle_of_pt_on_line(linestring: &LineString, pt: Coord) -> f64 {
    let line = linestring
        .lines()
        .min_by_key(|line| (Euclidean.distance(line, pt) * 10e9) as usize)
        .unwrap();
    angle_of_line(line)
}

fn angle_of_line(line: Line) -> f64 {
    line.dy().atan2(line.dx()).to_degrees()
}
