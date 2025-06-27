use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;
use geo::{Coord, Euclidean, Length, LineString};
use itertools::{Itertools, Position};
use osm_reader::{NodeID, WayID};
use serde::Serialize;
use utils::Tags;

use crate::{classify::Quickfix, Kind, Node, Speedwalk, Way};

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

#[derive(Clone, Copy, Serialize)]
pub enum UserCmd {
    ApplyQuickfix(WayID, Quickfix),
    MakeSidewalk(WayID, f64, f64, Option<f64>),
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
            UserCmd::MakeSidewalk(way, left_meters, right_meters, trim_back_from_crossings) => {
                let sidewalks = model.make_sidewalks(
                    way,
                    left_meters,
                    right_meters,
                    trim_back_from_crossings,
                )?;

                // Update tags on the road
                {
                    let cmds = self.change_way_tags.entry(way).or_insert_with(Vec::new);
                    cmds.push(TagCmd::Remove("sidewalk"));

                    // TODO What if we failed to make a left/right sidewalk on just part of it?
                    // Need to split this way
                    if left_meters > 0. && right_meters > 0. {
                        cmds.push(TagCmd::Set("sidewalk:both", "separate"));
                    } else if left_meters > 0. {
                        cmds.push(TagCmd::Set("sidewalk:left", "separate"));
                        cmds.push(TagCmd::Set("sidewalk:right", "no"));
                    } else if right_meters > 0. {
                        cmds.push(TagCmd::Set("sidewalk:left", "no"));
                        cmds.push(TagCmd::Set("sidewalk:right", "separate"));
                    }
                }

                for new_sidewalk in sidewalks {
                    let new_way_id = self.new_way_id();

                    // First insert new crossing nodes in the existing roads
                    let mut new_crossing_nodes: HashMap<HashedPoint, NodeID> = HashMap::new();
                    let mut num_crossings = 0;
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
                            num_crossings += 1;
                        }
                        self.new_nodes.insert(
                            node_id,
                            Node {
                                pt,
                                tags,
                                version: 0,

                                way_ids: vec![existing_way, new_way_id],
                                modified: true,
                            },
                        );
                        new_crossing_nodes.insert(HashedPoint::new(pt), node_id);

                        let mut node_ids = model.derived_ways[&existing_way].node_ids.clone();
                        node_ids.insert(idx, node_id);
                        self.change_way_nodes.insert(existing_way, node_ids);
                    }

                    // Now make the new sidewalk way
                    let mut node_ids = Vec::new();
                    let mut distance_per_node = Vec::new();
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

                                    way_ids: vec![new_way_id],
                                    modified: true,
                                },
                            );
                            node_ids.push(id);
                        }

                        pts.push(*pt);
                        if pts.len() == 1 {
                            distance_per_node.push(0.0);
                        } else {
                            distance_per_node.push(Euclidean.length(&LineString::new(pts.clone())));
                        }
                    }

                    let mut tags = Tags::empty();
                    tags.insert("highway", "footway");
                    tags.insert("footway", "sidewalk");
                    self.new_ways.insert(
                        new_way_id,
                        Way {
                            node_ids,
                            linestring: new_sidewalk.linestring,
                            tags,
                            version: 0,

                            kind: Kind::Sidewalk,
                            num_crossings,
                            is_main_road: false,
                            distance_per_node,
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

            // Don't mark the way's nodes as modified

            // TODO Update Node.way_ids here?
        }

        // TODO Update num_crossings sometimes
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
