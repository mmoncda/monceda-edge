#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="${1:-0.9.0}"

X86_TARGET="x86_64-unknown-linux-gnu"
ARM_TARGET="aarch64-unknown-linux-gnu"

cd "$ROOT"

command -v zig >/dev/null 2>&1 || {
    echo "zig not found"
    exit 1
}

if ! command -v cargo-zigbuild >/dev/null 2>&1; then
    if ! cargo zigbuild --help >/dev/null 2>&1; then
        echo "cargo-zigbuild not found"
        exit 1
    fi
fi

echo
echo "== Monceda Edge Linux Builds =="
echo "Version: $VERSION"

echo
echo "Building Linux x86_64..."

cargo zigbuild \
    --release \
    --target "${X86_TARGET}.2.28"

echo
echo "Building Linux aarch64..."

cargo zigbuild \
    --release \
    --target "${ARM_TARGET}.2.28"

X86_BIN="$ROOT/target/$X86_TARGET/release/monceda-edge"
ARM_BIN="$ROOT/target/$ARM_TARGET/release/monceda-edge"

for BIN in "$X86_BIN" "$ARM_BIN"; do
    if [ ! -f "$BIN" ]; then
        echo "Missing binary:"
        echo "  $BIN"
        exit 1
    fi

    INFO="$(file "$BIN")"

    if ! printf '%s\n' "$INFO" | grep -q "ELF"; then
        echo "Expected Linux ELF binary:"
        echo "  $INFO"
        exit 1
    fi
done

echo
echo "Linux binaries built successfully:"
file "$X86_BIN"
file "$ARM_BIN"

echo
echo "Creating release bundles..."

"$ROOT/deploy/release/build-bundle.sh" \
    "$VERSION" \
    x86_64 \
    "$X86_BIN"

"$ROOT/deploy/release/build-bundle.sh" \
    "$VERSION" \
    aarch64 \
    "$ARM_BIN"

echo
echo "Release artifacts:"
find "$ROOT/dist" \
    -maxdepth 1 \
    -type f \
    -name "monceda-edge-v${VERSION}-linux-*" \
    -print
