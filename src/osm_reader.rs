extern crate osmpbfreader;
use crate::osm_graph::Edge;
use crate::osm_graph::OSMGraph;
use log::info;
use osmpbfreader::objects::{NodeId, Way};
use osmpbfreader::OsmPbfReader;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

/// OSMReader is used to read an PBF file containing OSM data and build a graph from it
/// # Arguments
/// * `file_path` - The path of the PBF file
pub struct OSMReader {
    file_path: String,
}

/// OSMReader implementation
impl OSMReader {
    /// Create a new OSMReader
    /// # Arguments
    /// * `file_path` - The path of the PBF file
    pub fn new(file_path: String) -> Self {
        OSMReader {
            file_path: file_path,
        }
    }

    /// Build a graph from the PBF file
    /// # Returns
    /// * `OSMGraph` - The graph built from the PBF file
    pub async fn build_graph(&mut self) -> OSMGraph {
        let highway_to_keep = vec![
            "primary",
            "secondary",
            "tertiary",
            "unclassified",
            "residential",
            "primary_link",
            "secondary_link",
            "tertiary_link",
            "living_street",
        ];

        let file = File::open(&self.file_path).expect("Unable to open file");
        let reader = BufReader::new(file);
        let mut pbf_reader = OsmPbfReader::new(reader);

        let start_time = std::time::Instant::now();

        let mut osm_graph = OSMGraph::new();

        info!("Start reading file...");
        println!("Start reading file...");

        let objs = pbf_reader
            .get_objs_and_deps(|obj| {
                obj.is_way()
                    && obj.tags().contains_key("highway")
                    && highway_to_keep.contains(&obj.tags()["highway"].as_str())
            })
            .unwrap();

        info!("Read file in {} seconds", start_time.elapsed().as_secs());
        println!("Read file in {} seconds", start_time.elapsed().as_secs());

        let start_time = std::time::Instant::now();

        let mut link_counter: HashMap<NodeId, i64> = HashMap::new();

        let mut ways: Vec<Way> = Vec::new();

        for (id, obj) in &objs {
            if obj.is_node() {
                let node = obj.node().unwrap();
                osm_graph.add_node(node);
            }

            if obj.is_way() {
                let way = obj.way().unwrap();

                for node in &way.nodes {
                    let count = link_counter.get(&node).unwrap_or(&0) + 1;
                    link_counter.insert(*node, count);
                }

                osm_graph.add_way(way);
                ways.push(way.clone());
            }
        }

        for way in &ways {
            let mut source = NodeId(0);
            let mut one_way = way.tags.contains_key("oneway") && way.tags["oneway"] == "yes";

            let roundabout =
                way.tags.contains_key("junction") && way.tags["junction"] == "roundabout";

            if roundabout {
                one_way = true;
            }

            for node in &way.nodes {
                if node == &way.nodes[0] {
                    source = *node;
                }

                if node != &way.nodes[0] && node != &way.nodes[way.nodes.len() - 1] {
                    if link_counter[node] > 1 {
                        let index = way.nodes.iter().position(|x| *x == *node).unwrap();

                        let mut way1 = way.clone();

                        let source_index = way.nodes.iter().position(|x| *x == source).unwrap();
                        way1.nodes = way.nodes[source_index..index + 1].to_vec();

                        let mut weight = 1.0;
                        let mut distance = 0.0;

                        for i in 0..way1.nodes.len() - 1 {
                            let node1 = osm_graph.get_node(way1.nodes[i]).unwrap();
                            let node2 = osm_graph.get_node(way1.nodes[i + 1]).unwrap();

                            let distance1 = OSMGraph::haversine_distance(
                                node1.lat(),
                                node1.lon(),
                                node2.lat(),
                                node2.lon(),
                            );

                            distance += distance1;
                        }

                        let mut CURVY = 0.0;
                        let mut MIDDLE = 0.0;
                        let mut STRAIGHT = 0.0;

                        for i in 0..way1.nodes.len() - 2 {
                            let node1 = osm_graph.get_node(way1.nodes[i]).unwrap();
                            let node2 = osm_graph.get_node(way1.nodes[i + 1]).unwrap();
                            let node3 = osm_graph.get_node(way1.nodes[i + 2]).unwrap();

                            let radius = OSMGraph::circle_radius(node1, node2, node3);

                            // attributes a weight to the edge based on the radius : small curve, big curve, straight line
                            if (radius < 100.0) {
                                weight += 0.0; // * total_distance;
                                CURVY += 1.0;
                            } else if (radius < 200.0) {
                                weight += 0.0; // * total_distance;
                                MIDDLE += 1.0;
                            } else {
                                weight += 0.0; //* total_distance;
                                STRAIGHT += 1.0;
                            }
                        }

                        // calculate weight based on the number of curves (CURVY, MIDDLE, STRAIGHT) and the distance
                        if (CURVY > MIDDLE && CURVY > STRAIGHT) {
                            weight *= 0.5;
                        } else if (MIDDLE > CURVY && MIDDLE > STRAIGHT) {
                            weight *= 1.0;
                        } else {
                            weight *= 1.5;
                        }

                        weight *= distance;

                        let edge = Edge::new(
                            source,
                            *node,
                            distance,
                            weight,
                            0.0,
                            way1.nodes.clone(),
                            way1.id,
                        );

                        osm_graph.add_edge(edge.clone());

                        osm_graph.add_edge_from_node(source, edge.clone());

                        let nodes = way1
                            .nodes
                            .clone()
                            .into_iter()
                            .rev()
                            .collect::<Vec<NodeId>>();

                        if (!one_way) {
                            let edge =
                                Edge::new(*node, source, distance, weight, 0.0, nodes, way1.id);
                            osm_graph.add_edge(edge.clone());
                            osm_graph.add_edge_from_node(*node, edge.clone());
                        }

                        source = *node;
                    }
                }

                if node == &way.nodes[way.nodes.len() - 1] {
                    let source_index = way.nodes.iter().position(|x| *x == source).unwrap();

                    let mut weight = 1.0;
                    let mut distance = 0.0;

                    for i in 0..way.nodes.len() - 1 {
                        let node1 = osm_graph.get_node(way.nodes[i]).unwrap();
                        let node2 = osm_graph.get_node(way.nodes[i + 1]).unwrap();

                        let distance1 = OSMGraph::haversine_distance(
                            node1.lat(),
                            node1.lon(),
                            node2.lat(),
                            node2.lon(),
                        );

                        distance += distance1;
                    }

                    let mut CURVY = 0.0;
                    let mut MIDDLE = 0.0;
                    let mut STRAIGHT = 0.0;

                    // for each three nodes, calculate radius of the circle that passes through them
                    for i in 0..way.nodes.len() - 2 {
                        let node1 = osm_graph.get_node(way.nodes[i]).unwrap();
                        let node2 = osm_graph.get_node(way.nodes[i + 1]).unwrap();
                        let node3 = osm_graph.get_node(way.nodes[i + 2]).unwrap();

                        let radius = OSMGraph::circle_radius(node1, node2, node3);

                        // attributes a weight to the edge based on the radius : small curve, big curve, straight line
                        if (radius < 100.0) {
                            weight += 0.0; // * total_distance;
                            CURVY += 1.0;
                        } else if (radius < 200.0) {
                            weight += 0.0; // * total_distance;
                            MIDDLE += 1.0;
                        } else {
                            weight += 0.0; // * total_distance;
                            STRAIGHT += 1.0;
                        }
                    }

                    // Define weights for the scores (adjust these based on your preferences)
                    let w_curvy = 1.0;
                    let w_middle = 0.5;
                    let w_straight = 0.2;

                    // Calculate the overall score
                    let overall_score = w_curvy * CURVY + w_middle * MIDDLE - w_straight * STRAIGHT;

                    // calculate weight based on the number of curves (CURVY, MIDDLE, STRAIGHT) and the distance
                    if (CURVY > MIDDLE && CURVY > STRAIGHT) {
                        weight *= 0.5;
                    } else if (MIDDLE > CURVY && MIDDLE > STRAIGHT) {
                        weight *= 1.0;
                    } else {
                        weight *= 1.5;
                    }

                    weight *= distance;

                    let edge = Edge::new(
                        source,
                        *node,
                        distance,
                        weight,
                        0.0,
                        way.nodes[source_index..way.nodes.len()].to_vec(),
                        way.id,
                    );

                    osm_graph.add_edge(edge.clone());
                    osm_graph.add_edge_from_node(source, edge.clone());

                    let nodes = way.nodes[source_index..way.nodes.len()]
                        .to_vec()
                        .into_iter()
                        .rev()
                        .collect::<Vec<NodeId>>();

                    if (!one_way) {
                        let edge = Edge::new(*node, source, distance, weight, 0.0, nodes, way.id);
                        osm_graph.add_edge(edge.clone());
                        osm_graph.add_edge_from_node(*node, edge.clone());
                    }
                }
            }
        }

        info!("Build graph in {} seconds", start_time.elapsed().as_secs());
        println!("Build graph in {} seconds", start_time.elapsed().as_secs());

        info!("Nodes count: {}", osm_graph.get_node_count());
        println!("Nodes count: {}", osm_graph.get_node_count());
        info!("Ways count: {}", osm_graph.get_way_count());
        println!("Ways count: {}", osm_graph.get_way_count());
        info!("Edges count: {}", osm_graph.get_edge_count());
        println!("Edges count: {}", osm_graph.get_edge_count());

        osm_graph
    }
}
