use crate::osm_graph::Edge;
use crate::osm_graph::OSMGraph;
use crate::osm_graph::State;
use log::{info, warn};
use osmpbfreader::objects::{Node, NodeId, Way, WayId};
use rand::Rng;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{BinaryHeap, HashMap, HashSet};

/// Dijkstra algorithm to find the shortest path between two nodes
/// Returns a tuple of two vectors, the first one contains the nodes ids of the path
/// and the second one contains the edges of the path
/// If no path is found, returns None
/// # Arguments
/// * `graph` - The graph to search in
/// * `start_node` - The start node
/// * `end_node` - The end node
pub fn dijkstra(
    graph: &OSMGraph,
    start_node: &NodeId,
    end_node: &NodeId,
) -> (Option<Vec<NodeId>>, Option<Vec<Edge>>) {
    let mut distances: HashMap<NodeId, f64> = HashMap::new();

    for node_id in graph.get_nodes().keys() {
        distances.insert(*node_id, f64::INFINITY);
    }

    let mut visited: HashSet<NodeId> = HashSet::new();
    let mut heap = BinaryHeap::new();
    let mut start_edge_end_node = NodeId(-1);

    let start_edge: Vec<&Edge> = graph.get_edges_from_node_or_containing(*start_node);
    let start_node_edge: &Edge = &start_edge[0];

    distances.insert(start_node_edge.from, 0.0);
    heap.push(State::new(start_node_edge.from, 0.0));

    let mut prev_nodes: HashMap<NodeId, NodeId> = HashMap::new(); // TODO : prev edge ?
    let mut prev_edges: HashMap<NodeId, Edge> = HashMap::new();

    let mut end_edges: Vec<Edge> = Vec::new();
    for edge in graph.get_edges() {
        for node_id_ in &edge.nodes_ids {
            if node_id_ == end_node {
                end_edges.push(edge.clone());
            }
        }
    }

    let end_node_edge: &NodeId = &end_edges[0].to;

    while let Some(State { node_id, distance }) = heap.pop() {
        if node_id == *end_node || node_id == *end_node_edge {
            println!("Intersection point found: {:?}", node_id);

            let mut edge_path: Vec<Edge> = Vec::new();
            let mut path: Vec<NodeId> = Vec::new();
            let mut current_node = node_id;

            while current_node != start_node_edge.from {
                path.push(current_node);
                edge_path.push(prev_edges[&current_node].clone());
                current_node = prev_nodes[&current_node];
            }

            path.push(*start_node);

            path.reverse();
            edge_path.reverse();

            return (Some(path), Some(edge_path));
        }

        if !visited.contains(&node_id) {
            visited.insert(node_id);
        } else {
            continue;
        }

        if let Some(current_dist) = distances.get(&node_id) {
            if distance > *current_dist {
                continue;
            }
        }

        for edge in graph.get_edges_from_node_fast(&node_id) {
            if !visited.contains(&edge.to) {
                let new_dist = distance + edge.weight;

                if !distances.contains_key(&edge.to) || new_dist < distances[&edge.to] {
                    heap.push(State::new(edge.to, new_dist));
                    distances.insert(edge.to, new_dist);
                    prev_nodes.insert(edge.to, node_id);
                    prev_edges.insert(edge.to, edge.clone());
                }
            }
        }
    }

    warn!("No path found");

    (None, None) // No path found
}

/// Function generating random points around a start node
/// Returns a vector of node ids
/// # Arguments
/// * `distance_km` - The distance in kilometers for the path. We divide by 2 to get the radius
/// * `graph` - The graph to search in
/// * `start_node` - The start node
pub fn generate_random_points(
    distance_km: f64,
    graph: &OSMGraph,
    start_node: &NodeId,
) -> Vec<NodeId> {
    let mut rng = rand::thread_rng();
    let mut points = Vec::new();

    let start_lat = graph.get_node(*start_node).unwrap().lat();
    let start_lon = graph.get_node(*start_node).unwrap().lon();

    // Convert distance_km from kilometers to degrees (assuming Mercator projection)
    // 1 degree of latitude is approximately 111.32 km
    let distance_deg = distance_km / 111.32;

    // Generate random points number between 2 and 5
    let points_number = rng.gen_range(2..3);

    // Generate random points
    for _ in 0..points_number {
        let angle = rng.gen_range(0.0..(2.0 * std::f64::consts::PI));

        // Générer une distance aléatoire entre 0 et la distance / 2
        let distance_deg_random = rng.gen_range(0.0..(distance_deg / 2.0));

        let lat = start_lat + distance_deg_random * angle.cos();
        let lon = start_lon + distance_deg_random * angle.sin();

        let node_id = graph.get_nearest_node(lat, lon).unwrap();
        points.push(node_id);
    }

    points
}

/// Function generating a random loop around a start node
/// Returns a vector of tuples containing the nodes ids and the edges of the path
/// # Arguments
/// * `distance` - The distance in kilometers for the path.
/// * `graph` - The graph to search in
/// * `start_node` - The start node
pub fn generate_random_loop(
    distance: f64,
    graph: &OSMGraph,
    start_node: &NodeId,
) -> Vec<(Vec<NodeId>, Vec<Edge>)> {
    let mut points = generate_random_points(distance, graph, start_node);

    let mut path = Vec::new();

    let mut first_lat = graph.get_node(*start_node).unwrap().lat();
    let mut first_lon = graph.get_node(*start_node).unwrap().lon();

    for _ in 0..points.len() {
        let mut min_distance = f64::INFINITY;
        let mut min_index = 0;

        for (index, point) in points.iter().enumerate() {
            let lat = graph.get_node(*point).unwrap().lat();
            let lon = graph.get_node(*point).unwrap().lon();

            let distance = (lat - first_lat).powi(2) + (lon - first_lon).powi(2);

            if distance < min_distance {
                min_distance = distance;
                min_index = index;
            }
        }

        first_lat = graph.get_node(points[min_index]).unwrap().lat();
        first_lon = graph.get_node(points[min_index]).unwrap().lon();

        path.push(points[min_index]);
        points.remove(min_index);
    }

    path.insert(0, *start_node);
    path.push(*start_node);

    let mut route = Vec::new();

    for i in 0..path.len() - 1 {
        let (nodes, edges) = dijkstra(graph, &path[i], &path[i + 1]);
        // unwrap edges and nodes
        let nodes = nodes.unwrap();
        let edges = edges.unwrap();
        route.push((nodes, edges));
    }
    route
}
