use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;
use geo::{Closest, ClosestPoint, Coord, Distance, Euclidean, Haversine, LineString, Point};
use osm_reader::{NodeID, WayID};
use rstar::{RTree, primitives::GeomWithData};
use serde::Serialize;
use utils::Tags;

use crate::{
    Kind, Node, Speedwalk, Way,
    graph::{Edge, Graph},
};

const MAX_CROSSING_SNAP_DISTANCE_METERS: f64 = 25.0;

struct SnappedCrossingSegment {
    start_way: WayID,
    end_way: WayID,
    snapped_start: Coord,
    snapped_end: Coord,
}

pub struct ResolvedCrossingSegment {
    pub start_way: i64,
    pub end_way: i64,
    pub start_lng: f64,
    pub start_lat: f64,
    pub end_lng: f64,
    pub end_lat: f64,
}

fn snap_crossing_segment_with_way_ids(
    model: &Speedwalk,
    start_wgs84: Point,
    end_wgs84: Point,
) -> Result<SnappedCrossingSegment> {
    let start_pt = model.mercator.to_mercator(&start_wgs84);
    let end_pt = model.mercator.to_mercator(&end_wgs84);
    let closest_line = RTree::bulk_load(
        model
            .derived_ways
            .iter()
            .filter(|(_, way)| way.is_snap_target_for_crossing())
            .map(|(id, way)| GeomWithData::new(way.linestring.clone(), *id))
            .collect(),
    );
    let snap_to_line = |pt_wgs84: Point, pt_mercator: Coord| -> Result<(WayID, Coord)> {
        let Some(obj) = closest_line.nearest_neighbor(&Point::from(pt_mercator)) else {
            bail!("Couldn't find a line to snap to");
        };
        let snapped = match obj.geom().closest_point(&Point::from(pt_mercator)) {
            Closest::Intersection(c) | Closest::SinglePoint(c) => c.into(),
            Closest::Indeterminate => bail!("Couldn't snap point to line"),
        };
        let snapped_wgs84 = Point::from(model.mercator.pt_to_wgs84(snapped));
        let dist_m = Haversine.distance(pt_wgs84, snapped_wgs84);
        if dist_m > MAX_CROSSING_SNAP_DISTANCE_METERS {
            bail!(
                "Point is {:.1}m from the nearest routable line (max {:.0}m). Move it closer to the intended road/footway.",
                dist_m,
                MAX_CROSSING_SNAP_DISTANCE_METERS
            );
        }
        Ok((obj.data, snapped))
    };
    let (start_way, snapped_start) = snap_to_line(start_wgs84, start_pt.into())?;
    let (end_way, snapped_end) = snap_to_line(end_wgs84, end_pt.into())?;

    let dist_m = Euclidean.distance(Point::from(snapped_start), Point::from(snapped_end));
    if dist_m < 0.5 {
        bail!(
            "Both points snapped to the same location (distance {:.1}m). Try clicking on the two crossing ways you want to connect.",
            dist_m
        );
    }
    Ok(SnappedCrossingSegment {
        start_way,
        end_way,
        snapped_start,
        snapped_end,
    })
}

/// Resolves crossing segment snap endpoints and way IDs in one pass.
pub fn resolve_crossing_segment(
    model: &Speedwalk,
    start_wgs84: Point,
    end_wgs84: Point,
) -> Result<ResolvedCrossingSegment> {
    let snapped = snap_crossing_segment_with_way_ids(model, start_wgs84, end_wgs84)?;
    let start_out = model.mercator.pt_to_wgs84(snapped.snapped_start);
    let end_out = model.mercator.pt_to_wgs84(snapped.snapped_end);
    Ok(ResolvedCrossingSegment {
        start_way: snapped.start_way.0,
        end_way: snapped.end_way.0,
        start_lng: start_out.x,
        start_lat: start_out.y,
        end_lng: end_out.x,
        end_lat: end_out.y,
    })
}

