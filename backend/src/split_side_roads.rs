use osm_reader::{NodeID, WayID};

use crate::{Kind, Speedwalk};

// TODO Is this split between "figure out what to do" and "execute the change" useful?
pub struct Changes {
    pub delete_new_sidewalks: Vec<WayID>,
    // TODO Leave all the nodes alone, including the dummy new crossing node for now
    pub create_new_sidewalks: Vec<Vec<NodeID>>,
}

impl Speedwalk {
    pub fn split_at_side_roads(&self, way: WayID) -> Changes {
        let mut create_new_sidewalks = Vec::new();

        let mut nodes_so_far = Vec::new();
        let mut skip = 0;

        // Look for groups of (another NEW sidewalk, a road, another NEW sidewalk) with short edges
        for triple in self.derived_ways[&way].node_ids.windows(3) {
            if skip > 0 {
                skip -= 1;
                continue;
            }

            info!(
                "we got {:?}, {:?}, {:?} for {:?}, {:?}, {:?}",
                intersects(self, triple[0], way),
                intersects(self, triple[1], way),
                intersects(self, triple[2], way),
                triple[0],
                triple[1],
                triple[2]
            );

            if let (Intersects::NewSidewalk(s1), Intersects::Road, Intersects::NewSidewalk(s2)) = (
                intersects(self, triple[0], way),
                intersects(self, triple[1], way),
                intersects(self, triple[2], way),
            ) {
                // TODO Check length of these segments. Is it about 2x the distance used for
                // projecting new sidewalks?
                info!(
                    "chop {way} from {} to {} and from {} to {}",
                    triple[0], triple[1], triple[1], triple[2]
                );

                if !nodes_so_far.is_empty() {
                    nodes_so_far.push(triple[0]);
                    create_new_sidewalks.push(std::mem::take(&mut nodes_so_far));
                }

                nodes_so_far.push(triple[2]);
                skip = 2;
            } else {
                nodes_so_far.push(triple[0]);
            }
        }

        // Because of the windowing, we don't process the last 2 nodes
        {
            let all_nodes= &self.derived_ways[&way].node_ids;
            if all_nodes.len() >= 3 {
                let penultimate = all_nodes[all_nodes.len() - 2];
                let last = all_nodes[all_nodes.len() - 1];
                nodes_so_far.push(penultimate);
                nodes_so_far.push(last);
                nodes_so_far.dedup();
            }
        }

        if nodes_so_far.len() > 1 {
            create_new_sidewalks.push(std::mem::take(&mut nodes_so_far));
        }

        // If there's only one result, then no change happened!
        if create_new_sidewalks.len() == 1 {
            create_new_sidewalks.clear();
        }

        let mut delete_new_sidewalks = Vec::new();
        if !create_new_sidewalks.is_empty() {
            delete_new_sidewalks.push(way);
        }

        Changes {
            delete_new_sidewalks,
            create_new_sidewalks,
        }
    }
}

#[derive(Debug)]
enum Intersects {
    NewSidewalk(WayID),
    Road,
    Other,
}

fn intersects(model: &Speedwalk, at: NodeID, base_sidewalk: WayID) -> Intersects {
    let mut new_sidewalks = Vec::new();
    let mut roads = 0;

    for way in &model.derived_nodes[&at].way_ids {
        if *way == base_sidewalk {
            // Expected, ignore it
        } else if model.derived_ways[way].kind == Kind::Sidewalk && way.0 < 0 {
            new_sidewalks.push(*way);
        } else {
            roads += 1;
        }
    }

    //info!("at {at:?}, we have new_sidewalks {new_sidewalks:?} and {roads} roads");
    //info!("... raw is {:?}", model.derived_nodes[&at].way_ids);

    if new_sidewalks.len() == 1 && roads == 0 {
        Intersects::NewSidewalk(new_sidewalks[0])
    } else if new_sidewalks.is_empty() && roads > 0 {
        Intersects::Road
    } else {
        Intersects::Other
    }
}
