use osmpbfreader::objects::{Node, NodeId, Way, WayId};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::io::Write;

/// OSMGraph struct that contains the nodes, ways and edges of the graph
/// # Attributes
/// * `nodes` - The nodes of the graph
/// * `ways` - The ways of the graph
/// * `edges` - The edges of the graph
/// * `edges_from_node` - A hashmap that contains the edges that start from a node
/// * `empty_edges` - An empty vector of edges
#[derive(Debug, Clone)]
pub struct OSMGraph {
    pub nodes: HashMap<NodeId, Node>,
    pub ways: HashMap<WayId, Way>,
    pub edges: Vec<Edge>,
    pub edges_from_node: HashMap<NodeId, Vec<Edge>>,
    pub empty_edges: Vec<Edge>,
}

/// OSMGraph implementation
impl OSMGraph {
    /// Create a new OSMGraph
    /// # Returns
    /// * `OSMGraph` - The new OSMGraph
    pub fn new() -> Self {
        OSMGraph {
            nodes: HashMap::new(),
            ways: HashMap::new(),
            edges: Vec::new(),
            edges_from_node: HashMap::new(),
            empty_edges: Vec::new(),
        }
    }

    /// Add a node to the graph
    /// # Arguments
    /// * `node` - The node to add
    pub fn add_node(&mut self, node: &Node) -> () {
        self.nodes.insert(node.id, node.clone());
    }

    /// Add a way to the graph
    /// # Arguments
    /// * `way` - The way to add
    pub fn add_way(&mut self, way: &Way) -> () {
        self.ways.insert(way.id, way.clone());
    }

    /// Add an edge to the graph
    /// # Arguments
    /// * `edge` - The edge to add
    pub fn add_edge(&mut self, edge: Edge) -> () {
        self.edges.push(edge);
    }

    /// Get the number of nodes in the graph
    /// # Returns
    /// * `usize` - The number of nodes in the graph
    pub fn get_node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Get the number of ways in the graph
    /// # Returns
    /// * `usize` - The number of ways in the graph
    pub fn get_way_count(&self) -> usize {
        self.ways.len()
    }

    /// Get the number of edges in the graph
    /// # Returns
    /// * `usize` - The number of edges in the graph
    pub fn get_edge_count(&self) -> usize {
        self.edges.len()
    }

    /// Get the ways of the graph
    /// # Returns
    /// * `&HashMap<WayId, Way>` - A reference to the ways of the graph (key: way id, value: way)
    pub fn get_ways(&self) -> &HashMap<WayId, Way> {
        &self.ways
    }

    /// Get the edges of the graph
    /// # Returns
    /// * `&Vec<Edge>` - A reference to the edges of the graph
    pub fn get_edges(&self) -> &Vec<Edge> {
        &self.edges
    }

