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

use opentelemetry::{global, KeyValue};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter
};
use opentelemetry_sdk::resource::Resource;
use opentelemetry_gcloud_trace::GcpCloudTraceExporterBuilder;

#[derive(Deserialize, Debug)]
struct ChatRequest { prompt: String }

#[derive(Serialize)]
struct ChatResponse { answer: String }

struct AppState {
    agent: rig::agent::Agent<gemini::completion::CompletionModel>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let maps_api_key = env::var("MAPS_API_KEY")
        .expect("MAPS_API_KEY environment variable must be set");
    let _project_id = env::var("PROJECT_ID")
        .expect("PROJECT_ID environment variable must be set");

    // 1. Configure the Google Cloud Trace exporter
    let resource = Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", "cityscape")
        ])
        .build();
    let exporter = GcpCloudTraceExporterBuilder::for_default_project_id()
        .await?
        .with_resource(resource);
    // 2. Create a TracerProvider with the GCP exporter
    let tracer_provider = exporter.create_provider().await?;
    // 3. Set the global tracer provider for the opentelemetry crate
    // this ensures any part of the application, including 3P libs
    // can get a tracer instance and create spans without explicitly passing
    // the tracer_provider everywhere.
    global::set_tracer_provider(tracer_provider.clone());

    // 4. Obtain a tracer instance from the global TracerProvider
    // the string passed here is referred to as the instrumentation scope name
    // which helps identify the source of the telemetry data.
    let tracer = global::tracer("weather-agent");
    // Each layer will get the same tracing information and act on it
    // EnvFilter is the main filter for the entire pipeline
    tracing_subscriber::registry()
        // RUST_LOG env variable with value like "info,rig=trace,rmcp=debug"
        // Alternatively, use EnvFilter::new("info,rig=trace,rmcp=debug")
        .with(EnvFilter::from_default_env())
        .with(fmt::layer().compact()) //printing to console
        .with(tracing_opentelemetry::layer().with_tracer(tracer)) //off to GCP
        .init();

    let mut headers = HeaderMap::new();
    let header_value = HeaderValue::from_str(&maps_api_key).unwrap();
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

    let client = gemini::Client::from_env();
    let weather_agent = client.agent("gemini-2.5-flash")
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

#[tracing::instrument(name = "process_api_call", skip(state, payload))]
async fn handle_chat(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatRequest>,
) -> Json<ChatResponse> {
    let answer = state.agent.prompt(&payload.prompt).await
        .unwrap_or_else(|e| format!("Error: {}", e));

    Json(ChatResponse { answer })
}
