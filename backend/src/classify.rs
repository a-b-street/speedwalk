use serde::Serialize;
use utils::Tags;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum Kind {
    /// A separately mapped sidewalk
    Sidewalk,
    /// A road with some hint of separate sidewalks (maybe not consistent/complete)
    RoadWithSeparate,
    /// A road with no mention of separate sidewalks
    Road,
    /// A non-sidewalk footway, a crossing, a cycleway, or something else
    Other,
}

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

        if tags.has("sidewalk:both") || tags.has("sidewalk:left") || tags.has("sidewalk:right") {
            return Self::RoadWithSeparate;
        }

        Self::Road
    }
}
