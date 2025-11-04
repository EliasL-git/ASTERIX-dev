# ASTERIX Browser - Copilot Instructions

## Project Overview

ASTERIX is an experimental Rust-powered browser shell that focuses on a modern, Firefox-inspired experience while keeping the codebase lean. The project is in early development with a working desktop shell, textual renderer, asynchronous networking, and containerized demo environment.

## Architecture

The project follows a modular workspace architecture with three primary crates:

### Core Crates

- **asterix-core**: Networking primitives, tab metadata, and document fetch pipeline
  - Built on `reqwest` + `tokio` for async HTTP operations
  - Provides core data structures: `TabId`, `PageRequest`, `PageResponse`, `TabSnapshot`
  - Handles document fetching and parsing with `scraper`

- **asterix-browser**: Background runtime and orchestration layer
  - Multi-threaded tokio executor
  - Message passing for navigation requests
  - Coordinates between core services and UI
  - Uses `parking_lot` for synchronization primitives

- **asterix-ui**: Desktop shell built with eframe/egui
  - Provides tab controls, URL bar, and page preview
  - Main binary (`asterix`) located at `crates/asterix-ui/src/main.rs`
  - Currently renders pages as plain text while rendering engine is under construction

## Development Guidelines

### Toolchain Requirements

- Rust 1.82 or newer (edition 2021)
- Platform libraries for OpenGL when running on Linux:
  - `mesa`, `libx11`, `wayland` support

### Build and Run

```bash
# Check code
cargo check

# Run the browser UI
cargo run -p asterix-ui

# Build release version
cargo build --release -p asterix-ui

# Run tests
cargo test
```

### Coding Conventions

1. **Error Handling**: Use `anyhow` for application errors, `thiserror` for library errors
2. **Async Runtime**: All async code uses `tokio` runtime
3. **Serialization**: Use `serde` with derive macros for data structures
4. **Logging**: Use `tracing` for instrumentation and logging
5. **Workspace Dependencies**: Prefer workspace-level dependency declarations in root `Cargo.toml`
6. **License**: All code is Apache-2.0 licensed

### Code Patterns

- Use `parking_lot` RwLock/Mutex for thread-safe shared state
- Leverage `#[instrument]` from tracing for function tracing
- Follow Rust 2021 edition idioms
- Keep public APIs well-documented with doc comments
- Use structured error types with `thiserror`

### Testing

- Unit tests should be co-located with the code they test
- Integration tests go in the `tests/` directory (if present)
- Run tests with `cargo test`
- Ensure all public APIs have basic test coverage

## Docker and Containerization

The project includes Docker support for containerized demos:

- **Dockerfile**: Multi-stage build with Alpine base
- **docker-compose.yml**: Simplified deployment
- Container includes Xvfb, Fluxbox, VNC, and noVNC for graphical access
- Access via `http://localhost:8080` (noVNC) or VNC on port `5901`

### Docker Build

```bash
# Using Docker Compose (recommended)
docker-compose up --build

# Using Docker directly
docker build -t asterix .
docker run -it --rm -p 8080:8080 -p 5901:5901 asterix
```

## Roadmap Context

Future development areas (don't implement unless specifically asked):
- HTML/CSS layout engine integration (evaluating Servo components)
- JavaScript runtime embedding (QuickJS or SpiderMonkey)
- GPU-accelerated compositor using `wgpu`
- Enhanced sandboxing and process isolation

## Current State

- ✅ Basic browser shell with tab management
- ✅ URL bar and navigation
- ✅ Asynchronous HTTP fetching
- ✅ Textual page preview
- ✅ Containerized demo environment
- 🚧 HTML/CSS rendering (planned)
- 🚧 JavaScript execution (planned)
- 🚧 GPU rendering pipeline (planned)

## Important Notes

- The rendering engine is intentionally minimal (text-only) at this stage
- Focus on clean architecture and extensibility over features
- Performance matters, but correctness and maintainability come first
- Keep dependencies lean and well-justified
