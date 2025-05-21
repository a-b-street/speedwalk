use std::collections::{HashMap, HashSet};

use anyhow::Result;
use geo::{Coord, GeometryCollection, LineString};
use osm_reader::Element;
use utils::{Mercator, Tags};

use crate::{Edits, Kind, Node, Speedwalk, Way};

pub fn scrape_osm(input_bytes: &[u8]) -> Result<Speedwalk> {
    let mut nodes = HashMap::new();
    let mut ways = HashMap::new();
    let mut used_nodes = HashSet::new();
    osm_reader::parse(input_bytes, |elem| match elem {
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
            let tags: Tags = tags.into();
            if tags.has("highway") && !tags.is("area", "yes") {
                // This sometimes happens from Overpass?
                let num = node_ids.len();
                node_ids.retain(|n| nodes.contains_key(n));
                if node_ids.len() != num {
                    warn!("{id} refers to nodes outside the imported area");
                    return;
                }

                let mut pts = Vec::new();
                for node in &node_ids {
                    used_nodes.insert(*node);
                    pts.push(nodes[node].pt);
                }
                let linestring = LineString::new(pts);
                let kind = Kind::classify(&tags);
                ways.insert(
                    id,
                    Way {
                        node_ids,
                        linestring,
                        tags,
                        kind,
                        version: version.expect("way missing version"),
                    },
                );
            }
        }
        Element::Relation { .. } => {}
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

    Ok(Speedwalk {
        original_nodes: nodes.clone(),
        original_ways: ways.clone(),
        mercator,

        edits: Some(Edits::default()),

        derived_nodes: nodes,
        derived_ways: ways,
    })
}