#[cfg(test)]
fn snap_crossing_segment(
    model: &Speedwalk,
    start_wgs84: Point,
    end_wgs84: Point,
) -> Result<(Point, Point)> {
    let resolved = resolve_crossing_segment(model, start_wgs84, end_wgs84)?;
    Ok((
        Point::new(resolved.start_lng, resolved.start_lat),
        Point::new(resolved.end_lng, resolved.end_lat),
    ))
}

/// Distance along `ls` from its first vertex to the closest point on the line to `pt` (Mercator).
fn distance_along_linestring(ls: &LineString, pt: Coord) -> f64 {
    let p = Point::from(pt);
    let mut distance_before = 0.0;
    let mut best = f64::INFINITY;
    let mut best_along = 0.0;
    for line in ls.lines() {
        let dx = line.end.x - line.start.x;
        let dy = line.end.y - line.start.y;
        let len_sq = dx * dx + dy * dy;
        if len_sq < 1e-24 {
            continue;
        }
        let t =
            (((p.x() - line.start.x) * dx + (p.y() - line.start.y) * dy) / len_sq).clamp(0.0, 1.0);
        let cp = Point::new(line.start.x + t * dx, line.start.y + t * dy);
        let dist = Euclidean.distance(&p, &cp);
        let seg_len = len_sq.sqrt();
        let d_along = distance_before + t * seg_len;
        if dist < best {
            best = dist;
            best_along = d_along;
        }
        distance_before += seg_len;
    }
    best_along
}

/// Resolved edges to hide from the exported network (same snap targets as manual crossings).
#[derive(Serialize)]
pub struct ResolvedDeletionEdge {
    pub way_id: i64,
    pub node1: i64,
    pub node2: i64,
    pub mid_lat: f64,
    pub mid_lng: f64,
    pub tags: BTreeMap<String, String>,
}

/// Snap like a crossing, then select every graph edge on that way whose span overlaps the interval
/// between the two snapped positions along the way.
pub fn resolve_manual_deletion_edges(
    model: &Speedwalk,
    start_wgs84: Point,
    end_wgs84: Point,
) -> Result<Vec<ResolvedDeletionEdge>> {
    let snapped = snap_crossing_segment_with_way_ids(model, start_wgs84, end_wgs84)?;
    if snapped.start_way != snapped.end_way {
        bail!(
            "Both draft points must snap to the same way. Move them onto one road or path segment."
        );
    }
    let way_id = snapped.start_way;
    let way = &model.derived_ways[&way_id];
    let ls = &way.linestring;

    let d_a = distance_along_linestring(ls, snapped.snapped_start);
    let d_b = distance_along_linestring(ls, snapped.snapped_end);
    let lo = d_a.min(d_b);
    let hi = d_a.max(d_b);

    let graph = Graph::new(model);
    let mut edges_on_way: Vec<&Edge> = graph
        .edges
        .values()
        .filter(|e| e.osm_way == way_id)
        .collect();
    edges_on_way.sort_by_key(|e| e.idx_of_node1);

    let mut out = Vec::new();
    for edge in edges_on_way {
        let n1 = distance_along_linestring(ls, model.derived_nodes[&edge.osm_node1].pt);
        let n2 = distance_along_linestring(ls, model.derived_nodes[&edge.osm_node2].pt);
        let e_lo = n1.min(n2);
        let e_hi = n1.max(n2);
        if e_hi >= lo && e_lo <= hi {
            let a = model.derived_nodes[&edge.osm_node1].pt;
            let b = model.derived_nodes[&edge.osm_node2].pt;
            let mid_mercator = Coord {
                x: (a.x + b.x) / 2.0,
                y: (a.y + b.y) / 2.0,
            };
            let mid_wgs = model.mercator.pt_to_wgs84(mid_mercator);
            out.push(ResolvedDeletionEdge {
                way_id: way_id.0,
                node1: edge.osm_node1.0,
                node2: edge.osm_node2.0,
                mid_lat: mid_wgs.y,
                mid_lng: mid_wgs.x,
                tags: way.tags.0.clone(),
            });
        }
    }

    if out.is_empty() {
        bail!(
            "No segment found between the snapped points. Try placing the draft closer to the road."
        );
    }
    Ok(out)
}

