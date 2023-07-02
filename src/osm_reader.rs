extern crate osmpbfreader;
use crate::osm_graph::Edge;
use crate::osm_graph::OSMGraph;
use osmpbfreader::objects::{Node, NodeId, Way, WayId};
use osmpbfreader::{objects::OsmObj, OsmPbfReader};
use std::fs::File;
use std::io::BufReader;
//use crate::graph::Graph;
//use crate::graph::Node;
//use crate::graph::Way;
//use crate::graph::Edge;
//use crate::graph::Relation;

use std::collections::HashMap;

pub struct OSMReader {
    file_path: String,
}

impl OSMReader {
    pub fn new(file_path: String) -> Self {
        OSMReader {
            file_path: file_path,
        }
    }

    pub fn build_graph(&mut self) -> OSMGraph {
        let file = File::open(&self.file_path).expect("Unable to open file");
        let reader = BufReader::new(file);
        let mut pbf_reader = OsmPbfReader::new(reader);

        let start_time = std::time::Instant::now();

        //let mut graph = Graph::new();
        let mut osm_graph = OSMGraph::new();

        println!("Start reading file...");

        //let mut nodes_to_keep = Vec::new();

        //nodes_to_keep.push(0);

        let highway_to_keep = vec![
            "trunk",
            "primary",
            "secondary",
            "tertiary",
            "unclassified",
            "residential",
            "trunk_link",
            "primary_link",
            "secondary_link",
            "tertiary_link",
            "living_street",
        ];

        let objs = pbf_reader
            .get_objs_and_deps(|obj| {
                obj.is_way()
                    && obj.tags().contains_key("highway")
                    && highway_to_keep.contains(&obj.tags()["highway"].as_str())
            })
            .unwrap();

        println!("Read file in {} seconds", start_time.elapsed().as_secs());

        let start_time = std::time::Instant::now();

        let mut link_counter: HashMap<NodeId, i64> = HashMap::new();

        let mut ways: Vec<Way> = Vec::new();

        for (id, obj) in &objs {
            // si c'est un node
            if obj.is_node() {
                // add node with graph.add_node(node)
                let node = obj.node().unwrap();
                osm_graph.add_node(node);
            }

            if obj.is_way() {
                // add way with graph.add_way(way)
                let way = obj.way().unwrap();

                // for each node in way, add node to nodes_count, or increment it
                for node in &way.nodes {
                    let count = link_counter.get(&node).unwrap_or(&0) + 1;
                    link_counter.insert(*node, count);
                }

                osm_graph.add_way(way);
                ways.push(way.clone());
            }
        }

        // parse all ways a second time; a way will normally become one edge, but if any nodes apart from the first and the last have a link counter greater than one, then split the way into two edges at that point.

        // for each way in graph
        for way in &ways {
            // for each node in way

            let mut source = NodeId(0);

            // check if way is one way or two way
            let one_way = way.tags.contains_key("oneway") && way.tags["oneway"] == "yes";

            for node in &way.nodes {
                // si c'est le premier
                if node == &way.nodes[0] {
                    source = *node;
                }
                // if node is not first or last node in way
                if node != &way.nodes[0] && node != &way.nodes[way.nodes.len() - 1] {
                    // if node has a link counter greater than one
                    if link_counter[node] > 1 {
                        // split way into two edges at that point
                        //println!("Split way into two edges at that point");
                        //println!("Way: {:?}", way);
                        //println!("Node: {:?}", node);
                        //println!("Link counter: {:?}", link_counter[node]);
                        //println!("Way nodes: {:?}", way.nodes);
                        //println!("Way nodes len: {:?}", way.nodes.len());

                        // get index of node in way
                        let index = way.nodes.iter().position(|x| *x == *node).unwrap();

                        //println!("Index: {:?}", index);

                        // create two new ways
                        let mut way1 = way.clone();
                        //let mut way2 = way.clone();

                        //println!("Way1: {:?}", way1);
                        //println!("Way2: {:?}", way2);

                        // set way1 nodes
                        let source_index = way.nodes.iter().position(|x| *x == source).unwrap();
                        way1.nodes = way.nodes[source_index..index + 1].to_vec();

                        let mut distance = way1.nodes.len() as f64;

                        /*for i in 0..way1.nodes.len() - 1 {
                            let node1 = osm_graph.get_node(way1.nodes[i]).unwrap();
                            let node2 = osm_graph.get_node(way1.nodes[i + 1]).unwrap();
                            distance += OSMGraph::haversine_distance(node1.lat() as f64 / 10_000_000.0, node1.lon() as f64 / 10_000_000.0, node2.lat() as f64 / 10_000_000.0, node2.lon() as f64 / 10_000_000.0);
                        }*/

                        //println!("Distance: {:?}", distance);

                        //println!("Way1: {:?}", way1);

                        // set way2 nodes
                        //way2.nodes = way.nodes[index..way.nodes.len()].to_vec();

                        // add way1 and way2 to graph
                        //osm_graph.add_way(way1);
                        //osm_graph.add_way(way2);
                        // from: i64, to: i64, distance: f64, weight, nodes_ids: Vec<i64>
                        //println!("Distance: {:?}", distance);

                        if (distance < 1.0) {
                            //println!("Distance: {:?}", distance);
                        }

                        let edge = Edge::new(source, *node, distance, distance, way1.nodes.clone());

                        osm_graph.add_edge(edge.clone());

                        osm_graph.add_edge_from_node(source, edge.clone());

                        let nodes = way1
                            .nodes
                            .clone()
                            .into_iter()
                            .rev()
                            .collect::<Vec<NodeId>>();

                        if (!one_way) {
                            let edge = Edge::new(*node, source, distance, distance, nodes);
                            osm_graph.add_edge(edge.clone());
                            osm_graph.add_edge_from_node(*node, edge.clone());
                        }

                        source = *node;
                    }
                }
                // si c'est le dernier, on ajoute un edge
                if node == &way.nodes[way.nodes.len() - 1] {
                    // add edge with graph.add_edge(edge)
                    // from: i64, to: i64, distance: f64, weight, nodes_ids: Vec<i64>
                    let source_index = way.nodes.iter().position(|x| *x == source).unwrap();

                    let mut distance = way.nodes.len() as f64;

                    /*for i in 0..way.nodes.len() - 1 {
                        let node1 = osm_graph.get_node(way.nodes[i]).unwrap();
                        let node2 = osm_graph.get_node(way.nodes[i + 1]).unwrap();
                        distance += OSMGraph::haversine_distance(node1.lat() as f64 / 10_000_000.0, node1.lon() as f64 / 10_000_000.0, node2.lat() as f64 / 10_000_000.0, node2.lon() as f64 / 10_000_000.0);
                    }*/

                    let edge = Edge::new(
                        source,
                        *node,
                        distance,
                        distance,
                        way.nodes[source_index..way.nodes.len()].to_vec(),
                    );

                    osm_graph.add_edge(edge.clone());
                    osm_graph.add_edge_from_node(source, edge.clone());

                    let nodes = way.nodes.clone().into_iter().rev().collect::<Vec<NodeId>>();

                    if (!one_way) {
                        let edge = Edge::new(*node, source, distance, distance, nodes);
                        osm_graph.add_edge(edge.clone());
                        osm_graph.add_edge_from_node(*node, edge.clone());
                    }
                }
            }
        }

        println!("Build graph in {} seconds", start_time.elapsed().as_secs());

        println!("Nodes count: {}", osm_graph.get_node_count());
        println!("Ways count: {}", osm_graph.get_way_count());
        println!("Edges count: {}", osm_graph.get_edge_count());

        /*
        // for all nodes, check if they are in nodes_to_keep, if not, remove them
        //let mut nodes_to_remove: Vec<i64> = Vec::new();

        // log nodes to keep length
        println!("Nodes to keep: {}", nodes_to_keep.len());

        // log total nodes length
        println!("Total nodes: {}", graph.get_nodes().len());

        println!("Get nodes to remove...");

        let mut count = 0;

        /*for node_id in graph.get_nodes().keys() {
            //println!("Node id: {}", node_id);
            if !nodes_to_keep.contains(node_id) {
                //println!("Node id: {} count: {}", node_id, count);
                count += 1;

                // directly remove node from graph
                graph.remove_node(*node_id);

                if count % 1000 == 0 {
                    println!("Count: {}", count);
                }
            }
        }*/

        // print nodes to remove length
        //println!("Nodes to remove: {}", nodes_to_remove.len());

        // nodes to remove in functional style
        //nodes_to_remove = graph.get_nodes().keys().filter(|node_id| !nodes_to_keep.contains(node_id)).map(|node_id| *node_id).collect();

        // log nodes to remove length
        //println!("Nodes to remove 2: {}", nodes_to_remove.len());

        /*println!("Remove nodes...");
        for node_id in nodes_to_remove {
            graph.remove_node(node_id);
        }*/

        // print hashmap capacity
        println!("HashMap capacity: {}", graph.get_nodes().capacity());

        // remove nodes from graph that are not in nodes_to_keep
        graph.nodes.retain(|node_id, _| nodes_to_keep.contains(node_id));

        println!("Split ways into edges...");
        let ways: Vec<Way> = graph.get_ways().values().cloned().collect();

        // Split ways into edges
        for way in ways {
            let node_ids: &Vec<i64> = way.get_node_ids();
            let prev_node_id = node_ids[0];

            // print if node count > 2
            if node_ids.len() < 2 {
                println!("Way {} has {} nodes", way.get_id(), node_ids.len());
            }

            for node_id in node_ids.iter().skip(1) {
                let from = graph.get_node(prev_node_id).unwrap();
                let to = graph.get_node(*node_id).unwrap();
                let distance = from.get_distance(to.get_lat(), to.get_lon());
                let way_id = way.get_id();
                let weight = distance;

                //println!("Add edge from {} to {}", prev_node_id, *node_id);
                graph.add_edge(Edge::new(prev_node_id, *node_id, way_id, distance, weight, Vec::new()));
           }
        }

        let end_time = std::time::Instant::now();

        println!("Done");

        println!("Time elapsed: {:?}", end_time.duration_since(start_time));


        // print graph stats
        println!("Nodes: {}", graph.get_node_count());
        println!("Ways: {}", graph.get_way_count());
        println!("Edges: {}", graph.get_edge_count());*/

        osm_graph
    }
}
