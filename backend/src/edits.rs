use std::collections::{BTreeMap, HashMap, HashSet};

use anyhow::Result;
use geo::{Coord, LineString, Point};
use osm_reader::{NodeID, WayID};
use serde::Serialize;
use utils::Tags;

use crate::{Kind, Node, Speedwalk, Way};

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

#[derive(Clone, Serialize)]
pub enum UserCmd {
    SetTags(WayID, Vec<(String, String)>),
    MakeAllSidewalks(bool),
    ConnectAllCrossings,
    AssumeTags(bool),
    AddCrossing(Point, Tags),
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
            UserCmd::SetTags(way, replace) => {
                let cmds = self.change_way_tags.entry(way).or_insert_with(Vec::new);
                // Clear old sidewalk tags first
                for (k, _) in &model.derived_ways[&way].tags.0 {
                    if k.starts_with("sidewalk") {
                        cmds.push(TagCmd::Remove(k.clone()));
                    }
                }

                for (k, v) in replace {
                    cmds.push(TagCmd::Set(k, v));
                }
            }
            UserCmd::MakeAllSidewalks(only_severances) => {
                let results = model.make_all_sidewalks(only_severances);
                self.create_new_geometry(results, model);
            }
            UserCmd::ConnectAllCrossings => {
                let results = model.connect_all_crossings();
                self.create_new_geometry(results, model);
            }
            UserCmd::AssumeTags(drive_on_left) => {
                for (id, way) in &model.derived_ways {
                    if way.is_severance()
                        && !way.tags.has("sidewalk:both")
                        && !way.tags.has("sidewalk:left")
                        && !way.tags.has("sidewalk:right")
                        && !way.tags.has("sidewalk")
                        && (way.tags.is("oneway", "yes") || way.tags.is("junction", "roundabout"))
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
            UserCmd::AddCrossing(pt_wgs84, tags) => {
                let pt = model.mercator.to_mercator(&pt_wgs84);

                let new_node_id = self.new_node_id();
                self.new_nodes.insert(
                    new_node_id,
                    Node {
                        pt: pt.into(),
                        tags,
                        version: 0,

                        // Calculate later
                        way_ids: Vec::new(),
                        modified: true,
                        problems: Vec::new(),
                    },
                );

                // Find the one road this crossing should be on
                let Some((way_id, idx)) = model.add_one_crossing(pt) else {
                    bail!("Couldn't find the road to insert the crossing");
                };

                let mut node_ids = model.derived_ways[&way_id].node_ids.clone();
                node_ids.insert(idx, new_node_id);
                self.change_way_nodes.insert(way_id, node_ids);
            }
        }
        Ok(())
    }

    fn create_new_geometry(&mut self, results: CreateNewGeometry, model: &Speedwalk) {
        // TODO Or use+modify new_nodes immediately or something?
        let mut node_mapping: HashMap<HashedPoint, NodeID> = HashMap::new();

        // Modify existing ways first
        for (way_id, mut insert_points) in results.modify_existing {
            // When there are multiple points, insert the highest indices first, so nothing
            // messes up
            insert_points.sort_by(|a, b| b.1.cmp(&a.1));

            let mut node_ids = model.derived_ways[&way_id].node_ids.clone();

            for (pt, idx) in insert_points {
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
                        problems: Vec::new(),
                    },
                );
                node_mapping.insert(HashedPoint::new(pt), node_id);

                node_ids.insert(idx, node_id);
            }

            self.change_way_nodes.insert(way_id, node_ids);
        }

        // Create new geometry
        for (linestring, new_tags) in results.new_objects {
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
    pub new_objects: Vec<(LineString, Tags)>,
    /// All of the new objects have the same Kind
    pub new_kind: Kind,
    // Everywhere existing some new object crosses, find the index in the existing way where this
    // crossed point needs to be inserted
    pub modify_existing: HashMap<WayID, Vec<(Coord, usize)>>,
}
