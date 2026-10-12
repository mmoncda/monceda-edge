#!/usr/bin/env bash
set -euo pipefail

BUNDLE_DIR="${1:-.}"

cd "$BUNDLE_DIR"

echo "== System =="
uname -a
echo

echo "== Architecture =="
ARCH="$(uname -m)"
echo "$ARCH"
echo

echo "== Binary =="
file ./monceda-edge
./monceda-edge version
echo

echo "== Bundle files =="
for FILE in \
    monceda-edge \
    install.sh \
    monceda-edge.service \
    monceda-edge.toml \
    health-check.sh \
    README.md \
    RELEASE
do
    test -e "$FILE" || {
        echo "FAIL: missing $FILE"
        exit 1
    }
done

echo "Bundle contents: OK"
echo

echo "== Shell validation =="
bash -n ./install.sh
bash -n ./health-check.sh
echo "Shell scripts: OK"
echo

echo "== Systemd unit syntax =="
if command -v systemd-analyze >/dev/null 2>&1; then
    systemd-analyze verify ./monceda-edge.service
    echo "systemd unit: OK"
else
    echo "systemd-analyze unavailable; skipped"
fi

echo
echo "== Config references =="

grep -q '/var/lib/monceda-edge' monceda-edge.toml && \
    echo "state path: OK"

grep -q '/var/lib/monceda-edge/acme' monceda-edge.toml && \
    echo "ACME path: OK"

grep -q 'port = 80' monceda-edge.toml && \
    echo "HTTP 80: OK"

grep -q 'port = 443' monceda-edge.toml && \
    echo "HTTPS 443: OK"

echo
echo "Linux bundle smoke test PASSED."
