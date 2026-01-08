use axum::{extract::State, routing::post, Json, Router};
use rig::{
    client::{CompletionClient, ProviderClient},
    providers::gemini::{self},
    completion::Prompt,
};
use rmcp::{
    service::ServiceExt,
    transport::streamable_http_client::StreamableHttpClientTransportConfig
};

use reqwest::header::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::env;
use tracing_subscriber::{fmt, Registry, EnvFilter, prelude::*};

#[derive(Deserialize)]
struct ChatRequest { prompt: String }

#[derive(Serialize)]
struct ChatResponse { answer: String }

struct AppState {
    agent: rig::agent::Agent<gemini::completion::CompletionModel>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    Registry::default()
        .with(fmt::layer().pretty())
        .with(EnvFilter::from_default_env())
        .init();

    let maps_api_key = env::var("MAPS_API_KEY")
        .expect("MAPS_API_KEY environment variable must be set");

    let mut headers = HeaderMap::new();
    let header_value = HeaderValue::from_str(&maps_api_key)
        .expect("Invalid characters in MAPS_API_KEY");
    headers.insert("X-Goog-Api-Key", header_value);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let mut config = StreamableHttpClientTransportConfig::default();
    config.uri = Arc::from("https://mapstools.googleapis.com/mcp");
    let transport = rmcp::transport::StreamableHttpClientTransport::with_client(
        client,
        config,
    );
    
    let mcp_client = ().serve(transport).await?; 
    let tools = mcp_client.list_tools(Default::default()).await?.tools;

    let gemini_client = gemini::Client::from_env();
    let weather_agent = gemini_client.agent("gemini-2.5-flash")
        .preamble("Use the weather tool to get a summary of current weather conditions in a city to provide the image with up to date information.")
        .rmcp_tools(tools, mcp_client.peer().to_owned())
        .build();

    let shared_state = Arc::new(AppState {
        agent: weather_agent
    });

    let app = Router::new()
        .route("/chat", post(handle_chat))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("API Server running on http://localhost:8080");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn handle_chat(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    // The agent handles the tool-calling logic internally
    let answer = state.agent.prompt(&payload.prompt).await
        .unwrap_or_else(|e| format!("Error: {}", e));

    Json(ChatResponse { answer })
}