#[derive(Default)]
pub struct Edits {
    pub user_commands: Vec<UserCmd>,

    /// Graph edges (way + endpoint nodes) excluded from network export / routing views.
    pub manual_deleted_edges: HashSet<(WayID, NodeID, NodeID)>,

    // Derived consequences below
    // TODO Or maybe ditch TagCmd and the equivalent for inserting nodes somewhere
    change_way_tags: HashMap<WayID, Vec<TagCmd>>,
    change_way_nodes: HashMap<WayID, Vec<NodeID>>,

    new_nodes: HashMap<NodeID, Node>,
    new_ways: HashMap<WayID, Way>,

    id_counter: usize,
}

#[derive(Clone, Serialize)]
pub enum UserCmd {
    SetTags {
        way: WayID,
        remove_keys: Vec<String>,
        add_tags: Vec<(String, String)>,
    },
    MakeAllSidewalks(bool),
    ConnectAllCrossings(bool),
    AssumeTags(bool),
    AddCrossings(Vec<Point>, Tags),
    /// Add a crossing as a segment between two points; each point is snapped to the nearest road or sidewalk (closest line).
    AddCrossingSegment(Point, Point, Tags),
    /// Add a crossing from previously resolved snapped points and target ways.
    AddCrossingSegmentSnapped {
        start_way: WayID,
        end_way: WayID,
        snapped_start_wgs84: Point,
        snapped_end_wgs84: Point,
        tags: Tags,
    },
    /// Exclude one graph edge (same node orientation as [`Edge`]).
    ManualDeleteEdge {
        way: WayID,
        node1: NodeID,
        node2: NodeID,
    },
}

pub enum TagCmd {
    Set(String, String),
    Remove(String),
}

impl Edits {
    fn new_node_id(&mut self) -> NodeID {
        self.id_counter += 1;
        NodeID(-1 * (self.id_counter as i64))
    }

    fn new_way_id(&mut self) -> WayID {
        self.id_counter += 1;
        WayID(-1 * (self.id_counter as i64))
    }

