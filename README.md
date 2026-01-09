# Cityscape - A Gemini-powered API Server

Cityscape is a Rust-based API server designed to demonstrate the integration of a Gemini AI agent with external tools, specifically leveraging Google Cloud services for tracing and potentially Google Maps for location-based functionalities. This project is built using the `axum` web framework, `rig` for agent orchestration, and `rmcp` for tool communication.

## Features

*   **Gemini AI Agent Integration**: Interact with a powerful AI agent capable of understanding and responding to natural language prompts.
*   **Tool-Calling Capabilities**: The Gemini agent can utilize external tools, enabling it to perform actions like retrieving real-time weather information (as hinted by the "weather_agent" in the code).
*   **Google Cloud Trace Observability**: Integrated OpenTelemetry with Google Cloud Trace provides end-to-end distributed tracing for monitoring and debugging API requests.
*   **Web Server**: A lightweight and efficient API server built with `axum` for handling incoming requests.

## Getting Started

These instructions will get you a copy of the project up and running on your local machine for development and testing purposes.

### Prerequisites

*   **Rust & Cargo**: Ensure you have Rust and Cargo installed. You can install them via `rustup`:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
*   **Google Cloud Project**: A Google Cloud Project is required for Google Cloud Trace. Ensure you have authenticated your gcloud CLI.
*   **Google Maps API Key**: An API key for Google Maps is required.

### Setup

1.  **Clone the repository**:
    ```bash
    git clone https://github.com/your-username/cityscape.git
    cd cityscape
    ```

2.  **Environment Variables**: Set the `MAPS_API_KEY` environment variable. This key is used for interacting with Google Maps services.
    ```bash
    export MAPS_API_KEY="YOUR_GOOGLE_MAPS_API_KEY"
    ```

3.  **Install Dependencies**:
    ```bash
    cargo build
    ```
    _Note: You might encounter dependency resolution issues due to `opentelemetry` ecosystem versioning. The project's `Cargo.toml` attempts to align these, but manual intervention might be needed in some cases._

### Running the Server

To start the API server:

```bash
cargo run
```

The server will typically run on `http://localhost:8080`.

## API Endpoints

### `/chat` (POST)

This endpoint allows you to send prompts to the Gemini AI agent.

*   **Method**: `POST`
*   **Body**: JSON object with a `prompt` field:
    ```json
    {
        "prompt": "What is the weather in London?"
    }
    ```
*   **Response**: JSON object with an `answer` field containing the agent's response:
    ```json
    {
        "answer": "The weather in London is sunny with a temperature of 15°C."
    }
    ```

## Contributing

Contributions are welcome! If you find a bug or have a feature request, please open an issue. If you'd like to contribute code, please fork the repository and submit a pull request.

## License

This project is licensed under the MIT License - see the LICENSE file for details.