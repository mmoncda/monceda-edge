#!/usr/bin/env bash
set -euo pipefail

# Keep macOS AppleDouble/xattr metadata out of Linux release archives.
export COPYFILE_DISABLE=1
export COPY_EXTENDED_ATTRIBUTES_DISABLE=1

VERSION="${1:-0.8.1}"
ARCH="${2:-}"
BINARY="${3:-}"

if [ -z "$ARCH" ]; then
    echo "Usage:"
    echo "  $0 <version> <x86_64|aarch64> <linux-binary>"
    exit 1
fi

case "$ARCH" in
    x86_64|aarch64)
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

if [ -z "$BINARY" ] || [ ! -f "$BINARY" ]; then
    echo "Linux binary not found: $BINARY"
    exit 1
fi

FILE_INFO="$(file "$BINARY")"

if ! printf '%s' "$FILE_INFO" | grep -q "ELF"; then
    echo "Refusing to package non-Linux binary:"
    echo "  $FILE_INFO"
    exit 1
fi

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
DIST="$ROOT/dist"
NAME="monceda-edge-v${VERSION}-linux-${ARCH}"
STAGE="$DIST/$NAME"

rm -rf "$STAGE"
mkdir -p "$STAGE"

install -m 0755 \
    "$BINARY" \
    "$STAGE/monceda-edge"

install -m 0644 \
    "$ROOT/deploy/linux/monceda-edge.service" \
    "$STAGE/monceda-edge.service"

install -m 0644 \
    "$ROOT/deploy/linux/monceda-edge.staging.toml" \
    "$STAGE/monceda-edge.toml"

install -m 0755 \
    "$ROOT/deploy/linux/health-check.sh" \
    "$STAGE/health-check.sh"

install -m 0644 \
    "$ROOT/deploy/linux/README.md" \
    "$STAGE/README.md"

cp \
    "$ROOT/deploy/release/install-bundle.sh" \
    "$STAGE/install.sh"

chmod +x "$STAGE/install.sh"

cat > "$STAGE/RELEASE" <<META
name=monceda-edge
version=$VERSION
os=linux
arch=$ARCH
META

(
    cd "$DIST"
    tar --no-xattrs -czf "$NAME.tar.gz" "$NAME"
)

if command -v shasum >/dev/null 2>&1; then
    (
        cd "$DIST"
        shasum -a 256 "$NAME.tar.gz" > "$NAME.tar.gz.sha256"
    )
elif command -v sha256sum >/dev/null 2>&1; then
    (
        cd "$DIST"
        sha256sum "$NAME.tar.gz" > "$NAME.tar.gz.sha256"
    )
else
    echo "Warning: no SHA-256 utility found."
fi

echo
echo "Bundle created:"
echo "  $DIST/$NAME.tar.gz"

if [ -f "$DIST/$NAME.tar.gz.sha256" ]; then
    echo
    echo "Checksum:"
    cat "$DIST/$NAME.tar.gz.sha256"
fi
