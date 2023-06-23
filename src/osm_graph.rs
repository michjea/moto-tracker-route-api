use osmpbfreader::objects::{Node, Way, NodeId, WayId};
use std::collections::{BinaryHeap, HashMap};

pub struct OSMGraph {
    pub nodes: HashMap<NodeId, Node>,
    pub ways: HashMap<WayId, Way>,
    pub edges: Vec<Edge>,
}

impl OSMGraph {
    pub fn new() -> Self {
        OSMGraph {
            nodes: HashMap::new(),
            ways: HashMap::new(),
            edges: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: &Node) -> () {
        self.nodes.insert(node.id, node.clone());
    }

    pub fn add_way(&mut self, way: &Way) -> () {
        self.ways.insert(way.id, way.clone());
    }

    pub fn get_node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn get_way_count(&self) -> usize {
        self.ways.len()
    }
}

pub struct Edge {
    pub from: i64,
    pub to: i64,
    pub distance: f64,
    pub nodes_ids: Vec<i64>,
}

impl Edge {
    pub fn new(from: i64, to: i64, distance: f64, nodes_ids: Vec<i64>) -> Self {
        Edge {
            from: from,
            to: to,
            distance: distance,
            nodes_ids: nodes_ids,
        }
    }
}