//mod graph;
mod osm_graph;
mod osm_reader;
mod route_calculation;

//use graph::Graph;
use osm_graph::Edge;
use osm_reader::OSMReader;
use osmpbfreader::objects::{Node, NodeId, Tags, Way, WayId};
use std::env;
//use graph::Node;
//use graph::Edge;
//use osm4routing;
//use osm4routing::Edge;
//use osm4routing::Node;
//use osm4routing::OsmObj::Node;
//use osm4routing::osm4routing::models::Node;
//use osm4routing::models::Node;
use osm_graph::OSMGraph;
use route_calculation::bidirectional_dijkstra_path_2;
use route_calculation::dijkstra;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

fn main() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let file_path = current_dir.join("data").join("switzerland-latest.osm.pbf");
    //let file_path = current_dir.join("data").join("luxembourg-latest.osm.pbf");

    let mut osm_reader = OSMReader::new(file_path.to_str().unwrap().to_string());
    let mut graph = osm_reader.build_graph();
    //let mut graph = OSMGraph::new();

    // calculate dijkstra
    let start_time = std::time::Instant::now();

    // coordinates of Neuchâtel
    let coords_neuchatel = (46.992979, 6.931933);

    let coords_poms_moi = (47.2715023, 6.9877472);
    let coords_poms_2 = (47.26944444, 6.98444444);

    // coordinates of Saignelégier
    let coords_saignelegier = (47.25, 7.0);

    let start_node = {
        let graph = &graph; // Create a new scope to borrow graph immutably
        graph
            .get_nearest_node(coords_poms_moi.0, coords_poms_moi.1)
            .unwrap()
    };

    println!("Start node: {:?}", start_node);

    let end_node = {
        let graph = &graph; // Create a new scope to borrow graph immutably
        graph
            .get_nearest_node(coords_neuchatel.0, coords_neuchatel.1)
            .unwrap()
    };

    println!("End node: {:?}", end_node);

    // Tests

    // get path with node 717405927 //intersection église // ok
    // 984721184 // intersection début chemin sur la Velle
    // 984723399 // milieu chemin sur la Velle
    // 984722674 // milieu chemin sur la Velle
    // 984722038 // fin chemin sur la Velle

    /*let node_1 = NodeId(984722674);

    let edges_from_node_1 = graph.get_edges_from_node(node_1);

    println!("Edges from node 1: {:?}", edges_from_node_1);*/

    /*let mut graph = OSMGraph::new();

    graph.add_node(&Node{id: NodeId(717405927), decimicro_lat: 0, decimicro_lon: 0, tags: Tags::new()});
    graph.add_node(&Node{id: NodeId(984721184), decimicro_lat: 1, decimicro_lon: 0, tags: Tags::new()});
    graph.add_node(&Node{id: NodeId(984723399), decimicro_lat: 2, decimicro_lon: 0, tags: Tags::new()});
    graph.add_node(&Node{id: NodeId(984722674), decimicro_lat: 3, decimicro_lon: 0, tags: Tags::new()});
    graph.add_node(&Node{id: NodeId(984722038), decimicro_lat: 3, decimicro_lon: 0, tags: Tags::new()});

    graph.add_edge(Edge::new(NodeId(717405927), NodeId(984721184), 1.0, 1.0, vec![NodeId(717405927), NodeId(984721184)]));
    graph.add_edge(Edge::new(NodeId(984721184), NodeId(717405927), 1.0, 1.0, vec![NodeId(984721184), NodeId(717405927)]));

    graph.add_edge(Edge::new(NodeId(984721184), NodeId(984723399), 1.0, 1.0, vec![NodeId(984721184), NodeId(984723399)]));
    graph.add_edge(Edge::new(NodeId(984723399), NodeId(984721184), 1.0, 1.0, vec![NodeId(984723399), NodeId(984721184)]));

    graph.add_edge(Edge::new(NodeId(984723399), NodeId(984722674), 1.0, 1.0, vec![NodeId(984723399), NodeId(984722674)]));
    graph.add_edge(Edge::new(NodeId(984722674), NodeId(984723399), 1.0, 1.0, vec![NodeId(984722674), NodeId(984723399)]));

    graph.add_edge(Edge::new(NodeId(984722674), NodeId(984722038), 3.0, 3.0, vec![NodeId(984722674), NodeId(984722038)]));
    graph.add_edge(Edge::new(NodeId(984722038), NodeId(984722674), 3.0, 3.0, vec![NodeId(984722038), NodeId(984722674)]));

    let start_node = NodeId(717405927);
    let end_node = NodeId(984722038);*/

    let result = dijkstra(&graph, &start_node, &end_node);

    let duration = start_time.elapsed();

    println!(
        "Time elapsed in bidirectional_dijkstra_path_2() is: {:?}",
        duration
    );

    println!("Result: {:?}", result);

    // osm4routing result is a Result<(Vec<Node>, Vec<Edge>), Error>

    //let result = osm4routing::read(file_path.to_str().unwrap());
}

// get route from A to B
#[get("/route/{from}/{to}")]
async fn route(path: web::Path<(String, String)>, data: web::Data<AppState>) -> impl Responder {
    //, data: web::Data<AppState>) -> impl Responder {
    // print request
    println!("Request: {:?}", path);

    let (from, to) = path.into_inner();

    //HttpResponse::Ok().body(format!("Path: "));
    HttpResponse::Ok().body(format!("From {} to {}", from, to))
}

#[actix_web::main]
async fn start() -> std::io::Result<()> {
    // change to main to start server

    let current_dir = env::current_dir().expect("Failed to get current directory");
    let file_path = current_dir.join("data").join("switzerland-latest.osm.pbf");
    //let file_path = current_dir.join("data").join("luxembourg-latest.osm.pbf");

    let mut osm_reader = OSMReader::new(file_path.to_str().unwrap().to_string());
    let graph = osm_reader.build_graph();

    println!("Graph built");

    let app_state = AppState {
        app_name: String::from("OSM4Routing"),
        graph: graph,
    };

    print!("Starting server...");

    // start server
    HttpServer::new(move || App::new().data(app_state.clone()).service(route))
        //.bind("|| App::new().data(app_state.clone()).service(route))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

#[derive(Clone)]
struct AppState {
    app_name: String,
    graph: OSMGraph,
}
