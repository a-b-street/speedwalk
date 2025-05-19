use std::collections::HashMap;

use osm_reader::{NodeID, WayID};
use serde::Serialize;
use utils::Tags;

use crate::{classify::Quickfix, Kind, Node, Speedwalk, Way};

#[derive(Default)]
pub struct Edits {
    pub user_commands: Vec<UserCmd>,

    // Derived consequences below
    pub change_way_tags: HashMap<WayID, Vec<TagCmd>>,

    pub new_nodes: HashMap<NodeID, Node>,
    pub new_ways: HashMap<WayID, Way>,

    id_counter: usize,
}

#[derive(Clone, Copy, Serialize)]
pub enum UserCmd {
    ApplyQuickfix(WayID, Quickfix),
    MakeSidewalk(WayID, f64, f64),
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

    pub fn apply_cmd(&mut self, cmd: UserCmd, model: &Speedwalk) {
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
                }
            }
            UserCmd::MakeSidewalk(way, left_meters, right_meters) => {
                let (left, right) = model.make_sidewalk(way, left_meters, right_meters);

                let cmds = self.change_way_tags.entry(way).or_insert_with(Vec::new);
                cmds.push(TagCmd::Remove("sidewalk"));

                if left.is_some() && right.is_some() {
                    cmds.push(TagCmd::Set("sidewalk:both", "separate"));
                } else if left.is_some() {
                    cmds.push(TagCmd::Set("sidewalk:left", "separate"));
                    cmds.push(TagCmd::Set("sidewalk:right", "no"));
                } else if right.is_some() {
                    cmds.push(TagCmd::Set("sidewalk:left", "no"));
                    cmds.push(TagCmd::Set("sidewalk:right", "separate"));
                }

                for linestring in vec![left, right].into_iter().flatten() {
                    let mut node_ids = Vec::new();
                    for pt in linestring.coords() {
                        let id = self.new_node_id();
                        self.new_nodes.insert(
                            id,
                            Node {
                                pt: *pt,
                                tags: Tags::empty(),
                                version: 0,
                            },
                        );
                        node_ids.push(id);
                    }

                    let mut tags = Tags::empty();
                    tags.insert("highway", "footway");
                    tags.insert("footway", "sidewalk");
                    let id = self.new_way_id();
                    self.new_ways.insert(
                        id,
                        Way {
                            node_ids,
                            linestring,
                            tags,
                            kind: Kind::Sidewalk,
                            version: 0,
                        },
                    );
                }
            }
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
                out.push(format!(r#"      <tag k="{k}" v="{v}" />"#));
            }
            out.push("    </node>".to_string());
        }
        for (id, way) in &self.new_ways {
            out.push(format!(r#"    <way id="{}" version="{}">"#, id.0, way.version));
            for node in &way.node_ids {
                out.push(format!(r#"      <nd ref="{}" />"#, node.0));
            }
            for (k, v) in &way.tags.0 {
                out.push(format!(r#"      <tag k="{k}" v="{v}" />"#));
            }
            out.push("    </way>".to_string());
        }
        out.push("  </create>".to_string());

        out.push("  <modify>".to_string());
        for id in self.change_way_tags.keys() {
            let way = &model.derived_ways[id];

            out.push(format!(r#"    <way id="{}" version="{}">"#, id.0, way.version));
            for node in &way.node_ids {
                out.push(format!(r#"      <nd ref="{}" />"#, node.0));
            }
            for (k, v) in &way.tags.0 {
                out.push(format!(r#"      <tag k="{k}" v="{v}" />"#));
            }
            out.push("    </way>".to_string());
        }
        out.push("  </modify>".to_string());

        out.push("</osmChange>".to_string());

        out.join("\n")
    }
}

impl Speedwalk {
    // TODO Or do this as we apply each UserCmd?
    pub fn after_edit(&mut self) {
        self.derived_nodes = self.original_nodes.clone();
        self.derived_ways = self.original_ways.clone();

        let edits = self.edits.as_ref().unwrap();

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
        }

        for (id, node) in &edits.new_nodes {
            self.derived_nodes.insert(*id, node.clone());
        }
        for (id, way) in &edits.new_ways {
            self.derived_ways.insert(*id, way.clone());
        }
    }
}
