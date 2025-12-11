use serde::Serialize;
use utils::Tags;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Kind {
    /// A road with some hint of separate sidewalks (maybe not consistent/complete)
    RoadWithSeparate,
    /// A road with tagged sidewalks on at least one side
    RoadWithTags,
    /// A road tagged as having no sidewalks
    RoadWithoutSidewalksExplicit,
    /// A road assumed to have no sidewalks
    RoadWithoutSidewalksImplicit,
    /// A road missing sidewalk info completely
    RoadUnknown,

    /// A separately mapped sidewalk
    Sidewalk,

    Crossing,

    /// A non-sidewalk footway, a cycleway, or something else
    Other,
}

impl Kind {
    pub fn classify(tags: &Tags) -> Self {
        // TODO Maybe count more combos as sidewalks.
        // https://github.com/a-b-street/speedwalk/issues/23
        if tags.is("highway", "footway") && tags.is("footway", "sidewalk") {
            return Self::Sidewalk;
        }

        // TODO Or crossing=*, or should we be strict?
        if tags.is("highway", "footway") && tags.is("footway", "crossing") {
            return Self::Crossing;
        }
        if tags.is("highway", "cycleway") && tags.is("cycleway", "crossing") {
            return Self::Crossing;
        }

        if tags.is_any(
            "highway",
            vec![
                "corridor",
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

        if tags.is_any("sidewalk", vec!["no", "none"])
            || tags.is_any("sidewalk:both", vec!["no", "none"])
        {
            return Self::RoadWithoutSidewalksExplicit;
        }

        // RoadWithSeparate doesn't mean both sides are consistently tagged as separate/no; there
        // could be a mix
        if tags.is("sidewalk:both", "separate")
            || tags.is("sidewalk:left", "separate")
            || tags.is("sidewalk:right", "separate")
            || tags.is("sidewalk", "separate")
        {
            return Self::RoadWithSeparate;
        }

        if tags.has("sidewalk")
            || tags.is_any("sidewalk:both", vec!["yes", "no"])
            || tags.is_any("sidewalk:left", vec!["yes", "no"])
            || tags.is_any("sidewalk:right", vec!["yes", "no"])
        {
            return Self::RoadWithTags;
        }

        if tags.is_any("highway", vec!["motorway", "motorway_link", "service"]) {
            return Self::RoadWithoutSidewalksImplicit;
        }

        Self::RoadUnknown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify() {
        let mut ok = true;
        for (input, expected) in [
            (vec!["sidewalk:both=yes"], Kind::RoadWithTags),
            (
                vec!["sidewealk:left=yes", "sidewalk:right=separate"],
                Kind::RoadWithSeparate,
            ),
            (
                vec!["sidewalk:left=no", "sidewalk:right=yes"],
                Kind::RoadWithTags,
            ),
            (vec!["sidewalk:left=no"], Kind::RoadWithTags),
            (vec!["sidewalk:left=yes"], Kind::RoadWithTags),
            // TODO Not sure about some of these: https://github.com/a-b-street/speedwalk/issues/23
            (vec!["highway=path", "footway=sidewalk"], Kind::Other),
            (vec!["highway=cycleway", "foot=yes"], Kind::Other),
        ] {
            let actual = Kind::classify(&Tags::new_from_pairs(&input));
            if actual != expected {
                println!("For {input:?}, expected {expected:?} but got {actual:?}\n");
                ok = false;
            }
        }

        if !ok {
            panic!("Some cases failed");
        }
    }
}
