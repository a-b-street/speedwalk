use std::collections::HashMap;

use osm_reader::WayID;

use crate::classify::Quickfix;

pub struct Edits {
    pub tags: HashMap<WayID, Vec<Cmd>>,
}

pub enum Cmd {
    Set(&'static str, &'static str),
    Remove(&'static str),
}

impl Edits {
    pub fn apply_quickfix(&mut self, way: WayID, quickfix: Quickfix) {
        let cmds = self.tags.entry(way).or_insert_with(Vec::new);
        match quickfix {
            Quickfix::OldSidewalkSeparate => {
                cmds.push(Cmd::Remove("sidewalk"));
                cmds.push(Cmd::Set("sidewalk:both", "separate"));
            }
            Quickfix::OldSidewalkNo | Quickfix::OldSidewalkNone => {
                cmds.push(Cmd::Remove("sidewalk"));
                cmds.push(Cmd::Set("sidewalk:both", "no"));
            }
        }
    }
}
