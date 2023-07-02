use osmpbfreader::objects::{Node, NodeId, Way, WayId};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::Write;

struct JsonNode {
    latitude: f64,
    longitude: f64,
}

#[derive(Debug, Clone)]
pub struct OSMGraph {
    pub nodes: HashMap<NodeId, Node>,
    pub ways: HashMap<WayId, Way>,
    pub edges: Vec<Edge>,

    // TODO : add a variable to store the edges starting from a node
    pub edges_from_node: HashMap<NodeId, Vec<Edge>>,

    pub empty_edges: Vec<Edge>,
}

impl OSMGraph {
    pub fn new() -> Self {
        OSMGraph {
            nodes: HashMap::new(),
            ways: HashMap::new(),
            edges: Vec::new(),
            edges_from_node: HashMap::new(),
            empty_edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: &Node) -> () {
        self.nodes.insert(node.id, node.clone());
    }

    pub fn add_way(&mut self, way: &Way) -> () {
        self.ways.insert(way.id, way.clone());
    }

    pub fn add_edge(&mut self, edge: Edge) -> () {
        self.edges.push(edge);
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_way_count(&self) -> usize {
        self.ways.len()
    }

    pub fn get_edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn get_ways(&self) -> &HashMap<WayId, Way> {
        &self.ways
    }

    pub fn get_edges(&self) -> &Vec<Edge> {
        &self.edges
    }

    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_nodes(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }

    pub fn distance(&self, node_id_1: NodeId, node_id_2: NodeId) -> f64 {
        let node_1 = self.nodes.get(&node_id_1).unwrap();
        let node_2 = self.nodes.get(&node_id_2).unwrap();

        let lat_1 = node_1.decimicro_lat;
        let lon_1 = node_1.decimicro_lon;
        let lat_2 = node_2.decimicro_lat;
        let lon_2 = node_2.decimicro_lon;

        let distance = (lat_1 - lat_2).pow(2) + (lon_1 - lon_2).pow(2);

        (distance as f64).sqrt()
    }

    pub fn haversine_distance(from_lat: f64, from_lon: f64, to_lat: f64, to_lon: f64) -> f64 {
        let earth_radius = 6371.0; // km

        // Distance between latitudes and longitudes
        let d_lat = (to_lat - from_lat).to_radians();
        let d_lon = (to_lon - from_lon).to_radians();

        // Convert to radians
        let from_lat = from_lat.to_radians();
        let to_lat = to_lat.to_radians();

        // Apply formula
        let a = (d_lat / 2.0).sin() * (d_lat / 2.0).sin()
            + (d_lon / 2.0).sin() * (d_lon / 2.0).sin() * from_lat.cos() * to_lat.cos();
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        let distance = earth_radius * c;

        distance
    }

    pub fn get_nearest_node(&self, lat: f64, lon: f64) -> Option<NodeId> {
        let mut min_distance = std::f64::MAX;
        let mut nearest_node_id = None;

        for (node_id, node) in &self.nodes {
            let distance = OSMGraph::haversine_distance(lat, lon, node.lat(), node.lon());

            if distance < min_distance {
                min_distance = distance;
                nearest_node_id = Some(*node_id);
            }
        }

        //println!("Nearest node: {:?}", nearest_node_id);
        nearest_node_id
    }

    pub fn reconstruct_path(&self, visited_nodes: &Vec<NodeId>) -> serde_json::Value {
        let path = visited_nodes
            .iter()
            .map(|node_id| {
                let node = self.nodes.get(node_id).unwrap();
                json!({"latitude": node.lat(), "longitude": node.lon()})
            })
            .collect::<Vec<_>>();

        let json_obj = json!({ "path": path });

        json_obj
    }

    pub fn combine_paths(&self, path_1: Vec<NodeId>, path_2: Vec<NodeId>) -> Vec<NodeId> {
        let mut combined_path = path_1.clone();

        // add path_2 to combined_path, but skip the first node
        for node_id in path_2.iter().skip(1) {
            combined_path.push(*node_id);
        }

        combined_path
    }

    pub fn get_edges_from_node_or_containing(&self, node_id: NodeId) -> Vec<&Edge> {
        let mut edges = Vec::new();

        for edge in &self.edges {
            if edge.from == node_id {
                edges.push(edge);
            }
        }

        if (edges.len() == 0) {
            //println!("No edges found for node: {:?}", node_id);

            for edge in &self.edges {
                for n_id in &edge.nodes_ids {
                    if node_id == *n_id {
                        edges.push(edge);
                        //println!("Found edge containing node: {:?}", node_id);
                    }
                }
            }

            //println!("Found {} edges containing node: {:?}", edges.len(), node_id);
        }
        //println!("Found {} edges from node: {:?}", edges.len(), node_id);

        edges
    }

    pub fn get_edges_from_node(&self, node_id: NodeId) -> Vec<&Edge> {
        let mut edges = Vec::new();

        for edge in &self.edges {
            if edge.from == node_id {
                edges.push(edge);
            }
        }

        //println!("Found {} edges from node: {:?}", edges.len(), node_id);

        edges
    }

    pub fn add_edge_from_node(&mut self, node_id: NodeId, edge: Edge) {
        // add edge to edges_from_node hashmap
        let edges = self.edges_from_node.entry(node_id).or_insert(Vec::new());
        edges.push(edge);
    }

    pub fn get_edges_from_node_fast(&self, node_id: &NodeId) -> &Vec<Edge> {
        // return edges from edges_from_node hashmap
        let edges = self.edges_from_node.get(node_id);

        edges.unwrap_or(&self.empty_edges)
    }

    pub fn get_edges_to_node(&self, node_id: NodeId) -> Vec<&Edge> {
        let mut edges = Vec::new();

        for edge in &self.edges {
            if edge.to == node_id {
                edges.push(edge);
            }
        }

        edges
    }
}

#[derive(Debug, Clone)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub distance: f64,
    pub weight: f64,
    pub nodes_ids: Vec<NodeId>,
}

impl Edge {
    pub fn new(
        from: NodeId,
        to: NodeId,
        distance: f64,
        weight: f64,
        nodes_ids: Vec<NodeId>,
    ) -> Self {
        Edge {
            from: from,
            to: to,
            distance: distance,
            weight: weight,
            nodes_ids: nodes_ids,
        }
    }
}

#[derive(Debug, Clone)]
pub struct State {
    pub node_id: NodeId,
    pub distance: f64,
}

impl State {
    pub fn new(node_id: NodeId, distance: f64) -> State {
        State { node_id, distance }
    }
}

impl Eq for State {}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.distance == other.distance
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.distance.partial_cmp(&self.distance)
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap_or(Ordering::Equal)
    }
}
