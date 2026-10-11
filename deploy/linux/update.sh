#!/usr/bin/env bash

set -euo pipefail

BINARY="${1:-./target/release/monceda-edge}"

if [ ! -f "$BINARY" ]; then
    echo "Binary not found: $BINARY"
    exit 1
fi

echo "Updating Monceda Edge..."

sudo install \
    -o root \
    -g root \
    -m 0755 \
    "$BINARY" \
    /usr/local/bin/monceda-edge

sudo systemctl restart monceda-edge

sleep 2

sudo systemctl --no-pager --full status monceda-edge
