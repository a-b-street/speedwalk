use std::collections::{HashMap, HashSet};

use anyhow::Result;
use geo::{Coord, GeometryCollection, LineString, Polygon};
use osm_reader::{Element, OsmID};
use rstar::RTree;
use utils::{Mercator, Tags};

use crate::{Edits, Kind, Node, Speedwalk, Way};

pub fn scrape_osm(input_bytes: &[u8]) -> Result<Speedwalk> {
    let mut timestamp = None;
    let mut nodes = HashMap::new();
    let mut ways = HashMap::new();
    let mut used_nodes = HashSet::new();

    let mut possible_building_parts = HashMap::new();
    let mut buildings = Vec::new();

    osm_reader::parse(input_bytes, |elem| match elem {
        Element::Timestamp(ts) => {
            timestamp = Some(ts);
        }
        Element::Node {
            id,
            lon,
            lat,
            tags,
            version,
            ..
        } => {
            nodes.insert(
                id,
                Node {
                    pt: Coord { x: lon, y: lat },
                    tags: tags.into(),
                    // TODO Change API to be fallible
                    version: version.expect("node missing version"),

                    way_ids: Vec::new(),
                    modified: false,
                    problems: Vec::new(),
                },
            );
        }
        Element::Way {
            id,
            mut node_ids,
            tags,
            version,
            ..
        } => {
            let num = node_ids.len();
            node_ids.retain(|n| nodes.contains_key(n));
            if node_ids.len() != num {
                warn!("{id} refers to nodes outside the imported area");
                return;
            }

            let tags: Tags = tags.into();
            if tags.has("highway") && !tags.is("area", "yes") {
                let mut pts = Vec::new();
                for node_id in &node_ids {
                    used_nodes.insert(*node_id);

                    let node = nodes.get_mut(node_id).unwrap();
                    node.way_ids.push(id);

                    pts.push(node.pt);
                }

                let linestring = LineString::new(pts);
                let kind = Kind::classify(&tags);
                ways.insert(
                    id,
                    Way {
                        node_ids,
                        linestring,
                        tags,
                        version: version.expect("way missing version"),

                        kind,
                        modified: false,
                        problems: Vec::new(),
                    },
                );
            } else if tags.has("building") {
                buildings.push(Polygon::new(
                    LineString::new(node_ids.into_iter().map(|n| nodes[&n].pt).collect()),
                    Vec::new(),
                ));
            } else if node_ids[0] == *node_ids.last().unwrap() {
                possible_building_parts.insert(
                    id,
                    Polygon::new(
                        LineString::new(node_ids.into_iter().map(|n| nodes[&n].pt).collect()),
                        Vec::new(),
                    ),
                );
            }
        }
        Element::Relation { members, tags, .. } => {
            if tags.contains_key("building") {
                for (role, id) in members {
                    if role == "outer"
                        && let OsmID::Way(way) = id
                    {
                        if let Some(polygon) = possible_building_parts.remove(&way) {
                            buildings.push(polygon);
                        }
                    }
                }
            }
        }
        Element::Bounds { .. } => {}
    })?;

    nodes.retain(|id, _| used_nodes.contains(id));

    let mercator = Mercator::from(GeometryCollection::from(
        ways.values()
            .map(|way| way.linestring.clone())
            .collect::<Vec<_>>(),
    ))
    .unwrap();
    for node in nodes.values_mut() {
        node.pt = mercator.pt_to_mercator(node.pt);
    }
    for way in ways.values_mut() {
        mercator.to_mercator_in_place(&mut way.linestring);
    }
    info!("Found {} ways", ways.len());
    for polygon in &mut buildings {
        mercator.to_mercator_in_place(polygon);
    }

    let mut model = Speedwalk {
        original_nodes: nodes.clone(),
        original_ways: ways.clone(),
        mercator,
        timestamp,
        closest_building: RTree::bulk_load(buildings),

        edits: Some(Edits::default()),

        derived_nodes: nodes,
        derived_ways: ways,
    };
    model.recalculate_problems();
    Ok(model)
}
