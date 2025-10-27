use serde::Serialize;
use utils::Tags;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
pub enum Kind {
    /// A road with no mention of separate sidewalks
    Road,
    /// A road with some hint of separate sidewalks (maybe not consistent/complete)
    RoadWithSeparate,
    /// A separately mapped sidewalk
    Sidewalk,
    Crossing,
    /// A non-sidewalk footway, a cycleway, or something else
    Other,
}

impl Kind {
    pub fn classify(tags: &Tags) -> Self {
        if tags.is("highway", "footway") && tags.is("footway", "sidewalk") {
            return Self::Sidewalk;
        }

        // TODO Or crossing=*, or should we be strict?
        if tags.is("highway", "footway") && tags.is("footway", "crossing") {
            return Self::Crossing;
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

        if tags.has("sidewalk:both") || tags.has("sidewalk:left") || tags.has("sidewalk:right") {
            return Self::RoadWithSeparate;
        }

        Self::Road
    }
}
