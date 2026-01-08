RUST_LOG=info,rig=debug,rmcp=debug MAPS_API_KEY="" GEMINI_API_KEY="" cargo run

curl -X POST http://localhost:8080/chat \
     -H "Content-Type: application/json" \
     -d '{"prompt": "What is the weather like in London"}'