    pub fn apply_cmd(&mut self, cmd: UserCmd, model: &Speedwalk) -> Result<()> {
        self.user_commands.push(cmd.clone());
        match cmd {
            UserCmd::SetTags {
                way,
                remove_keys,
                add_tags,
            } => {
                let cmds = self.change_way_tags.entry(way).or_insert_with(Vec::new);
                // First remove all tags in the removal list
                for key in remove_keys {
                    cmds.push(TagCmd::Remove(key));
                }
                // Then add/set all tags in the addition list
                for (k, v) in add_tags {
                    cmds.push(TagCmd::Set(k, v));
                }
            }
            UserCmd::MakeAllSidewalks(only_severances) => {
                let results = model.make_all_sidewalks(only_severances);
                self.create_new_geometry(results, model);
            }
            UserCmd::ConnectAllCrossings(include_crossing_no) => {
                let results = model.connect_all_crossings(include_crossing_no);
                self.create_new_geometry(results, model);
            }
            UserCmd::AssumeTags(drive_on_left) => {
                for (id, way) in &model.derived_ways {
                    if way.is_severance()
                        && !way.tags.has("sidewalk:both")
                        && !way.tags.has("sidewalk:left")
                        && !way.tags.has("sidewalk:right")
                        && !way.tags.has("sidewalk")
                        && is_oneway(&way.tags)
                    {
                        let cmds = self.change_way_tags.entry(*id).or_insert_with(Vec::new);
                        cmds.push(TagCmd::Set(
                            "sidewalk".to_string(),
                            if drive_on_left {
                                "left".to_string()
                            } else {
                                "right".to_string()
                            },
                        ));
                    }
                }
            }
            UserCmd::AddCrossings(pts_wgs84, tags) => {
                info!(
                    "Building rtree for up to {} existing sidewalks",
                    model.derived_ways.len()
                );
                let closest_road = RTree::bulk_load(
                    model
                        .derived_ways
                        .iter()
                        // TODO and not Crossing or Other?
                        .filter(|(_, way)| way.kind != Kind::Sidewalk)
                        .map(|(id, way)| GeomWithData::new(way.linestring.clone(), *id))
                        .collect(),
                );

                let mut insert_new_nodes = HashMap::new();
                for pt in pts_wgs84 {
                    let pt = model.mercator.to_mercator(&pt);

                    // Find the one road this crossing should be on
                    let Some(obj) = closest_road.nearest_neighbor(&pt) else {
                        bail!("Couldn't find the road to insert the crossing");
                    };

                    insert_new_nodes
                        .entry(obj.data)
                        .or_insert_with(Vec::new)
                        .push((pt.into(), tags.clone()));
                }

                self.create_new_geometry(
                    CreateNewGeometry {
                        new_ways: Vec::new(),
                        // Unused
                        new_kind: Kind::Sidewalk,
                        insert_new_nodes,
                        modify_existing_way_tags: HashMap::new(),
                    },
                    model,
                );
            }
            UserCmd::AddCrossingSegment(start_wgs84, end_wgs84, way_tags) => {
                let snapped = snap_crossing_segment_with_way_ids(model, start_wgs84, end_wgs84)?;
                let node_tags = way_tags.clone();
                let mut insert_new_nodes: HashMap<WayID, Vec<(Coord, Tags)>> = HashMap::new();
                insert_new_nodes
                    .entry(snapped.start_way)
                    .or_default()
                    .push((snapped.snapped_start, node_tags.clone()));
                insert_new_nodes
                    .entry(snapped.end_way)
                    .or_default()
                    .push((snapped.snapped_end, node_tags));
                let crossing_way =
                    LineString::new(vec![snapped.snapped_start, snapped.snapped_end]);
                let new_ways = vec![(crossing_way, way_tags)];
                self.create_new_geometry(
                    CreateNewGeometry {
                        new_ways,
                        new_kind: Kind::Crossing,
                        insert_new_nodes,
                        modify_existing_way_tags: HashMap::new(),
                    },
                    model,
                );
            }
            UserCmd::AddCrossingSegmentSnapped {
                start_way,
                end_way,
                snapped_start_wgs84,
                snapped_end_wgs84,
                tags: way_tags,
            } => {
                let snapped_start = model.mercator.to_mercator(&snapped_start_wgs84);
                let snapped_end = model.mercator.to_mercator(&snapped_end_wgs84);
                let node_tags = way_tags.clone();
                let mut insert_new_nodes: HashMap<WayID, Vec<(Coord, Tags)>> = HashMap::new();
                insert_new_nodes
                    .entry(start_way)
                    .or_default()
                    .push((snapped_start.into(), node_tags.clone()));
                insert_new_nodes
                    .entry(end_way)
                    .or_default()
                    .push((snapped_end.into(), node_tags));
                let crossing_way = LineString::new(vec![snapped_start.into(), snapped_end.into()]);
                let new_ways = vec![(crossing_way, way_tags)];
                self.create_new_geometry(
                    CreateNewGeometry {
                        new_ways,
                        new_kind: Kind::Crossing,
                        insert_new_nodes,
                        modify_existing_way_tags: HashMap::new(),
                    },
                    model,
                );
            }
            UserCmd::ManualDeleteEdge { way, node1, node2 } => {
                self.manual_deleted_edges.insert((way, node1, node2));
            }
        }
        Ok(())
    }

    pub fn apply_cmds_without_rebuild(
        &mut self,
        cmds: Vec<UserCmd>,
        model: &Speedwalk,
    ) -> Result<()> {
        for cmd in cmds {
            self.apply_cmd(cmd, model)?;
        }
        Ok(())
    }