    /// Get a node from the graph
    /// # Arguments
    /// * `id` - The id of the node to get
    /// # Returns
    /// * `Option<&Node>` - A reference to the node if it exists, None otherwise
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(&id)
    }

    /// Get the nodes of the graph
    /// # Returns
    /// * `&HashMap<NodeId, Node>` - A reference to the nodes of the graph (key: node id, value: node)
    pub fn get_nodes(&self) -> &HashMap<NodeId, Node> {
        &self.nodes
    }

    /// Calculate the distance between two nodes using the haversine formula
    /// # Arguments
    /// * `from_lat` - The latitude of the first node
    /// * `from_lon` - The longitude of the first node
    /// * `to_lat` - The latitude of the second node
    /// * `to_lon` - The longitude of the second node
    /// # Returns
    /// * `f64` - The distance between the two nodes in meters
    /// # Formula
    /// * `r` - The radius of the earth in km
    /// * `delta_lat` - The difference between the latitudes of the two nodes in radians
    /// * `delta_lon` - The difference between the longitudes of the two nodes in radians
    /// * `a` - The first part of the formula
    /// * `c` - The second part of the formula
    /// * `distance` - The distance between the two nodes in km
    pub fn haversine_distance(from_lat: f64, from_lon: f64, to_lat: f64, to_lon: f64) -> f64 {
        let r = 6371.0; // km

        // Convert to radians
        let from_lat = from_lat.to_radians();
        let to_lat = to_lat.to_radians();

        let from_lon = from_lon.to_radians();
        let to_lon = to_lon.to_radians();

        let delta_lat = to_lat - from_lat;
        let delta_lon = to_lon - from_lon;

        // Apply formula
        let a = (delta_lat / 2.0).sin().powi(2)
            + from_lat.cos() * to_lat.cos() * (delta_lon / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().asin();

        let distance = r * c;

        distance * 1000.0 // convert to meters
    }

    /// Get the nearest node from a given latitude and longitude
    /// # Arguments
    /// * `lat` - The latitude
    /// * `lon` - The longitude
    /// # Returns
    /// * `Option<NodeId>` - The id of the nearest node if it exists, None otherwise
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

        nearest_node_id
    }

    /// Reconstruction of the path from the visited nodes
    /// # Arguments
    /// * `visited_nodes` - The visited nodes
    /// # Returns
    /// * `serde_json::Value` - The path in json format
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

    /// Reconstruction of the path and directions instructions from the visited nodes and edges
    /// # Arguments
    /// * `visited_nodes` - The visited nodes
    /// * `visited_edges` - The visited edges
    /// # Returns
    /// * `serde_json::Value` - The path and directions instructions in json format
    pub fn directions_instructions_and_path(
        &self,
        visited_nodes: &Vec<NodeId>,
        visited_edges: &Vec<Edge>,
    ) -> serde_json::Value {
        let mut path = Vec::new();
        let mut instructions: Vec<(String, f64, f64, f64)> = Vec::new();
        let mut total_distance = 0.0;

        let mut prev_edge = visited_edges.first().unwrap();

        let mut roundabout_exit_counter = 0;
        let mut is_prev_roundabout = false;

        let mut distance_since_last_instruction = 0.0;

        // iter over visited edges and add the nodes coordinates to the path
        for i in 0..visited_edges.len() {
            for node_id in &visited_edges[i].nodes_ids {
                let node = self.nodes.get(node_id).unwrap();
                path.push(json!({"latitude": node.lat(), "longitude": node.lon()}));
            }
            total_distance += visited_edges[i].distance_m;

            distance_since_last_instruction += visited_edges[i].distance_m;

            // Find way
            let way = self.ways.get(&visited_edges[i].way_id).unwrap();

            // get edge node lat and lon
            let edge_first_node = self.nodes.get(&visited_edges[i].from).unwrap();

            // print if way is intersection or roundabout
            if (way.tags.contains_key("highway") && way.tags["highway"] == "motorway_link")
                || (way.tags.contains_key("junction") && way.tags["junction"] == "roundabout")
            {
                if (is_prev_roundabout) {
                    roundabout_exit_counter += 1;
                } else {
                    roundabout_exit_counter = 1;
                }

                is_prev_roundabout = true;
            } else {
                if (is_prev_roundabout) {
                    let instruction = format!("Roundabout exit: {}", roundabout_exit_counter);

                    instructions.push((
                        instruction,
                        distance_since_last_instruction,
                        edge_first_node.lat(),
                        edge_first_node.lon(),
                    ));

                    distance_since_last_instruction = 0.0;

                    roundabout_exit_counter = 0;
                    is_prev_roundabout = false;
                }

                if (way.tags.contains_key("name")) {
                    if prev_edge.way_id != visited_edges[i].way_id {
                        let prev_way = self.ways.get(&prev_edge.way_id).unwrap();
                        if prev_way.tags.contains_key("name")
                            && prev_way.tags["name"] == way.tags["name"]
                        {
                            let instruction = format!("Continue on road {}", way.tags["name"]);
                            instructions.push((
                                instruction,
                                distance_since_last_instruction,
                                edge_first_node.lat(),
                                edge_first_node.lon(),
                            ));
                            distance_since_last_instruction = 0.0;
                        } else {
                            let prev_edge_last_node = self.nodes.get(&prev_edge.to).unwrap();
                            let prev_edge_first_node = self.nodes.get(&prev_edge.from).unwrap();
                            let current_edge_first_node =
                                self.nodes.get(&visited_edges[i].from).unwrap();
                            let current_edge_last_node =
                                self.nodes.get(&visited_edges[i].to).unwrap();

                            let prev_edge_angle = (prev_edge_last_node.lat()
                                - prev_edge_first_node.lat())
                            .atan2(prev_edge_last_node.lon() - prev_edge_first_node.lon())
                            .to_degrees();

                            let current_edge_angle = (current_edge_last_node.lat()
                                - current_edge_first_node.lat())
                            .atan2(current_edge_last_node.lon() - current_edge_first_node.lon())
                            .to_degrees();

                            let angle = current_edge_angle - prev_edge_angle;

                            let mut instruction = String::new();

                            // check the angle and determine if it is a left or right turn
                            if angle > 0.0 && angle < 180.0 {
                                //println!("Turn left to road {}", way.tags["name"]);
                                instruction = format!("Turn left to road {}", way.tags["name"]);
                            } else if angle < 0.0 && angle > -180.0 {
                                //println!("Turn right to road {}", way.tags["name"]);
                                instruction = format!("Turn right to road {}", way.tags["name"]);
                            } else if angle > 180.0 && angle < 360.0 {
                                //println!("Turn right to road {}", way.tags["name"]);
                                instruction = format!("Turn right to road {}", way.tags["name"]);
                            } else if angle < -180.0 && angle > -360.0 {
                                //println!("Turn left to road {}", way.tags["name"]);
                                instruction = format!("Turn left to road {}", way.tags["name"]);
                            } else {
                                //println!("Turn");
                                instruction = format!("Turn");
                            }
                            instructions.push((
                                instruction,
                                distance_since_last_instruction,
                                edge_first_node.lat(),
                                edge_first_node.lon(),
                            ));
                            distance_since_last_instruction = 0.0;
                        }
                    }
                }
            }

            prev_edge = &visited_edges[i];
        }

        // add the last node to the path
        let json_obj =
            json!({ "path": path, "instructions": instructions, "total_distance": total_distance });

        json_obj
    }

    /// Combine two paths
    /// # Arguments
    /// * `path_1` - The first path
    /// * `path_2` - The second path
    /// # Returns
    /// * `Vec<NodeId>` - The combined path
    pub fn combine_paths(&self, path_1: Vec<NodeId>, path_2: Vec<NodeId>) -> Vec<NodeId> {
        let mut combined_path = path_1.clone();

        // add path_2 to combined_path, but skip the first node
        for node_id in path_2.iter().skip(1) {
            combined_path.push(*node_id);
        }

        combined_path
    }

    /// Get the edges that start from a node or contain it
    /// # Arguments
    /// * `node_id` - The id of the node
    /// # Returns
    /// * `Vec<&Edge>` - A vector of references to the edges that start from the node or contain it
    pub fn get_edges_from_node_or_containing(&self, node_id: NodeId) -> Vec<&Edge> {
        let mut edges = Vec::new();

        for edge in &self.edges {
            if edge.from == node_id {
                edges.push(edge);
            }
        }

        if (edges.len() == 0) {
            for edge in &self.edges {
                for n_id in &edge.nodes_ids {
                    if node_id == *n_id {
                        edges.push(edge);
                    }
                }
            }
        }

        edges
    }

    /// Get the edges that start from a node
    /// # Arguments
    /// * `node_id` - The id of the node
    /// # Returns
    /// * `Vec<&Edge>` - A vector of references to the edges that start from the node
    pub fn get_edges_from_node(&self, node_id: NodeId) -> Vec<&Edge> {
        let mut edges = Vec::new();

        for edge in &self.edges {
            if edge.from == node_id {
                edges.push(edge);
            }
        }

        edges
    }

    /// Add an edge and the node where it starts to the graph
    /// # Arguments
    /// * `node_id` - The id of the node where the edge starts
    /// * `edge` - The edge to add
    pub fn add_edge_from_node(&mut self, node_id: NodeId, edge: Edge) {
        let edges = self.edges_from_node.entry(node_id).or_insert(Vec::new());
        edges.push(edge);
    }

    /// Get the edges that start from a node
    /// # Arguments
    /// * `node_id` - The id of the node
    /// # Returns
    /// * `&Vec<Edge>` - A reference to the vector of edges that start from the node
    pub fn get_edges_from_node_fast(&self, node_id: &NodeId) -> &Vec<Edge> {
        let edges = self.edges_from_node.get(node_id);

        edges.unwrap_or(&self.empty_edges)
    }

    /// Get the edges that end to a node
    /// # Arguments
    /// * `node_id` - The id of the node
    /// # Returns
    /// * `Vec<&Edge>` - A vector of references to the edges that end to the node
    pub fn get_edges_to_node(&self, node_id: NodeId) -> Vec<&Edge> {
        let mut edges = Vec::new();

        for edge in &self.edges {
            if edge.to == node_id {
                edges.push(edge);
            }
        }

        edges
    }

    /// calculate the radius of the circle that contains the given three points
    /// # Arguments
    /// * `node_1` - The first node
    /// * `node_2` - The second node
    /// * `node_3` - The third node
    /// # Returns
    /// * `f64` - The radius of the circle
    pub fn circle_radius(node_1: &Node, node_2: &Node, node_3: &Node) -> f64 {
        let lat_1 = node_1.lat();
        let lon_1 = node_1.lon();

        let lat_2 = node_2.lat();
        let lon_2 = node_2.lon();

        let lat_3 = node_3.lat();
        let lon_3 = node_3.lon();

        let a = (lat_1 - lat_2) * (lat_1 - lat_2) + (lon_1 - lon_2) * (lon_1 - lon_2);
        let b = (lat_2 - lat_3) * (lat_2 - lat_3) + (lon_2 - lon_3) * (lon_2 - lon_3);
        let c = (lat_3 - lat_1) * (lat_3 - lat_1) + (lon_3 - lon_1) * (lon_3 - lon_1);

        let s = 0.5 * (a + b + c);

        let area = (s * (s - a) * (s - b) * (s - c)).sqrt();

        let radius = a * b * c / (4.0 * area);

        radius.sqrt()
    }
}

