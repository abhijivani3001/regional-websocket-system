# Regional WebSocket System

[![Rust](https://img.shields.io/badge/Rust-1.80+-000000?logo=rust&logoColor=white)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

Multi-region **WebSocket** backend designed for low-latency, horizontally scalable real-time applications.  
Each geographic region runs an independent WebSocket server. Clients connect to the nearest / lowest-latency region.

## Quick Start (Local Development)

### 1. Run multiple regional backend instances

Open separate terminals and run one command per region:

```bash
# US East
APP__REGION__NAME=us-east APP__SERVER__PORT=8080 cargo run
```

```bash
# Europe West
APP__REGION__NAME=eu-west APP__SERVER__PORT=8081 cargo run
```

```bash
# Asia Pacific South
APP__REGION__NAME=ap-south APP__SERVER__PORT=8082 cargo run
```

Useful variants:

```bash
# More logging
RUST_LOG=debug,tracing=info cargo run

# Release build (much faster after first compile)
cargo run --release
```

### 2. Start the frontend

```bash
cd frontend
npm install
npm run start
```

Frontend usually available at:
http://localhost:5173 with Vite

### 3. Expected behavior

The frontend should:

- Try connecting to one or more endpoints (`ws://localhost:8080/ws`, etc.)
- Show connection status per region
- Display latency / ping / region name
- Allow sending test messages

Look in the browser console / UI for connection logs.

## Project Layout

```
.
├── Cargo.toml
├── src/
│   ├── main.rs
│   └── ...
└── ...
├── frontend/
│   ├── package.json
│   ├── src/
│   └── vite.config.ts
|   └── ...
└── README.md
```