    fn create_new_geometry(&mut self, results: CreateNewGeometry, model: &Speedwalk) {
        // TODO Or use+modify new_nodes immediately or something?
        let mut node_mapping: HashMap<HashedPoint, NodeID> = HashMap::new();
        // Insert all existing nodes. When we create crossing ways from a crossing node, we don't
        // want to overwrite the crossing node.
        for (id, node) in &model.derived_nodes {
            node_mapping.insert(HashedPoint::new(node.pt), *id);
        }

        // Modify existing ways first
        for (way_id, insert_points) in results.insert_new_nodes {
            let mut node_ids = model.derived_ways[&way_id].node_ids.clone();
            let mut linestring = model.derived_ways[&way_id].linestring.clone();

            for (pt, tags) in insert_points {
                let node_id = self.new_node_id();
                self.new_nodes.insert(
                    node_id,
                    Node {
                        pt,
                        tags,
                        version: 0,

                        // Calculate later
                        way_ids: Vec::new(),
                        modified: true,
                        problems: Vec::new(),
                    },
                );
                node_mapping.insert(HashedPoint::new(pt), node_id);

                // Figure out where in this way to insert this node. insert_points could be in any
                // order.
                let Some((idx, _)) = linestring.lines().enumerate().min_by_key(|(_, line)| {
                    (Euclidean.distance(line, &Point::from(pt)) * 10e6) as usize
                }) else {
                    unreachable!("Couldn't find the line on a way to insert a node");
                };

                node_ids.insert(idx + 1, node_id);
                linestring.0.insert(idx + 1, pt);
            }

            self.change_way_nodes.insert(way_id, node_ids);
        }

        // Create new geometry
        for (linestring, new_tags) in results.new_ways {
            let mut node_ids = Vec::new();
            for pt in linestring.coords() {
                let id = node_mapping
                    .entry(HashedPoint::new(*pt))
                    .or_insert_with(|| {
                        let node_id = self.new_node_id();
                        self.new_nodes.insert(
                            node_id,
                            Node {
                                pt: *pt,
                                tags: Tags::empty(),
                                version: 0,

                                // Calculate later
                                way_ids: Vec::new(),
                                modified: true,
                                problems: Vec::new(),
                            },
                        );
                        node_id
                    });
                node_ids.push(*id);
            }

            let new_way_id = self.new_way_id();
            self.new_ways.insert(
                new_way_id,
                Way {
                    node_ids,
                    linestring,
                    tags: new_tags,
                    version: 0,

                    kind: results.new_kind.clone(),
                    modified: true,
                    problems: Vec::new(),
                },
            );
        }

        // Update tags
        for (way_id, cmds) in results.modify_existing_way_tags {
            self.change_way_tags
                .entry(way_id)
                .or_insert_with(Vec::new)
                .extend(cmds);
        }
    }

