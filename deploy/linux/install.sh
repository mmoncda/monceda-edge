#!/usr/bin/env bash

set -euo pipefail

BINARY="${1:-./target/release/monceda-edge}"
CONFIG="${2:-./deploy/linux/monceda-edge.staging.toml}"
SERVICE="${3:-./deploy/linux/monceda-edge.service}"

if [ ! -f "$BINARY" ]; then
    echo "Binary not found: $BINARY"
    exit 1
fi

if [ ! -f "$CONFIG" ]; then
    echo "Config not found: $CONFIG"
    exit 1
fi

if [ ! -f "$SERVICE" ]; then
    echo "Service file not found: $SERVICE"
    exit 1
fi

echo "Installing Monceda Edge..."

if ! id monceda-edge >/dev/null 2>&1; then
    sudo useradd \
        --system \
        --home /var/lib/monceda-edge \
        --shell /usr/sbin/nologin \
        monceda-edge
fi

sudo install -d \
    -o monceda-edge \
    -g monceda-edge \
    -m 0750 \
    /var/lib/monceda-edge

sudo install -d \
    -o monceda-edge \
    -g monceda-edge \
    -m 0750 \
    /var/lib/monceda-edge/acme

sudo install -d \
    -o monceda-edge \
    -g monceda-edge \
    -m 0750 \
    /var/lib/monceda-edge/fallback

sudo install -d \
    -o root \
    -g monceda-edge \
    -m 0750 \
    /etc/monceda-edge

sudo install \
    -o root \
    -g root \
    -m 0755 \
    "$BINARY" \
    /usr/local/bin/monceda-edge

sudo install \
    -o root \
    -g monceda-edge \
    -m 0640 \
    "$CONFIG" \
    /etc/monceda-edge/monceda-edge.toml

sudo install \
    -o root \
    -g root \
    -m 0644 \
    "$SERVICE" \
    /etc/systemd/system/monceda-edge.service

sudo systemctl daemon-reload

echo
echo "Monceda Edge installed."
echo
echo "Binary:"
echo "  /usr/local/bin/monceda-edge"
echo
echo "Config:"
echo "  /etc/monceda-edge/monceda-edge.toml"
echo
echo "State:"
echo "  /var/lib/monceda-edge"
echo
echo "Service is NOT automatically started."
echo
echo "Validate first:"
echo "  sudo systemctl start monceda-edge"
echo "  sudo journalctl -u monceda-edge -f"
