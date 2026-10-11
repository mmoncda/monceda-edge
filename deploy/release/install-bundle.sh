#!/usr/bin/env bash
set -euo pipefail

EXPECTED_ARCH=""

if [ -f ./RELEASE ]; then
    EXPECTED_ARCH="$(
        awk -F= '$1=="arch" {print $2}' ./RELEASE
    )"
fi

MACHINE="$(uname -m)"

case "$MACHINE" in
    x86_64|amd64)
        MACHINE="x86_64"
        ;;
    aarch64|arm64)
        MACHINE="aarch64"
        ;;
    *)
        echo "Unsupported Linux architecture: $MACHINE"
        exit 1
        ;;
esac

if [ -n "$EXPECTED_ARCH" ] && [ "$MACHINE" != "$EXPECTED_ARCH" ]; then
    echo "Architecture mismatch."
    echo "Bundle: $EXPECTED_ARCH"
    echo "Machine: $MACHINE"
    exit 1
fi

if [ ! -x ./monceda-edge ]; then
    echo "monceda-edge binary missing."
    exit 1
fi

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
    ./monceda-edge \
    /usr/local/bin/monceda-edge

sudo install \
    -o root \
    -g monceda-edge \
    -m 0640 \
    ./monceda-edge.toml \
    /etc/monceda-edge/monceda-edge.toml

sudo install \
    -o root \
    -g root \
    -m 0644 \
    ./monceda-edge.service \
    /etc/systemd/system/monceda-edge.service

sudo systemctl daemon-reload

echo
echo "Monceda Edge bundle installed."
echo
/usr/local/bin/monceda-edge version
echo
echo "Service is not started automatically."
echo
echo "Next:"
echo "  sudo systemctl start monceda-edge"
echo "  sudo journalctl -u monceda-edge -f"
