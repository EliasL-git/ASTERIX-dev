#!/bin/sh
set -euo pipefail

DISPLAY="${DISPLAY:-:99}"
GEOMETRY="${ASTERIX_GEOMETRY:-1280x720x24}"
VNC_PORT="${VNC_PORT:-5901}"
NOVNC_PORT="${NOVNC_PORT:-8080}"

cleanup() {
    kill $(jobs -p) 2>/dev/null || true
}

trap cleanup EXIT

Xvfb "$DISPLAY" -screen 0 "$GEOMETRY" >/tmp/xvfb.log 2>&1 &

# Allow X server a moment to come online
sleep 2

fluxbox >/tmp/fluxbox.log 2>&1 &

x11vnc -display "$DISPLAY" \
    -rfbport "$VNC_PORT" \
    -forever \
    -shared \
    -nopw \
    -quiet >/tmp/x11vnc.log 2>&1 &

websockify --web /usr/share/novnc/ "$NOVNC_PORT" "localhost:${VNC_PORT}" >/tmp/websockify.log 2>&1 &

export DISPLAY
exec /opt/asterix/bin/asterix "$@"
