#!/usr/bin/env bash
set -euo pipefail

echo "== OS =="
uname -a
echo

echo "== Architecture =="
uname -m
echo

echo "== Ports 80/443 =="
if command -v ss >/dev/null 2>&1; then
  ss -ltnp | grep -E ':(80|443)\b' || echo "ports 80/443 currently free"
else
  echo "ss unavailable"
fi
echo

echo "== Required commands =="

for cmd in \
  curl \
  openssl \
  systemctl \
  useradd \
  install
do
  if command -v "$cmd" >/dev/null 2>&1; then
    echo "OK   $cmd"
  else
    echo "MISS $cmd"
  fi
done

echo
echo "== systemd =="

if systemctl --version >/dev/null 2>&1; then
  systemctl --version | head -1
else
  echo "systemd unavailable"
fi

echo
echo "Host preflight complete."
