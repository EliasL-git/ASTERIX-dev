# syntax=docker/dockerfile:1.6

FROM rust:1.83-alpine AS builder

RUN apk add --no-cache \
        git \
        build-base \
        cmake \
        pkgconfig \
        libx11-dev \
        libxrandr-dev \
        libxi-dev \
        libxinerama-dev \
        libxkbcommon-dev \
        wayland-dev \
        mesa-dev

WORKDIR /workspace

# Copy cargo configuration for optimized builds
COPY .cargo .cargo

# Copy only dependency manifests first for better layer caching
COPY Cargo.toml Cargo.toml
COPY crates/asterix-core/Cargo.toml crates/asterix-core/Cargo.toml
COPY crates/asterix-browser/Cargo.toml crates/asterix-browser/Cargo.toml
COPY crates/asterix-ui/Cargo.toml crates/asterix-ui/Cargo.toml

# Create dummy source files to cache dependencies
RUN mkdir -p crates/asterix-core/src && \
    echo "fn main() {}" > crates/asterix-core/src/lib.rs && \
    mkdir -p crates/asterix-browser/src && \
    echo "fn main() {}" > crates/asterix-browser/src/lib.rs && \
    mkdir -p crates/asterix-ui/src && \
    echo "fn main() {}" > crates/asterix-ui/src/main.rs

# Build dependencies only - this layer will be cached unless Cargo.toml changes
RUN cargo build --release -p asterix-ui

# Now copy the actual source code
COPY crates crates

# Touch files to trigger rebuild of actual source (not dependencies)
RUN touch crates/asterix-core/src/lib.rs && \
    touch crates/asterix-browser/src/lib.rs && \
    touch crates/asterix-ui/src/main.rs

# Build the actual application (dependencies are already cached)
RUN cargo build --release -p asterix-ui

FROM alpine:3.20

RUN apk add --no-cache \
        libstdc++ \
        mesa-dri-gallium \
        mesa-gl \
        libx11 \
        libxrender \
        libxrandr \
        libxinerama \
        libxcursor \
        libxcb \
        libxkbcommon \
        fontconfig \
        ttf-dejavu \
        xvfb \
        fluxbox \
        x11vnc \
        websockify \
        novnc \
        tini \
        bash

WORKDIR /opt/asterix

COPY --from=builder /workspace/target/release/asterix ./bin/asterix
COPY docker/entrypoint.sh ./scripts/entrypoint.sh
RUN chmod +x ./scripts/entrypoint.sh

ENV DISPLAY=:99 \
    VNC_PORT=5901 \
    NOVNC_PORT=8080

EXPOSE 8080 5901

ENTRYPOINT ["/sbin/tini", "--", "/opt/asterix/scripts/entrypoint.sh"]
CMD []
