extern crate osmpbfreader;
use osmpbfreader::{OsmPbfReader, objects::OsmObj};
use osmpbfreader::objects::{Node, Way, NodeId, WayId};
use std::fs::File;
use std::io::BufReader;
use crate::osm_graph::OSMGraph;
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

        let mut nodes_to_keep = Vec::new();

        nodes_to_keep.push(0);

        let highway_to_keep = vec!["trunk", "primary", "secondary", "tertiary", "unclassified", "residential", "trunk_link", "primary_link", "secondary_link", "tertiary_link", "living_street"];

        // Read only ways in parallel, that are in highway_to_keep



        /*for obj in pbf_reader.par_iter() {
            //use std::process::exit;
            //let obj = obj.unwrap_or_else(|e| {println!("{:?}", e); exit(1)});

            match obj.unwrap() {
                OsmObj::Node(node) => {
                    // get tags in HashMap
                    let mut tags = HashMap::new();

                    for (key, value) in node.tags.iter() {
                        tags.insert(key.to_string(), value.to_string());
                    }
                    
                    // NodeId is i64
                    let id = node.id.0 as i64;

                    let lat = node.decimicro_lat as f64 / 10_000_000.0;
                    let lon = node.decimicro_lon as f64 / 10_000_000.0;

                    //println!("lat: {:?}", lat);

                    graph.add_node(Node::new(id, lat, lon, tags));

                    //println!("Node: {:?}", node);
                },
                OsmObj::Way(way) => {
                    let id = way.id.0 as i64;

                    let mut tags = HashMap::new();

                    for (key, value) in way.tags.iter() {
                        tags.insert(key.to_string(), value.to_string());
                    }

                    // only add ways with highway tag, and with highway tag in highway_to_keep
                    if !tags.contains_key("highway") || !highway_to_keep.contains(&tags["highway"].as_str()) {
                        continue;
                    }

                    let node_ids: Vec<i64> = way.nodes.iter().map(|node| node.0 as i64).collect();

                    // add node ids to nodes to keep
                    for node_id in &node_ids {
                        nodes_to_keep.push(*node_id);
                    }

                    graph.add_way(Way::new(id, node_ids.clone(), tags));
                    
                    //println!("Way: {:?}", way);
                },
                OsmObj::Relation(relation) => {
                    //println!("Relation: {:?}", relation);
                    //println!("Relation: {:?}", relation.tags);
                },
            }
        }*/

        let objs = pbf_reader.get_objs_and_deps(|obj| {
            obj.is_way() && obj.tags().contains_key("highway") && highway_to_keep.contains(&obj.tags()["highway"].as_str())
        }).unwrap();

        println!("Read file in {} seconds", start_time.elapsed().as_secs());

        let start_time = std::time::Instant::now();

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
                osm_graph.add_way(way);
            }
        }

        println!("Build graph in {} seconds", start_time.elapsed().as_secs());

        println!("Nodes count: {}", osm_graph.get_node_count());
        println!("Ways count: {}", osm_graph.get_way_count());


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