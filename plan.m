ASTERIX development roadmap

1. Deliver a lightweight Rust prototype of the ASTERIX browser shell.
	- Cargo workspace with core runtime, UI shell, orchestration layer, and CLI launcher.
	- Async networking pipeline using `reqwest`/`tokio` with tab metadata management.
	- Desktop shell (Firefox-inspired) powered by `eframe`/`egui` for quick iteration.

2. Provide an accessible cloud-hosted demo path.
	- Docker image based on Alpine that compiles ASTERIX in release mode.
	- Runtime container wiring Xvfb, Fluxbox, x11vnc, and noVNC for browser-in-browser previews.
	- Entry script handling display startup, cleanup, and delegating to the ASTERIX launcher.

3. Future feature milestones.
	- Integrate a real rendering engine (Servo or `wgpu` compositor) and HTML parser.
	- Embed a JavaScript runtime and strengthen sandboxing/process isolation.
	- Expand automated tests, telemetry, and continuous delivery pipelines.