use std::collections::{BinaryHeap, HashMap};
use crate::graph::Graph;
use crate::graph::Node;
use crate::graph::State;

pub fn calculate_radius(){}

pub fn calculate_curve_wheight(){}

pub fn bidirectional_dijkstra_path(graph: &Graph, start_node: &Node, end_node: &Node) -> Option<Vec<i64>> {
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
}