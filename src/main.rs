use axum::{
    extract::{Path, Query},
    http::StatusCode,
    routing::{get},
    Json, Router,
};
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // Create a shared HTTP client wrapped in Arc for thread safety
    let client = Arc::new(Client::new());

    // Define routes
    let app = Router::new()
        .route("/", get(home_handler))
        .route("/ships", get({
            let client = Arc::clone(&client);
            move |query| get_swapi_handler(query, "starships", client)
        }))
        .route("/characters", get({
            let client = Arc::clone(&client);
            move |query| get_swapi_handler(query, "people", client)
        }))
        .fallback(get(catch_all_handler));

    // Start the server on port 5004
    let addr = SocketAddr::from(([0, 0, 0, 0], 5004));
    println!("Server running at http://{}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

// Home route handler
async fn home_handler() -> Json<Value> {
    Json(json!({
        "_welcome_note": "Star Wars API Wrapper from SWAPI. Use the available APIs to access information about Star Wars starships and characters.",
        "available_apis": {
            "/ships": "Access information about Star Wars starships. Use the 'search' query parameter to filter results.",
            "/characters": "Access information about Star Wars characters. Use the 'search' query parameter to filter results.",
            "/": "Home page listing all available APIs.",
            "search_query": "Use the 'search' query parameter to filter results based on the name of the starship or character."
        }
    }))
}

// Handler for SWAPI requests (e.g., /ships and /characters)
async fn get_swapi_handler(
    Query(params): Query<HashMap<String, String>>,
    endpoint: &'static str,
    client: Arc<Client>,
) -> (StatusCode, Json<Value>) {
    let search_term = params.get("search").cloned().unwrap_or_default();
    let url = if search_term.is_empty() {
        format!("https://swapi.py4e.com/api/{}/", endpoint)
    } else {
        format!("https://swapi.py4e.com/api/{}/?search={}", endpoint, search_term)
    };

    match client.get(&url).send().await {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(data) => (StatusCode::OK, Json(data)),
                    Err(err) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": format!("Failed to parse SWAPI response: {}", err) })),
                    ),
                }
            } else {
                (
                    response.status(),
                    Json(json!({ "error": format!("SWAPI returned an error: {}", response.status()) })),
                )
            }
        }
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to connect to SWAPI: {}", err) })),
        ),
    }
}

// Fallback handler for undefined routes
async fn catch_all_handler(Path(_): Path<String>) -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(json!({ "message": "Not active API" })),
    )
}
