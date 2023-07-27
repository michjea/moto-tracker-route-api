//mod graph;
mod osm_graph;
mod osm_reader;
mod route_calculation;
use actix_cors::Cors;
use actix_web::{get, post, web, web::ServiceConfig, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use log::{info, warn};
use osm_graph::OSMGraph;
use osm_reader::OSMReader;
use osmpbfreader::objects::{Node, NodeId, Tags, Way, WayId};
use route_calculation::dijkstra;
use route_calculation::generate_random_loop;
use serde_json::{json, Map, Value};
use shuttle_actix_web::ShuttleActixWeb;
use std::env;
use std::fs::File;
use std::io::copy;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::AsyncReadExt;
use tokio::io::AsyncWriteExt;

async fn start() {
    let current_dir = env::current_dir().expect("Failed to get current directory");
    let file_path = current_dir.join("data").join("switzerland-latest.osm.pbf");

    let mut osm_reader = OSMReader::new(file_path.to_str().unwrap().to_string());
    let mut graph = osm_reader.build_graph().await;

    let start_time = std::time::Instant::now();

    let coords_neuchatel = (46.992979, 6.931933);
    let coords_poms_moi = (47.2715023, 6.9877472);
    let coords_poms_2 = (47.26944444, 6.98444444);
    let coords_saignelegier = (47.25, 7.0);

    let timer = std::time::Instant::now();

    let start_node = {
        let graph = &graph; // Create a new scope to borrow graph immutably
        graph
            .get_nearest_node(coords_poms_moi.0, coords_poms_moi.1)
            .unwrap()
    };

    info!("Time to get nearest node: {:?}", timer.elapsed());

    info!("Start node: {:?}", start_node);

    let timer = std::time::Instant::now();

    let end_node = {
        let graph = &graph; // Create a new scope to borrow graph immutably
        graph
            .get_nearest_node(coords_neuchatel.0, coords_neuchatel.1)
            .unwrap()
    };

    info!("Time to get nearest node: {:?}", timer.elapsed());

    info!("End node: {:?}", end_node);

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
}

/// Parameters for the calculate-route endpoint
/// # Fields
/// * from_lat: latitude of the starting point
/// * from_lon: longitude of the starting point
/// * to_lat: latitude of the ending point
/// * to_lon: longitude of the ending point
/// Example: http://localhost:8080/calculate-route/?from_lat=47.2715023&from_lon=6.9877472&to_lat=47.25&to_lon=7.0
#[derive(serde::Deserialize, Debug)]
struct CalculateRouteParams {
    from_lat: f64,
    from_lon: f64,
    to_lat: f64,
    to_lon: f64,
}

/// Parameters for the calculate-loop endpoint
/// # Fields
/// * from_lat: latitude of the starting point
/// * from_lon: longitude of the starting point
/// * distance: distance of the loop in meters
/// Example: http://localhost:8080/calculate-loop/?from_lat=47.2715023&from_lon=6.9877472&distance=1000
#[derive(serde::Deserialize, Debug)]
struct CalculateLoopParams {
    from_lat: f64,
    from_lon: f64,
    distance: f64, // distance in meters
}

/// Calculate a route between two points
/// # Parameters
/// * from_lat: latitude of the starting point
/// * from_lon: longitude of the starting point
/// * to_lat: latitude of the ending point
/// * to_lon: longitude of the ending point
/// # Returns
/// * A JSON object containing the route
/// # Example
/// http://localhost:8080/calculate-route/?from_lat=47.2715023&from_lon=6.9877472&to_lat=47.25&to_lon=7.0
#[get("/calculate-route/")]
async fn calculate_route(
    params: web::Query<CalculateRouteParams>,
    data: web::Data<AppState>,
) -> impl Responder {
    let start_node = {
        let graph = &data.graph;
        graph
            .get_nearest_node(params.from_lat, params.from_lon)
            .unwrap()
    };

    let end_node = {
        let graph = &data.graph;
        graph
            .get_nearest_node(params.to_lat, params.to_lon)
            .unwrap()
    };

    let (result, edges) = dijkstra(&data.graph, &start_node, &end_node);

    if result.is_none() {
        return HttpResponse::BadRequest().body("No route found");
    } else {
        let result = result.unwrap();

        let test = data
            .graph
            .directions_instructions_and_path(&result, &edges.unwrap());

        let mut path = vec![];

        path.push(test);

        HttpResponse::Ok().json(path)
    }
}

/// Calculate a loop starting from a point
/// # Parameters
/// * from_lat: latitude of the starting point
/// * from_lon: longitude of the starting point
/// * distance: distance of the loop in meters
/// # Returns
/// * A JSON object containing the route
/// # Example
/// http://localhost:8080/calculate-loop/?from_lat=47.2715023&from_lon=6.9877472&distance=1000
#[get("/calculate-loop/")]
async fn calculate_loop(
    params: web::Query<CalculateLoopParams>,
    data: web::Data<AppState>,
) -> impl Responder {
    info!("Request: {:?}", params);

    let start_node = {
        let graph = &data.graph; // Create a new scope to borrow graph immutably
        graph
            .get_nearest_node(params.from_lat, params.from_lon)
            .unwrap()
    };

    let result = generate_random_loop(params.distance, &data.graph, &start_node);

    let mut path = vec![];

    for i in 0..result.len() - 1 {
        let node_ids = &result[i].0;
        let edges_ids = &result[i].1;
        let res = data
            .graph
            .directions_instructions_and_path(&node_ids, &edges_ids);

        path.push(res);
    }

    HttpResponse::Ok().json(path)
}

/// Main function
/// # Returns
/// * A ShuttleActixWeb object
/// # Example
#[shuttle_runtime::main]
async fn actix_web(
    #[shuttle_static_folder::StaticFolder(folder = "static")] static_folder: PathBuf,
) -> ShuttleActixWeb<impl FnOnce(&mut ServiceConfig) + Send + Clone + 'static> {
    let url = "https://download.geofabrik.de/europe/switzerland-latest.osm.pbf";

    info!("Downloading file...");
    println!("Downloading file...");

    let response = reqwest::get(url).await;

    let date_time = Utc::now();
    let timestamp: i64 = date_time.timestamp();

    let name: String = format!("swiss-{}.pbf", timestamp);

    let file_path = static_folder.join(&name);

    info!("Saving file...");
    println!("Saving file...");

    match response {
        Ok(mut res) => {
            let mut file = File::create(&file_path).expect("Failed to create file");
            match res.bytes().await {
                Ok(bytes) => {
                    info!("Writing file...");
                    file.write_all(&bytes).expect("Failed to write to file");
                    info!("File downloaded successfully");
                }
                Err(_) => warn!("Failed to save file"),
            }
        }
        Err(_) => warn!("Failed to download file"),
    }

    info!("File saved !");
    println!("File saved !");

    let mut osm_reader = OSMReader::new(file_path.to_str().unwrap().to_string());
    let graph = osm_reader.build_graph().await;

    info!("Graph built");
    println!("Graph built");

    let app_state = AppState {
        app_name: String::from("OSM4Routing"),
        graph: graph,
    };

    let app_data = web::Data::new(app_state);

    info!("Starting server...");
    println!("Starting server...");

    let config = move |cfg: &mut ServiceConfig| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        cfg.service(
            web::scope("")
                .service(calculate_loop)
                .service(calculate_route)
                .app_data(app_data.clone())
                .wrap(Arc::new(cors)),
        );

        /*cfg.app_data(app_data.clone())
        .service(route)
        .service(calculate_loop)
        .service(calculate_route)
        .wrap(Arc::new(cors));*/
    };

    Ok(config.into())
}

/// Main function
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
        graph: graph.await,
    };

    let app_data = web::Data::new(app_state);

    println!("Starting server...");

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .app_data(app_data.clone())
            .service(calculate_route)
    })
    .bind(("127.0.0.1", 4242))?
    .run()
    .await
}

/// Application state
/// # Fields
/// * app_name: name of the application
/// * graph: OSMGraph
#[derive(Clone)]
struct AppState {
    app_name: String,
    graph: OSMGraph,
}
