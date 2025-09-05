use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;
use geo::{Coord, LineString};
use itertools::{Itertools, Position};
use osm_reader::{NodeID, WayID};
use serde::Serialize;
use utils::Tags;

use crate::{Kind, Node, Speedwalk, Way, classify::Quickfix};

#[derive(Default)]
pub struct Edits {
    pub user_commands: Vec<UserCmd>,

    // Derived consequences below
    // TODO Or maybe ditch TagCmd and the equivalent for inserting nodes somewhere
    change_way_tags: HashMap<WayID, Vec<TagCmd>>,
    change_way_nodes: HashMap<WayID, Vec<NodeID>>,

    new_nodes: HashMap<NodeID, Node>,
    new_ways: HashMap<WayID, Way>,

    id_counter: usize,
}

#[derive(Clone, Copy, PartialEq, Serialize)]
pub enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Serialize)]
pub enum UserCmd {
    ApplyQuickfix(WayID, Quickfix),
    MakeSidewalk(WayID, Side, f64, Option<f64>),
    ConnectCrossing(NodeID),
    SplitAtSideRoads(WayID),
    // assume both for missing
    MakeAllSidewalksV2(bool),
}

pub enum TagCmd {
    Set(&'static str, &'static str),
    Remove(&'static str),
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
        self.user_commands.push(cmd);
        match cmd {
            UserCmd::ApplyQuickfix(way, quickfix) => {
                let cmds = self.change_way_tags.entry(way).or_insert_with(Vec::new);
                match quickfix {
                    Quickfix::OldSidewalkSeparate => {
                        cmds.push(TagCmd::Remove("sidewalk"));
                        cmds.push(TagCmd::Set("sidewalk:both", "separate"));
                    }
                    Quickfix::OldSidewalkNo | Quickfix::OldSidewalkNone => {
                        cmds.push(TagCmd::Remove("sidewalk"));
                        cmds.push(TagCmd::Set("sidewalk:both", "no"));
                    }
                    Quickfix::SetOldSidewalkBoth => {
                        cmds.push(TagCmd::Set("sidewalk", "both"));
                    }
                    Quickfix::SetOldSidewalkLeft => {
                        cmds.push(TagCmd::Set("sidewalk", "left"));
                    }
                    Quickfix::SetOldSidewalkRight => {
                        cmds.push(TagCmd::Set("sidewalk", "right"));
                    }
                    Quickfix::SetOldSidewalkNo => {
                        cmds.push(TagCmd::Set("sidewalk", "no"));
                    }
                }
            }
            UserCmd::MakeSidewalk(way, side, offset_meters, trim_back_from_crossings) => {
                let sidewalks =
                    model.make_sidewalks(way, side, offset_meters, trim_back_from_crossings)?;

                // Update tags on the road
                {
                    let cmds = self.change_way_tags.entry(way).or_insert_with(Vec::new);
                    cmds.push(TagCmd::Remove("sidewalk"));

                    // TODO Wrong now
                    if side == Side::Left {
                        cmds.push(TagCmd::Set("sidewalk:left", "separate"));
                        cmds.push(TagCmd::Set("sidewalk:right", "no"));
                    } else {
                        cmds.push(TagCmd::Set("sidewalk:left", "no"));
                        cmds.push(TagCmd::Set("sidewalk:right", "separate"));
                    }
                }

                // TODO Check upfront for a weird bug and skip.
                let mut ok = true;
                for new_sidewalk in &sidewalks {
                    for (existing_way, _, idx) in &new_sidewalk.crossing_points {
                        let node_ids = self
                            .change_way_nodes
                            .get(&existing_way)
                            .unwrap_or_else(|| &model.derived_ways[existing_way].node_ids);
                        if *idx > node_ids.len() {
                            error!(
                                "Unknown bug, crossing_points for {existing_way} are very broken"
                            );
                            ok = false;
                        }
                    }
                }

                if !ok {
                    return Ok(());
                }

                for new_sidewalk in sidewalks {
                    // First insert new crossing nodes in the existing roads
                    let mut new_crossing_nodes: HashMap<HashedPoint, NodeID> = HashMap::new();
                    for (existing_way, pt, idx) in new_sidewalk.crossing_points {
                        let node_id = self.new_node_id();
                        let mut tags = Tags::empty();
                        // TODO Do this when we cross service roads or not?
                        // TODO This is wrong when a small new segment ends at a road (the new
                        // sidewalk segment probably shouldn't exist at all)
                        if !model.derived_ways[&existing_way]
                            .tags
                            .is("highway", "footway")
                        {
                            tags.insert("highway", "crossing");
                        }
                        self.new_nodes.insert(
                            node_id,
                            Node {
                                pt,
                                tags,
                                version: 0,

                                // Calculate later
                                way_ids: Vec::new(),
                                modified: true,
                            },
                        );
                        new_crossing_nodes.insert(HashedPoint::new(pt), node_id);

                        // If we're doing this twice in the same round, use change_way_nodes!
                        // TODO Still relevant?
                        let mut node_ids = self
                            .change_way_nodes
                            .get(&existing_way)
                            .cloned()
                            .unwrap_or_else(|| model.derived_ways[&existing_way].node_ids.clone());
                        node_ids.insert(idx, node_id);
                        self.change_way_nodes.insert(existing_way, node_ids);
                    }

                    // Now make the new sidewalk way
                    let mut node_ids = Vec::new();
                    let mut pts = Vec::new();
                    for (pos, pt) in new_sidewalk.linestring.coords().with_position() {
                        if pos == Position::First && new_sidewalk.connect_start_node.is_some() {
                            node_ids.push(new_sidewalk.connect_start_node.unwrap());
                        } else if pos == Position::Last && new_sidewalk.connect_end_node.is_some() {
                            node_ids.push(new_sidewalk.connect_end_node.unwrap());
                        } else if let Some(id) = new_crossing_nodes.get(&HashedPoint::new(*pt)) {
                            node_ids.push(*id);
                        } else {
                            let id = self.new_node_id();
                            self.new_nodes.insert(
                                id,
                                Node {
                                    pt: *pt,
                                    tags: Tags::empty(),
                                    version: 0,

                                    // Calculate later
                                    way_ids: Vec::new(),
                                    modified: true,
                                },
                            );
                            node_ids.push(id);
                        }

                        pts.push(*pt);
                    }

                    // TODO Not sure why this is happening
                    node_ids.dedup();

                    let mut tags = Tags::empty();
                    tags.insert("highway", "footway");
                    tags.insert("footway", "sidewalk");
                    let new_way_id = self.new_way_id();
                    self.new_ways.insert(
                        new_way_id,
                        Way {
                            node_ids,
                            linestring: new_sidewalk.linestring,
                            tags,
                            version: 0,

                            kind: Kind::Sidewalk,
                            is_main_road: false,
                            modified: true,
                        },
                    );
                }
            }
            UserCmd::SplitAtSideRoads(way) => {
                let changes = model.split_at_side_roads(way);
                for way_id in changes.delete_new_sidewalks {
                    info!("Split: delete {way_id}");
                    // It could be in either, or even both?!
                    self.new_ways.remove(&way_id);
                    self.change_way_nodes.remove(&way_id);
                }

                for node_ids in changes.create_new_sidewalks {
                    let mut tags = Tags::empty();
                    tags.insert("highway", "footway");
                    tags.insert("footway", "sidewalk");
                    let mut pts = Vec::new();
                    for node_id in &node_ids {
                        if let Some(node) = self.new_nodes.get(&node_id) {
                            pts.push(node.pt);
                        } else {
                            pts.push(model.derived_nodes[node_id].pt);
                        }
                    }
                    let new_way_id = self.new_way_id();
                    self.new_ways.insert(
                        new_way_id,
                        Way {
                            node_ids,
                            linestring: LineString::new(pts),
                            tags,
                            version: 0,

                            kind: Kind::Sidewalk,
                            is_main_road: false,
                            modified: true,
                        },
                    );
                }
            }
            UserCmd::ConnectCrossing(crossing_node) => {
                let (new_linestring, sidewalk1, insert_idx1, sidewalk2, insert_idx2) =
                    model.connect_crossing(crossing_node)?;

                // Make new nodes for the endpoints
                let new_node1 = self.new_node_id();
                self.new_nodes.insert(
                    new_node1,
                    Node {
                        pt: new_linestring.0[0],
                        tags: Tags::empty(),
                        version: 0,

                        // Calculate later
                        way_ids: Vec::new(),
                        modified: true,
                    },
                );

                let new_node2 = self.new_node_id();
                self.new_nodes.insert(
                    new_node2,
                    Node {
                        pt: *new_linestring.0.last().unwrap(),
                        tags: Tags::empty(),
                        version: 0,

                        // Calculate later
                        way_ids: Vec::new(),
                        modified: true,
                    },
                );

                // Split the existing sidewalks by these new nodes
                let mut node_ids1 = model.derived_ways[&sidewalk1].node_ids.clone();
                node_ids1.insert(insert_idx1, new_node1);
                self.change_way_nodes.insert(sidewalk1, node_ids1);

                let mut node_ids2 = model.derived_ways[&sidewalk2].node_ids.clone();
                node_ids2.insert(insert_idx2, new_node2);
                self.change_way_nodes.insert(sidewalk2, node_ids2);

                // Make the new way
                let mut tags = Tags::empty();
                tags.insert("highway", "footway");
                tags.insert("footway", "crossing");
                let new_way_id = self.new_way_id();
                self.new_ways.insert(
                    new_way_id,
                    Way {
                        node_ids: vec![new_node1, crossing_node, new_node2],
                        linestring: new_linestring,
                        tags,
                        version: 0,

                        kind: Kind::Other,
                        is_main_road: false,
                        modified: true,
                    },
                );
            }
            UserCmd::MakeAllSidewalksV2(assume_both_for_missing) => {
                let results = model.make_all_sidewalks_v2(assume_both_for_missing);

                // TODO Or use+modify new_nodes immediately or something?
                let mut node_mapping: HashMap<(isize, isize), NodeID> = HashMap::new();

                // Modify existing ways first
                for (way_id, insert_points) in results.modify_existing {
                    // TODO Handle this more carefully
                    if insert_points.len() > 1 {
                        continue;
                    }
                    let (pt, idx) = insert_points[0];

                    let node_id = self.new_node_id();
                    self.new_nodes.insert(
                        node_id,
                        Node {
                            pt,
                            tags: Tags::empty(),
                            version: 0,

                            // Calculate later
                            way_ids: Vec::new(),
                            modified: true,
                        },
                    );
                    node_mapping.insert(hashify_point(pt), node_id);

                    let mut node_ids = model.derived_ways[&way_id].node_ids.clone();
                    node_ids.insert(idx, node_id);
                    self.change_way_nodes.insert(way_id, node_ids);
                }

                // Create new geometry
                for linestring in results.new_sidewalks {
                    let mut node_ids = Vec::new();
                    for pt in linestring.coords() {
                        let id = node_mapping.entry(hashify_point(*pt)).or_insert_with(|| {
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
                                },
                            );
                            node_id
                        });
                        node_ids.push(*id);
                    }

                    let mut tags = Tags::empty();
                    tags.insert("highway", "footway");
                    tags.insert("footway", "sidewalk");
                    let new_way_id = self.new_way_id();
                    self.new_ways.insert(
                        new_way_id,
                        Way {
                            node_ids,
                            linestring,
                            tags,
                            version: 0,

                            kind: Kind::Sidewalk,
                            is_main_road: false,
                            modified: true,
                        },
                    );
                }
            }
        }
        Ok(())
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
                        way.tags.insert(*k, *v);
                    }
                    TagCmd::Remove(k) => {
                        way.tags.remove(*k);
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
        // cm precision
        Self((pt.x * 100.0) as isize, (pt.y * 100.0) as isize)
    }
}

fn escape(v: &str) -> String {
    v.replace("\"", "&quot;")
}

fn hashify_point(pt: Coord) -> (isize, isize) {
    ((pt.x * 10_000.0) as isize, (pt.y * 10_000.0) as isize)
}
