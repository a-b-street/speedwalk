use serde::Serialize;
use utils::Tags;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize)]
pub enum Kind {
    /// A separately mapped sidewalk
    Sidewalk,
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

        Self::Road
    }
}
