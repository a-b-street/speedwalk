use serde::{Deserialize, Serialize};
use utils::Tags;

#[derive(Clone, Debug, PartialEq)]
pub enum Kind {
    /// A separately mapped sidewalk
    Sidewalk,
    /// A roadway with bronze-level tags indicating the separate sidewalks
    GoodRoadway,
    /// A roadway not meeting bronze, but with a likely quick-fix
    QuickfixRoadway(Quickfix),
    /// A roadway not meeting bronze, with a problem
    BadRoadway(Problem),
    /// Something else / irrelevant
    Other,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum Quickfix {
    OldSidewalkSeparate,
    OldSidewalkNo,
    OldSidewalkNone,

    SetOldSidewalkBoth,
    SetOldSidewalkLeft,
    SetOldSidewalkRight,
    SetOldSidewalkNo,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub enum Problem {
    DoubleTaggedLeftBoth,
    DoubleTaggedRightBoth,
    OldStyleSidewalk,
    MissingNewStyle,
}

// See
// https://wiki.openstreetmap.org/wiki/Draft:Foundation/Local_Chapters/United_States/Pedestrian_Working_Group/Guide
impl Kind {
    pub fn classify(tags: &Tags) -> Self {
        if tags.is("highway", "footway") && tags.is("footway", "sidewalk") {
            return Self::Sidewalk;
        }

        if tags.is_any(
            "highway",
            vec![
                "cycleway",
                "footway",
                "path",
                "pedestrian",
                "platform",
                "steps",
                "track",
            ],
        ) {
            return Self::Other;
        }

        let left = tags.is_any("sidewalk:left", vec!["separate", "no"]);
        let right = tags.is_any("sidewalk:right", vec!["separate", "no"]);
        let both = tags.is_any("sidewalk:both", vec!["separate", "no"]);

        if left && both {
            return Self::BadRoadway(Problem::DoubleTaggedLeftBoth);
        }
        if right && both {
            return Self::BadRoadway(Problem::DoubleTaggedRightBoth);
        }

        if let Some(sidewalk) = tags.get("sidewalk") {
            if !both && !left && !right {
                if sidewalk == "no" {
                    return Self::QuickfixRoadway(Quickfix::OldSidewalkNo);
                } else if sidewalk == "none" {
                    return Self::QuickfixRoadway(Quickfix::OldSidewalkNone);
                } else if sidewalk == "separate" {
                    return Self::QuickfixRoadway(Quickfix::OldSidewalkSeparate);
                }
            }

            return Self::BadRoadway(Problem::OldStyleSidewalk);
        }

        if tags.is("highway", "motorway") {
            // No sidewalks implied
            return Self::GoodRoadway;
        }
        if tags.is("highway", "service") {
            return Self::GoodRoadway;
        }

        if !both && !(left && right) {
            return Self::BadRoadway(Problem::MissingNewStyle);
        }

        Self::GoodRoadway
    }

    pub fn to_simple_string(&self) -> &'static str {
        match self {
            Kind::Sidewalk => "sidewalk",
            Kind::GoodRoadway => "good_roadway",
            Kind::QuickfixRoadway(_) => "quickfix_roadway",
            Kind::BadRoadway(Problem::OldStyleSidewalk) => "old_style_roadway",
            Kind::BadRoadway(_) => "bad_roadway",
            Kind::Other => "other",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify() {
        for (expected, input) in vec![
            (
                Kind::QuickfixRoadway(Quickfix::OldSidewalkNo),
                vec!["highway=motorway", "sidewalk=no"],
            ),
            (Kind::GoodRoadway, vec!["highway=motorway"]),
        ] {
            let got = Kind::classify(&tags(input.clone()));
            if got != expected {
                panic!("For tags {input:?}, got {got:?} but expected {expected:?}");
            }
        }
    }

    fn tags(kv: Vec<&str>) -> Tags {
        let mut tags = Tags::empty();
        for pair in kv {
            let parts = pair.split('=').collect::<Vec<_>>();
            tags.insert(parts[0], parts[1]);
        }
        tags
    }
}
