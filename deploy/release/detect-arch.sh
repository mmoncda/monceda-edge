#!/usr/bin/env bash
set -euo pipefail

ARCH="$(uname -m)"

case "$ARCH" in
    x86_64|amd64)
        echo "x86_64"
        ;;
    aarch64|arm64)
        echo "aarch64"
        ;;
    *)
        echo "Unsupported architecture: $ARCH" >&2
        exit 1
        ;;
esac
