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
        NodeID(self.id_counter as i64)
    }

    fn new_way_id(&mut self) -> WayID {
        self.id_counter += 1;
        WayID(self.id_counter as i64)
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
                    // TODO Make a bunch of new nodes

                    let mut tags = Tags::empty();
                    tags.insert("highway", "footway");
                    tags.insert("footway", "sidewalk");
                    let id = self.new_way_id();
                    self.new_ways.insert(
                        id,
                        Way {
                            linestring,
                            tags,
                            kind: Kind::Sidewalk,
                        },
                    );
                }
            }
        }
    }
}
