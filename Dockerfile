# syntax=docker/dockerfile:1.6

FROM rust:1.83-alpine AS builder

RUN apk add --no-cache \
        git \
        build-base \
        cmake \
        pkgconfig \
        openssl-dev \
        openssl-libs-static \
        libx11-dev \
        libxrandr-dev \
        libxi-dev \
        libxinerama-dev \
        libxkbcommon-dev \
        wayland-dev \
        mesa-dev

WORKDIR /workspace

COPY . .

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
