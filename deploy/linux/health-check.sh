#!/usr/bin/env bash

set -euo pipefail

BASE="${1:-https://127.0.0.1}"

echo "== Health =="
curl -ksS "$BASE/_edge/health"
echo

echo
echo "== ACME =="
curl -ksS "$BASE/_edge/acme"
echo

echo
echo "== Readiness =="
curl -ksS "$BASE/_edge/readiness"
echo

echo
echo "== Onboarding =="
curl -ksS "$BASE/_edge/onboarding"
echo

echo
echo "== Network =="
curl -ksS "$BASE/_edge/network-check"
echo

echo
echo "== ACME Activation =="
curl -ksS "$BASE/_edge/acme-activation"
echo
