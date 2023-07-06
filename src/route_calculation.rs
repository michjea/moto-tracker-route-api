use std::collections::{BinaryHeap, HashMap, HashSet};
//use crate::graph::Graph;
use crate::osm_graph::Edge;
use crate::osm_graph::OSMGraph;
use crate::osm_graph::State;
use osmpbfreader::objects::{Node, NodeId, Way, WayId};
//use crate::graph::Node;
//use crate::graph::State;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub fn calculate_radius() {}

pub fn calculate_curve_wheight() {}

/*pub fn bidirectional_dijkstra_path(graph: &Graph, start_node: &Node, end_node: &Node) -> Option<Vec<i64>> {
    let mut forward_distances = HashMap::new();
    let mut backward_distances = HashMap::new();

    println!("Start node: {:?}", start_node.id);
    println!("End node: {:?}", end_node.id);

    forward_distances.insert(start_node.id, 0.0);
    backward_distances.insert(end_node.id, 0.0);

    let mut forward_heap = BinaryHeap::new();
    let mut backward_heap = BinaryHeap::new();

    forward_heap.push(State::new(start_node.id, 0.0));
    backward_heap.push(State::new(end_node.id, 0.0));

    let mut forward_prev_nodes = HashMap::new();
    let mut backward_prev_nodes = HashMap::new();

    while let Some(State { node_id: forward_node_id, distance: forward_dist }) = forward_heap.pop() {
        if forward_node_id == end_node.id || backward_distances.contains_key(&forward_node_id) {
            // Intersection point found
            let forward_path = graph.reconstruct_path(&forward_prev_nodes, forward_node_id);
            let backward_path = graph.reconstruct_path(&backward_prev_nodes, forward_node_id);
            return Some(graph.combine_paths(forward_path, backward_path));
        }

        if let Some(current_dist) = forward_distances.get(&forward_node_id) {
            if forward_dist < *current_dist {
                continue;
            }
        }

        for edge in graph.get_edges_from_node(forward_node_id) {
            println!("Edge: {:?}", edge);
            let new_dist = forward_dist + edge.weight;
            if !forward_distances.contains_key(&edge.to) || new_dist < forward_distances[&edge.to] {
                forward_heap.push(State::new(edge.to, new_dist));
                forward_distances.insert(edge.to, new_dist);
                forward_prev_nodes.insert(edge.to, forward_node_id);
            }
        }
    }

    while let Some(State { node_id: backward_node_id, distance: backward_dist }) = backward_heap.pop() {
        if backward_node_id == start_node.id || forward_distances.contains_key(&backward_node_id) {
            // Intersection point found
            let forward_path = graph.reconstruct_path(&forward_prev_nodes, backward_node_id);
            let backward_path = graph.reconstruct_path(&backward_prev_nodes, backward_node_id);
            return Some(graph.combine_paths(forward_path, backward_path));
        }

        if let Some(current_dist) = backward_distances.get(&backward_node_id) {
            if backward_dist < *current_dist {
                continue;
            }
        }

        for edge in graph.get_edges_to_node(backward_node_id) {
            let new_dist = backward_dist + edge.weight;
            if !backward_distances.contains_key(&edge.from) || new_dist < backward_distances[&edge.from] {
                backward_heap.push(State::new(edge.from, new_dist));
                backward_distances.insert(edge.from, new_dist);
                backward_prev_nodes.insert(edge.from, backward_node_id);
            }
        }
    }

    None // No path found
}*/

/*pub fn bidirectional_dijkstra_path_2(
    graph: &OSMGraph,
    start_node: &NodeId,
    end_node: &NodeId,
) -> Option<Vec<NodeId>> {
    let mut forward_distances: HashMap<NodeId, f64> = HashMap::new();
    let mut backward_distances: HashMap<NodeId, f64> = HashMap::new();

    println!("Start node: {:?}", start_node);
    println!("End node: {:?}", end_node);

    forward_distances.insert(*start_node, 0.0);
    backward_distances.insert(*end_node, 0.0);

    let mut forward_heap = BinaryHeap::new();
    let mut backward_heap = BinaryHeap::new();

    forward_heap.push(State::new(*start_node, 0.0));
    backward_heap.push(State::new(*end_node, 0.0));

    let mut forward_prev_nodes: HashMap<NodeId, NodeId> = HashMap::new();
    let mut backward_prev_nodes: HashMap<NodeId, NodeId> = HashMap::new();

    while let Some(State {
        node_id: forward_node_id,
        distance: forward_dist,
    }) = forward_heap.pop()
    {
        if forward_node_id == *end_node || backward_distances.contains_key(&forward_node_id) {
            // Intersection point found
            let forward_path: Vec<NodeId> =
                graph.reconstruct_path(&forward_prev_nodes, forward_node_id);
            let backward_path: Vec<NodeId> =
                graph.reconstruct_path(&backward_prev_nodes, forward_node_id);
            return Some(graph.combine_paths(forward_path, backward_path));
        }

        if let Some(current_dist) = forward_distances.get(&forward_node_id) {
            if forward_dist < *current_dist {
                continue;
            }
        }

        for edge in graph.get_edges_from_node(forward_node_id) {
            //println!("Edge: {:?}", edge);
            let new_dist = forward_dist + edge.weight;
            if !forward_distances.contains_key(&edge.to) || new_dist < forward_distances[&edge.to] {
                forward_heap.push(State::new(edge.to, new_dist));
                forward_distances.insert(edge.to, new_dist);
                forward_prev_nodes.insert(edge.to, forward_node_id);
            }
        }
    }

    while let Some(State {
        node_id: backward_node_id,
        distance: backward_dist,
    }) = backward_heap.pop()
    {
        if backward_node_id == *start_node || forward_distances.contains_key(&backward_node_id) {
            // Intersection point found
            let forward_path: Vec<NodeId> =
                graph.reconstruct_path(&forward_prev_nodes, backward_node_id);
            let backward_path: Vec<NodeId> =
                graph.reconstruct_path(&backward_prev_nodes, backward_node_id);
            return Some(graph.combine_paths(forward_path, backward_path));
        }

        if let Some(current_dist) = backward_distances.get(&backward_node_id) {
            if backward_dist < *current_dist {
                continue;
            }
        }

        for edge in graph.get_edges_to_node(backward_node_id) {
            let new_dist = backward_dist + edge.weight;
            if !backward_distances.contains_key(&edge.from)
                || new_dist < backward_distances[&edge.from]
            {
                backward_heap.push(State::new(edge.from, new_dist));
                backward_distances.insert(edge.from, new_dist);
                backward_prev_nodes.insert(edge.from, backward_node_id);
            }
        }
    }

    None // No path found
}*/

