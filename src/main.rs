mod graph;
mod osm_reader;
mod route_calculation;
mod osm_graph;

use graph::Graph;
use osm_reader::OSMReader;
use std::env;
use graph::Node;
use graph::Edge;
//use osm4routing;
//use osm4routing::Edge;
//use osm4routing::Node;
//use osm4routing::OsmObj::Node;
//use osm4routing::osm4routing::models::Node;
//use osm4routing::models::Node;
use route_calculation::bidirectional_dijkstra_path;

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

fn main() {
    println!("Hello, world!");

    let current_dir = env::current_dir().expect("Failed to get current directory");
    let file_path = current_dir.join("data").join("switzerland-latest.osm.pbf");
    //let file_path = current_dir.join("data").join("luxembourg-latest.osm.pbf");
    
    let mut osm_reader = OSMReader::new(file_path.to_str().unwrap().to_string());
    let mut graph = osm_reader.build_graph();

    /*// calculate dijkstra
    let start_time = std::time::Instant::now();

    // coordinates of Neuchâtel
    let coords_neuchatel = (46.992979, 6.931933);

    // coordinates of Saignelégier
    let coords_saignelegier = (47.25, 7.0);

    let start_node = {
        let graph = &graph; // Create a new scope to borrow graph immutably
        graph.get_nearest_node(coords_neuchatel.0, coords_neuchatel.1)
    };

    println!("Start node: {:?}", start_node);
    
    let end_node = {
        let graph = &graph; // Create a new scope to borrow graph immutably
        graph.get_nearest_node(coords_saignelegier.0, coords_saignelegier.1)
    };
    
    println!("End node: {:?}", end_node);

    let result = bidirectional_dijkstra_path(&graph, start_node.unwrap(), end_node.unwrap());
     
    let duration = start_time.elapsed();

    println!("Time elapsed in expensive_function() is: {:?}", duration);

    println!("Result: {:?}", result);

    // osm4routing result is a Result<(Vec<Node>, Vec<Edge>), Error>

    //let result = osm4routing::read(file_path.to_str().unwrap());*/
}

// get route from A to B
#[get("/route/{from}/{to}")]
async fn route(path: web::Path<(String, String)>)-> impl Responder {//, data: web::Data<AppState>) -> impl Responder {
    let (from, to) = path.into_inner();
    // prnit data nodes and edges length
    HttpResponse::Ok().body(format!("From {} to {}", from, to))
}

#[actix_web::main]
async fn start() -> std::io::Result<()> { // change to main to start server
    // start server
    HttpServer::new(|| App::new()
        //.app_data(web::Data::new(app_state.clone()))
        .service(route))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

/*struct AppState {
    app_name: String,
    edges: Vec<Edge>,
    nodes: Vec<Node>,
}*/