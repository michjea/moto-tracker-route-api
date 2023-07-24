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

                //instructions.push(json!({"instruction": "Roundabout", "distance": edge.distance_m}));
                //println!("Roundabout");
                //println!("{:?}", way);

                // get previous edge last node id
                // git previous edge first node id
                let prev_edge_first_node_id = prev_edge.nodes_ids.first().unwrap();
                let prev_edge_last_node_id = prev_edge.nodes_ids.last().unwrap();
                //println!("Before roundabout first : {:?}", prev_edge_first_node_id);
                //println!("Before roundabout last : {:?}", prev_edge_last_node_id);

                // get the nodes of the way and there tags
                for node_id in &visited_edges[i].nodes_ids {
                    let node = self.nodes.get(node_id).unwrap();
                    //println!("{:?}", node);
                }

                // get next edge first node id
                let next_edge_first_node_id = visited_edges[i].nodes_ids.first().unwrap();
                // get next edge last node id
                let next_edge_last_node_id = visited_edges[i].nodes_ids.last().unwrap();
                //println!("After roundabout first: {:?}", next_edge_first_node_id);
                //println!("After roundabout last: {:?}", next_edge_last_node_id);
            } else {
                if (is_prev_roundabout) {
                    //println!("Roundabout exit: {}", roundabout_exit_counter);
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
                    //println!("Way name: {}", way.tags["name"]);

                    // check if prev_edge and current edge are on the same way
                    if prev_edge.way_id != visited_edges[i].way_id {
                        //instructions.push(json!({"instruction": "Turn", "distance": edge.distance_m}));
                        // check if name of prev_edge and current edge are the same
                        let prev_way = self.ways.get(&prev_edge.way_id).unwrap();
                        if prev_way.tags.contains_key("name")
                            && prev_way.tags["name"] == way.tags["name"]
                        {
                            //println!("Continue on road {}", way.tags["name"]);
                            let instruction = format!("Continue on road {}", way.tags["name"]);
                            instructions.push((
                                instruction,
                                distance_since_last_instruction,
                                edge_first_node.lat(),
                                edge_first_node.lon(),
                            ));
                            distance_since_last_instruction = 0.0;
                        } else {
                            // check the angle between last edge and current edge for determining the turn direction
                            let prev_edge_last_node = self.nodes.get(&prev_edge.to).unwrap();
                            let prev_edge_first_node = self.nodes.get(&prev_edge.from).unwrap();
                            let current_edge_first_node =
                                self.nodes.get(&visited_edges[i].from).unwrap();
                            let current_edge_last_node =
                                self.nodes.get(&visited_edges[i].to).unwrap();

                            // calculate the angle between the last edge and the current edge
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
                } else {
                    //println!("Way name: No name");
                    if (way.tags.contains_key("noname")) {
                        //println!("Way name: {}", way.tags["noname"]);
                    } else {
                        //println!("Way no name : {:?}", way.tags);
                    }
                }
            }

            prev_edge = &visited_edges[i];
        }

        // add the last node to the path
        let json_obj =
            json!({ "path": path, "instructions": instructions, "total_distance": total_distance });

        // print instructions
        //println!("Instructions: {:?}", instructions);

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

    // calculate the radius of the circle that contains the given three points
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

impl Edge {
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