/// Edge of the graph
/// # Fields
/// * `from` - The id of the node where the edge starts
/// * `to` - The id of the node where the edge ends
/// * `distance_m` - The distance of the edge in meters
/// * `weight` - The weight of the edge
/// * `time` - The time needed to traverse the edge
/// * `nodes_ids` - The ids of the nodes that the edge contains
/// * `way_id` - The id of the way that the edge belongs to
#[derive(Debug, Clone)]
pub struct Edge {
    pub from: NodeId,
    pub to: NodeId,
    pub distance_m: f64,
    pub weight: f64,
    pub time: f64,
    pub nodes_ids: Vec<NodeId>,
    pub way_id: WayId,
}

/// Edge implementation
impl Edge {
    /// Create a new edge
    /// # Arguments
    /// * `from` - The id of the node where the edge starts
    /// * `to` - The id of the node where the edge ends
    /// * `distance_m` - The distance of the edge in meters
    /// * `weight` - The weight of the edge
    /// * `time` - The time needed to traverse the edge
    /// * `nodes_ids` - The ids of the nodes that the edge contains
    /// * `way_id` - The id of the way that the edge belongs to
    pub fn new(
        from: NodeId,
        to: NodeId,
        distance_m: f64,
        weight: f64,
        time: f64,
        nodes_ids: Vec<NodeId>,
        way_id: WayId,
    ) -> Self {
        Edge {
            from,
            to,
            distance_m,
            weight,
            time,
            nodes_ids,
            way_id,
        }
    }
}

/// State of the graph
/// # Fields
/// * `node_id` - The id of the node
/// * `distance` - The distance of the node from the source node
#[derive(Debug, Clone)]
pub struct State {
    pub node_id: NodeId,
    pub distance: f64,
}

/// State implementation
impl State {
    /// Create a new state
    /// # Arguments
    /// * `node_id` - The id of the node
    /// * `distance` - The distance of the node from the source node
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
