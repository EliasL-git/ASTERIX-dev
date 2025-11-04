# ASTERIX Browser

ASTERIX is an experimental Rust-powered browser shell that focuses on a modern, Firefox-inspired experience while keeping the codebase lean. The current milestone delivers a working desktop shell with a textual renderer, asynchronous networking, and a containerised demo environment.

## Architecture Snapshot

- `asterix-core`: networking primitives, tab metadata, and document fetch pipeline built on `reqwest` + `tokio`.
- `asterix-browser`: background runtime with a multi-threaded tokio executor and message passing for navigation requests.
- `asterix-ui`: desktop shell built with `eframe`/`egui`, offering tab controls, URL bar, and a textual page preview.
- `asterix-cli`: launcher binary that wires tracing, runtime, and UI together.

## Getting Started

```bash
cargo run -p asterix-ui
```

The preview window opens with a blank tab. Enter a URL in the toolbar and press `Enter` or click `Go` to fetch a page. Responses are rendered as plain text while the rendering engine is under construction.

### Recommended Toolchain

- Rust 1.82 or newer (edition 2021)
- Platform libraries for OpenGL (`mesa`, `libx11`, `wayland`) when running on Linux

## Containerised Demo

A Docker image is provided for browser-on-the-go demos. It starts an Alpine base with Xvfb, Fluxbox, VNC, and noVNC so the full graphical shell can be exercised from any modern browser—no CLI experience required.

```bash
docker build -t asterix .
docker run -it --rm -p 8080:8080 -p 5901:5901 asterix
```

Once running, open `http://localhost:8080` in a local browser to access the noVNC console (a Firefox-like GUI). You can also connect with a VNC client on port `5901` if you prefer.

## Roadmap Highlights

- Integrate an HTML/CSS layout engine (evaluate Servo components).
- Embed a JavaScript runtime (QuickJS or SpiderMonkey bindings).
- Replace textual preview with GPU accelerated compositor using `wgpu`.
- Expand sandboxing and process isolation story (bubblewrap/firejail integration).

Contributions and architectural feedback are welcome as the ASTERIX runtime matures.