    pub fn to_osc(&self, model: &Speedwalk) -> String {
        let mut out = vec![r#"<osmChange version="0.6" generator="Speedwalk">"#.to_string()];

        out.push("  <create>".to_string());
        for (id, node) in &self.new_nodes {
            let pt = model.mercator.pt_to_wgs84(node.pt);
            out.push(format!(
                r#"    <node id="{}" version="{}" lon="{}" lat="{}">"#,
                id.0, node.version, pt.x, pt.y
            ));
            for (k, v) in &node.tags.0 {
                out.push(format!(r#"      <tag k="{k}" v="{}" />"#, escape(v)));
            }
            out.push("    </node>".to_string());
        }
        for (id, way) in &self.new_ways {
            out.push(format!(
                r#"    <way id="{}" version="{}">"#,
                id.0, way.version
            ));
            for node in &way.node_ids {
                out.push(format!(r#"      <nd ref="{}" />"#, node.0));
            }
            for (k, v) in &way.tags.0 {
                out.push(format!(r#"      <tag k="{k}" v="{}" />"#, escape(v)));
            }
            out.push("    </way>".to_string());
        }
        out.push("  </create>".to_string());

        out.push("  <modify>".to_string());
        for id in union_keys(&self.change_way_tags, &self.change_way_nodes) {
            let way = &model.derived_ways[&id];

            out.push(format!(
                r#"    <way id="{}" version="{}">"#,
                id.0, way.version
            ));
            for node in &way.node_ids {
                out.push(format!(r#"      <nd ref="{}" />"#, node.0));
            }
            for (k, v) in &way.tags.0 {
                out.push(format!(r#"      <tag k="{k}" v="{}" />"#, escape(v)));
            }
            out.push("    </way>".to_string());
        }
        out.push("  </modify>".to_string());

        out.push("</osmChange>".to_string());

        out.join("\n")
    }

    pub fn to_osmchange_json(&self, model: &Speedwalk) -> Result<String> {
        let mut out = OsmChange::default();

        for (id, node) in &self.new_nodes {
            let pt = model.mercator.pt_to_wgs84(node.pt);
            out.create.push(OsmElement {
                r#type: "node",
                id: id.0,
                tags: node.tags.0.clone(),
                version: node.version,

                lon: Some(pt.x),
                lat: Some(pt.y),
                nodes: Vec::new(),
            });
        }

        for (id, way) in &self.new_ways {
            out.create.push(OsmElement {
                r#type: "way",
                id: id.0,
                tags: way.tags.0.clone(),
                version: way.version,

                lon: None,
                lat: None,
                nodes: way.node_ids.iter().map(|n| n.0).collect(),
            });
        }

        for id in union_keys(&self.change_way_tags, &self.change_way_nodes) {
            let way = &model.derived_ways[&id];
            out.modify.push(OsmElement {
                r#type: "way",
                id: id.0,
                tags: way.tags.0.clone(),
                version: way.version,

                lon: None,
                lat: None,
                nodes: way.node_ids.iter().map(|n| n.0).collect(),
            });
        }

        Ok(serde_json::to_string(&out)?)
    }
}

impl Speedwalk {
    // TODO Or do this as we apply each UserCmd?
    pub fn after_edit(&mut self) {
        self.derived_nodes = self.original_nodes.clone();
        self.derived_ways = self.original_ways.clone();

        let edits = self.edits.as_ref().unwrap();

        // Order matters -- create new stuff, in case another command modifies it later
        for (id, node) in &edits.new_nodes {
            self.derived_nodes.insert(*id, node.clone());
        }
        for (id, way) in &edits.new_ways {
            self.derived_ways.insert(*id, way.clone());
        }

        for (way, cmds) in &edits.change_way_tags {
            let way = self.derived_ways.get_mut(way).unwrap();
            for cmd in cmds {
                match cmd {
                    TagCmd::Set(k, v) => {
                        way.tags.insert(k, v);
                    }
                    TagCmd::Remove(k) => {
                        way.tags.remove(k);
                    }
                }
            }
            way.kind = Kind::classify(&way.tags);
            way.modified = true;
        }
        for (way, node_ids) in &edits.change_way_nodes {
            let way = self.derived_ways.get_mut(way).unwrap();
            way.node_ids = node_ids.clone();
            way.modified = true;

            // Need to recalculate geometry!
            way.linestring = LineString::new(
                way.node_ids
                    .iter()
                    .map(|n| self.derived_nodes[n].pt)
                    .collect(),
            );

            // Don't mark the way's nodes as modified
        }

        // Recalculate the mapping from node to way_ids
        for node in self.derived_nodes.values_mut() {
            node.way_ids.clear();
        }
        for (way_id, way) in &self.derived_ways {
            for node_id in &way.node_ids {
                self.derived_nodes
                    .get_mut(node_id)
                    .unwrap()
                    .way_ids
                    .push(*way_id);
            }
        }
        // TODO Not entirely sure why this happens, but...
        for node in self.derived_nodes.values_mut() {
            node.way_ids.sort();
            node.way_ids.dedup();
        }

        self.recalculate_problems();
    }
}

#[derive(Default, Serialize)]
struct OsmChange {
    create: Vec<OsmElement>,
    modify: Vec<OsmElement>,
    delete: Vec<OsmElement>,
}

#[derive(Serialize)]
struct OsmElement {
    r#type: &'static str,
    id: i64,
    version: i32,
    tags: BTreeMap<String, String>,

    // For nodes
    #[serde(skip_serializing_if = "Option::is_none")]
    lon: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lat: Option<f64>,

    // For ways
    #[serde(skip_serializing_if = "Vec::is_empty")]
    nodes: Vec<i64>,
}

fn union_keys<K: Clone + std::cmp::Eq + std::hash::Hash, V1, V2>(
    x1: &HashMap<K, V1>,
    x2: &HashMap<K, V2>,
) -> HashSet<K> {
    let mut keys = HashSet::new();
    keys.extend(x1.keys().cloned());
    keys.extend(x2.keys().cloned());
    keys
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HashedPoint(isize, isize);

impl HashedPoint {
    fn new(pt: Coord) -> Self {
        Self((pt.x * 10_000.0) as isize, (pt.y * 10_000.0) as isize)
    }
}

// TODO Use a library, if there's a lightweight one
fn escape(v: &str) -> String {
    v.replace("\"", "&quot;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("&", "&amp;")
        .replace("'", "&apos;")
}

pub struct CreateNewGeometry {
    pub new_ways: Vec<(LineString, Tags)>,
    /// All of the new ways have the same Kind
    pub new_kind: Kind,
    /// Insert new nodes into an existing way
    pub insert_new_nodes: HashMap<WayID, Vec<(Coord, Tags)>>,
    pub modify_existing_way_tags: HashMap<WayID, Vec<TagCmd>>,
}

fn is_oneway(tags: &Tags) -> bool {
    if tags.is("oneway", "no") {
        return false;
    }
    tags.is("oneway", "yes") || tags.is_any("junction", vec!["circular", "roundabout"])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn model_from_osm(osm: &str) -> Speedwalk {
        Speedwalk::new_from_osm(osm.as_bytes(), None).unwrap()
    }

    #[test]
    fn snap_crossing_segment_allows_points_on_same_way() {
        let osm = r#"<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6" generator="test">
  <node id="1" lon="0.000000" lat="0.000000" version="1" />
  <node id="2" lon="0.000090" lat="0.000000" version="1" />
  <node id="3" lon="0.000180" lat="0.000000" version="1" />
  <way id="100" version="1">
    <nd ref="1"/><nd ref="2"/><nd ref="3"/>
    <tag k="highway" v="footway"/>
    <tag k="footway" v="sidewalk"/>
  </way>
</osm>"#;
        let model = model_from_osm(osm);
        let snapped = snap_crossing_segment(
            &model,
            Point::new(0.000020, 0.000010),
            Point::new(0.000160, -0.000010),
        )
        .unwrap();
        assert_ne!(snapped.0, snapped.1);
    }

    #[test]
    fn resolve_manual_deletion_selects_multiple_edges_between_points() {
        let osm = r#"<?xml version="1.0" encoding="UTF-8"?>
<osm version="0.6" generator="test">
  <node id="1" lon="0.000000" lat="0.000000" version="1" />
  <node id="2" lon="0.000090" lat="0.000000" version="1" />
  <node id="3" lon="0.000180" lat="0.000000" version="1" />
  <node id="4" lon="0.000270" lat="0.000000" version="1" />
  <node id="5" lon="0.000090" lat="0.000090" version="1" />
  <node id="6" lon="0.000180" lat="0.000090" version="1" />
  <way id="100" version="1">
    <nd ref="1"/><nd ref="2"/><nd ref="3"/><nd ref="4"/>
    <tag k="highway" v="residential"/>
  </way>
  <way id="200" version="1">
    <nd ref="2"/><nd ref="5"/>
    <tag k="highway" v="footway"/>
    <tag k="footway" v="sidewalk"/>
  </way>
  <way id="201" version="1">
    <nd ref="3"/><nd ref="6"/>
    <tag k="highway" v="footway"/>
    <tag k="footway" v="sidewalk"/>
  </way>
</osm>"#;
        let model = model_from_osm(osm);
        let resolved = resolve_manual_deletion_edges(
            &model,
            Point::new(0.000020, 0.000010),
            Point::new(0.000250, -0.000010),
        )
        .unwrap();

        let edges: HashSet<(i64, i64, i64)> = resolved
            .iter()
            .map(|e| (e.way_id, e.node1, e.node2))
            .collect();
        let has_edge =
            |a: i64, b: i64| edges.contains(&(100, a, b)) || edges.contains(&(100, b, a));

        assert_eq!(resolved.len(), 3, "A->B should cover exactly 3 edges");
        assert!(has_edge(1, 2));
        assert!(has_edge(2, 3));
        assert!(has_edge(3, 4));
    }
}
