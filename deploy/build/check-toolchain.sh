#!/usr/bin/env bash
set -euo pipefail

echo "== Rust =="
rustc --version
cargo --version

echo
echo "== Zig =="
zig version

echo
echo "== cargo-zigbuild =="

if command -v cargo-zigbuild >/dev/null 2>&1; then
    echo "installed: $(command -v cargo-zigbuild)"
elif cargo zigbuild --help >/dev/null 2>&1; then
    echo "installed via cargo subcommand"
else
    echo "cargo-zigbuild not found"
    exit 1
fi

echo
echo "== Installed Rust targets =="

TARGETS="$(rustup target list --installed)"
printf '%s\n' "$TARGETS"

for TARGET in \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu
do
    if ! printf '%s\n' "$TARGETS" | grep -qx "$TARGET"; then
        echo
        echo "Missing Rust target: $TARGET"
        exit 1
    fi
done

echo
echo "Toolchain ready."
