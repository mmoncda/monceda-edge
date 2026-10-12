#!/usr/bin/env bash
set -euo pipefail

HOST="${1:-127.0.0.1}"
HTTPS_PORT="${2:-443}"

echo "== Monceda Edge Public Node Check =="
echo "Host: $HOST"
echo "HTTPS port: $HTTPS_PORT"
echo

echo "== HTTPS health =="
curl -kfsS \
  "https://${HOST}:${HTTPS_PORT}/_edge/health"
echo
echo

echo "== Readiness =="
curl -kfsS \
  "https://${HOST}:${HTTPS_PORT}/_edge/readiness"
echo
echo

echo "== Routes =="
curl -kfsS \
  "https://${HOST}:${HTTPS_PORT}/_edge/routes"
echo
echo

echo "== ACME =="
curl -kfsS \
  "https://${HOST}:${HTTPS_PORT}/_edge/acme"
echo
echo

echo "== Network =="
curl -kfsS \
  "https://${HOST}:${HTTPS_PORT}/_edge/network-check"
echo
echo

echo "Public node API check PASSED."