pub fn dijkstra(
    graph: &OSMGraph,
    start_node: &NodeId,
    end_node: &NodeId,
) -> Option<serde_json::Value> {
    //Option<Vec<NodeId>> {
    let mut distances: HashMap<NodeId, f64> = HashMap::new();

    // fill distances with infinity
    // get_nodes returns a hashmap, so we need to iterate over the keys
    for node_id in graph.get_nodes().keys() {
        distances.insert(*node_id, f64::INFINITY);
    }

    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut heap = BinaryHeap::new();
    let mut start_edge_end_node = NodeId(-1);

    // Start node edge -> because start node is not necessarly the first node of an edge
    let start_edge: Vec<&Edge> = graph.get_edges_from_node_or_containing(*start_node);
    let start_node_edge: &Edge = &start_edge[0];

    distances.insert(start_node_edge.from, 0.0);
    heap.push(State::new(start_node_edge.from, 0.0));

    let mut prev_nodes: HashMap<NodeId, NodeId> = HashMap::new(); // TODO : prev edge ?

    //find edges containing end node in nodes_id
    let mut end_edges: Vec<Edge> = Vec::new();
    for edge in graph.get_edges() {
        for node_id_ in &edge.nodes_ids {
            if node_id_ == end_node {
                end_edges.push(edge.clone());
            }
        }
    }

    //let end_node_edge: &NodeId = &end_edges[0].from;
    let end_node_edge: &NodeId = &end_edges[0].to;

    while let Some(State { node_id, distance }) = heap.pop() {
        if node_id == *end_node
            //|| node_id == start_edge_end_node
            || node_id == *end_node_edge
        //|| node_id == *end_node_edge_2
        {
            // Intersection point found
            println!("Intersection point found: {:?}", node_id);

            // reconstruct path
            let mut path: Vec<NodeId> = Vec::new();
            let mut current_node = node_id;
            //println!("start_node: {:?}", *start_node);
            //println!("start_node_edge.from: {:?}", start_node_edge.from);

            while current_node != start_node_edge.from {
                path.push(current_node);
                current_node = prev_nodes[&current_node];
                //println!("current_node: {:?}", current_node);
            }

            path.push(*start_node);

            return Some(graph.reconstruct_path(&path));
        }

        // if node is not visited, add it to visited
        if !visited.contains(&node_id) {
            visited.insert(node_id);
        } else {
            continue;
        }

        // if distance is greater than distance to end node, skip
        if let Some(current_dist) = distances.get(&node_id) {
            if distance > *current_dist {
                continue;
            }
        }

        // push node to prev_nodes
        //prev_nodes.push(node_id);

        for edge in graph.get_edges_from_node_fast(&node_id) {
            //graph.get_edges_from_node(node_id) {
            // if node is not visited
            if !visited.contains(&edge.to) {
                let new_dist = distance + edge.weight;

                // if node is not in distances
                if !distances.contains_key(&edge.to) || new_dist < distances[&edge.to] {
                    heap.push(State::new(edge.to, new_dist));
                    distances.insert(edge.to, new_dist);
                    //prev_nodes.push(node_id);
                    prev_nodes.insert(edge.to, node_id);
                }

                // check if end node is in the nodes_ids of the edge
                /*for node_id_ in &edge.nodes_ids {
                    if node_id_ == end_node {
                        start_edge_end_node = edge.to;
                        //println!("Start edge end node: {:?}", start_edge_end_node);
                    }
                }*/
            }
        }
    }

    None // No path found
}
