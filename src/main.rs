//mod graph;
mod osm_graph;
mod osm_reader;
mod route_calculation;

use actix_web::cookie::time::macros::datetime;
use osm_graph::Edge;
use osm_graph::OSMGraph;
use osm_reader::OSMReader;
use osmpbfreader::objects::{Node, NodeId, Tags, Way, WayId};
//use route_calculation::bidirectional_dijkstra_path_2;
use route_calculation::dijkstra;
use std::env;
use std::path::PathBuf;

use actix_web::{get, post, web, web::ServiceConfig, App, HttpResponse, HttpServer, Responder};
use shuttle_actix_web::ShuttleActixWeb;

use chrono::Utc;
use std::fs::File;
use std::io::copy;
use std::io::Write;

use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

fn start() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let file_path = current_dir.join("data").join("switzerland-latest.osm.pbf");

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

    let timer = std::time::Instant::now();

    let start_node = {
        let graph = &graph; // Create a new scope to borrow graph immutably
        graph
            .get_nearest_node(coords_poms_moi.0, coords_poms_moi.1)
            .unwrap()
    };

    println!("Time to get nearest node: {:?}", timer.elapsed());

    println!("Start node: {:?}", start_node);

    let timer = std::time::Instant::now();

    let end_node = {
        let graph = &graph; // Create a new scope to borrow graph immutably
        graph
            .get_nearest_node(coords_neuchatel.0, coords_neuchatel.1)
            .unwrap()
    };

    println!("Time to get nearest node: {:?}", timer.elapsed());

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

    let timer = std::time::Instant::now();

    let result = dijkstra(&graph, &start_node, &end_node);

    /*let duration = start_time.elapsed();

    println!(
        "Time elapsed in bidirectional_dijkstra_path_2() is: {:?}",
        duration
    );*/

    println!("Time to calculate dijkstra: {:?}", timer.elapsed());

    //println!("Result: {:?}", result);
}

// get route from A to B
#[get("/route/{from}/{to}")]
async fn route(path: web::Path<(String, String)>, data: web::Data<AppState>) -> impl Responder {
    //, data: web::Data<AppState>) -> impl Responder {
    // print request
    println!("TEST");
    println!("Request: {:?}", path);

    let (from, to) = path.into_inner();

    //HttpResponse::Ok().body(format!("Path: "));
    HttpResponse::Ok().body(format!("From {} to {}", from, to))
}

#[derive(serde::Deserialize, Debug)]
struct CalculateRouteParams {
    from_lat: f64,
    from_lon: f64,
    to_lat: f64,
    to_lon: f64,
}

#[get("/route/")]
async fn calculate_route(
    params: web::Query<CalculateRouteParams>,
    data: web::Data<AppState>,
) -> impl Responder {
    // print request
    println!("Request: {:?}", params);

    let start_node = {
        let graph = &data.graph; // Create a new scope to borrow graph immutably
        graph
            .get_nearest_node(params.from_lat, params.from_lon)
            .unwrap()
    };

    let end_node = {
        let graph = &data.graph; // Create a new scope to borrow graph immutably
        graph
            .get_nearest_node(params.to_lat, params.to_lon)
            .unwrap()
    };

    let result = dijkstra(&data.graph, &start_node, &end_node);

    HttpResponse::Ok().json(result)
}

#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_static_folder::StaticFolder(folder = "static")] static_folder: PathBuf,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    // change to main to start server

    //let current_dir = env::current_dir().expect("Failed to get current directory");
    //let file_path = current_dir.join("data").join("switzerland-latest.osm.pbf");
    //let file_path = current_dir.join("data").join("luxembourg-latest.osm.pbf");

    // use the static folder fot path
    // let file_path = static_folder.join("switzerland-latest.osm.pbf");

    let url = "https://download.geofabrik.de/europe/switzerland-latest.osm.pbf";

    println!("Downloading file...");

    let response = reqwest::get(url).await;

    let date_time = Utc::now();
    let timestamp: i64 = date_time.timestamp();

    let name: String = format!("swiss-{}.pbf", timestamp);

    let file_path = static_folder.join(&name);
    println!("Saving file...");

    match response {
        Ok(mut res) => {
            let mut file = File::create(file_path).expect("Failed to create file");
            match res.bytes().await {
                Ok(bytes) => {
                    println!("Writing file...");
                    file.write_all(&bytes).expect("Failed to write to file");
                    println!("File downloaded successfully");
                }
                Err(_) => println!("Failed to save file"),
            }
        }
        Err(_) => println!("Failed to download file"),
    }

    println!("File saved !");

    let file_path = static_folder.join(name);

    let mut osm_reader = OSMReader::new(file_path.to_str().unwrap().to_string());
    let graph = osm_reader.build_graph();

    println!("Graph built");

    let app_state = AppState {
        app_name: String::from("OSM4Routing"),
        graph: graph,
    };

    let app_data = web::Data::new(app_state);

    println!("Starting server...");

    // start server
    let config = move |cfg: &mut ServiceConfig| {
        cfg.service(route)
            .service(calculate_route)
            .app_data(app_data.clone());
    };

    Ok(config.into())
}

//#[actix_web::main]
async fn main_actix() -> std::io::Result<()> {
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

    let app_data = web::Data::new(app_state);

    println!("Starting server...");

    // start server
    HttpServer::new(move || {
        App::new()
            .app_data(app_data.clone())
            .service(route)
            .service(calculate_route)
    })
    //.bind("|| App::new().data(app_state.clone()).service(route))
    .bind(("127.0.0.1", 4242))?
    .run()
    .await
}

#[derive(Clone)]
struct AppState {
    app_name: String,
    graph: OSMGraph,
}
