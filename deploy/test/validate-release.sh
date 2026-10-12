#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
VERSION="${1:-0.8.2}"

cd "$ROOT"

FAIL=0

check_bundle() {
    ARCH="$1"
    TAR="dist/monceda-edge-v${VERSION}-linux-${ARCH}.tar.gz"
    SHA="${TAR}.sha256"

    echo
    echo "== $ARCH =="

    if [ ! -f "$TAR" ]; then
        echo "FAIL: missing $TAR"
        FAIL=1
        return
    fi

    if [ ! -f "$SHA" ]; then
        echo "FAIL: missing $SHA"
        FAIL=1
        return
    fi

    echo "Bundle:"
    ls -lh "$TAR"

    echo
    echo "Checksum:"
    (
        cd dist
        shasum -a 256 -c "$(basename "$SHA")"
    )

    TMP="$(mktemp -d)"
    trap 'rm -rf "$TMP"' RETURN

    tar -xzf "$TAR" -C "$TMP"

    DIR="$TMP/monceda-edge-v${VERSION}-linux-${ARCH}"

    if [ ! -d "$DIR" ]; then
        echo "FAIL: expected directory missing: $DIR"
        FAIL=1
        return
    fi

    REQUIRED=(
        monceda-edge
        install.sh
        monceda-edge.service
        monceda-edge.toml
        health-check.sh
        README.md
        RELEASE
    )

    for FILE in "${REQUIRED[@]}"; do
        if [ ! -e "$DIR/$FILE" ]; then
            echo "FAIL: missing bundle file: $FILE"
            FAIL=1
        fi
    done

    INFO="$(file "$DIR/monceda-edge")"
    echo "$INFO"

    if ! grep -q "ELF" <<<"$INFO"; then
        echo "FAIL: binary is not Linux ELF"
        FAIL=1
    fi

    case "$ARCH" in
        x86_64)
            grep -q "x86-64" <<<"$INFO" || {
                echo "FAIL: expected x86-64 ELF"
                FAIL=1
            }
            ;;
        aarch64)
            grep -qi "aarch64" <<<"$INFO" || {
                echo "FAIL: expected aarch64 ELF"
                FAIL=1
            }
            ;;
    esac

    bash -n "$DIR/install.sh"
    bash -n "$DIR/health-check.sh"

    echo "Bundle structure: OK"

    rm -rf "$TMP"
    trap - RETURN
}

check_bundle x86_64
check_bundle aarch64

echo
if [ "$FAIL" -ne 0 ]; then
    echo "Release validation FAILED."
    exit 1
fi

echo "Release validation PASSED."
