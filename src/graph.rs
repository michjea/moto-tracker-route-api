use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};

pub struct Graph {
    pub nodes: HashMap<i64, Node>,
    ways: HashMap<i64, Way>,
    pub edges: Vec<Edge>,
    //relations: HashMap<i64, Relation>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            ways: HashMap::new(),
            edges: Vec::new(),
            //relations: HashMap::new(),
        }
    }

    // Get nearest node to given coordinates
    pub fn get_nearest_node(&self, lat: f64, lon: f64) -> Option<&Node> {
        let mut nearest_node: Option<&Node> = None;
        let mut min_distance = std::f64::MAX;

        for node in self.nodes.values() {
            let distance = node.get_distance(lat, lon);

            if distance < min_distance {
                min_distance = distance;
                nearest_node = Some(node);
            }
        }

        nearest_node
    }

    pub fn add_node(&mut self, node: Node) -> () {
        self.nodes.insert(node.id, node);
    }

    pub fn add_way(&mut self, way: Way) -> () {
        self.ways.insert(way.id, way);
    }

    pub fn add_edge(&mut self, edge: Edge) -> () {
        self.edges.push(edge);
    }

    /*pub fn add_relation(&mut self, relation: Relation) -> () {
        self.relations.insert(relation.id, relation);
    }*/

    pub fn get_node(&self, id: i64) -> Option<&Node> {
        self.nodes.get(&id)
    }

    pub fn get_node_ids(&self) -> Vec<i64> {
        self.nodes.keys().map(|id| *id).collect()
    }

    pub fn get_way(&self, id: i64) -> Option<&Way> {
        self.ways.get(&id)
    }

    /*pub fn get_relation(&self, id: i64) -> Option<&Relation> {
        self.relations.get(&id)
    }*/

    pub fn get_nodes(&self) -> &HashMap<i64, Node> {
        &self.nodes
    }

    pub fn remove_node(&mut self, id: i64) -> () {
        self.nodes.remove(&id);
    }

    pub fn get_ways(&self) -> &HashMap<i64, Way> {
        &self.ways
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

    pub fn get_distance(&self, from: i64, to: i64) -> f64 {
        let from_node = self.get_node(from).unwrap();
        let to_node = self.get_node(to).unwrap();

        from_node.get_distance(to_node.lat, to_node.lon)
    }

    pub fn get_edges_from_node(&self, node_id: i64) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|edge| edge.from == node_id)
            .collect()
    }

    pub fn get_edges_to_node(&self, node_id: i64) -> Vec<&Edge> {
        self.edges
            .iter()
            .filter(|edge| edge.to == node_id)
            .collect()
    }

    pub fn reconstruct_path(&self, prev_nodes: &HashMap<i64, i64>, node_id: i64) -> Vec<i64> {
        let mut path = vec![node_id];
        let mut current = node_id;

        while let Some(&prev) = prev_nodes.get(&current) {
            path.push(prev);
            current = prev;
        }

        path.reverse();
        path
    }

    pub fn combine_paths(&self, forward_path: Vec<i64>, backward_path: Vec<i64>) -> Vec<i64> {
        let mut combined_path = forward_path;
        combined_path.extend(backward_path.into_iter().skip(1));
        combined_path
    }
}

#[derive(Debug, Default)]
pub struct Edge {
    pub from: i64, // Source
    pub to: i64,   // Target
    way_id: i64,
    pub distance: f64,
    pub weight: f64,
    pub nodes_ids: Vec<i64>,
}

impl Edge {
    pub fn new(
        from: i64,
        to: i64,
        way_id: i64,
        distance: f64,
        weight: f64,
        nodes_ids: Vec<i64>,
    ) -> Self {
        Edge {
            from,
            to,
            way_id,
            distance,
            weight,
            nodes_ids,
        }
    }
}

#[derive(Debug, Default)]
pub struct Node {
    pub id: i64,
    pub lat: f64,
    pub lon: f64,

    tags: HashMap<String, String>,
}

impl Node {
    pub fn new(id: i64, lat: f64, lon: f64, tags: HashMap<String, String>) -> Self {
        Node {
            id: id,
            lat: lat,
            lon: lon,
            tags: tags,
        }
    }

    // Get distance between two nodes
    pub fn get_distance(&self, lat: f64, lon: f64) -> f64 {
        let x = self.lat - lat;
        let y = self.lon - lon;

        (x * x + y * y).sqrt()
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }

    pub fn get_lat(&self) -> f64 {
        self.lat
    }

    pub fn get_lon(&self) -> f64 {
        self.lon
    }
}

#[derive(Debug, Default, Clone)]
pub struct Way {
    id: i64,
    node_ids: Vec<i64>,
    tags: HashMap<String, String>,
}

impl Way {
    pub fn new(id: i64, node_ids: Vec<i64>, tags: HashMap<String, String>) -> Self {
        Way {
            id: id,
            node_ids: node_ids,
            tags: tags,
        }
    }

    pub fn get_node_ids(&self) -> &Vec<i64> {
        &self.node_ids
    }

    pub fn get_id(&self) -> i64 {
        self.id
    }
}

#[derive(Debug)]
pub struct Relation {
    id: i64,
    members: Vec<Member>,

    tags: HashMap<String, String>,
}

#[derive(Debug)]
pub struct Member {
    id: i64,
    role: String,
    member_type: String,
}

pub struct State {
    pub node_id: i64,
    pub distance: f64,
}

impl State {
    pub fn new(node_id: i64, distance: f64) -> Self {
        State { node_id, distance }
    }
}

impl PartialEq for State {
    fn eq(&self, other: &Self) -> bool {
        self.distance.eq(&other.distance)
    }
}

impl Eq for State {}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.distance.partial_cmp(&other.distance)
    }
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        self.distance.partial_cmp(&other.distance).unwrap()
    }
}
