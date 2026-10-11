# Monceda Edge Linux Deployment

## Layout

Binary:

/usr/local/bin/monceda-edge

Configuration:

/etc/monceda-edge/monceda-edge.toml

Persistent state:

/var/lib/monceda-edge

ACME cache:

/var/lib/monceda-edge/acme

systemd service:

/etc/systemd/system/monceda-edge.service

## Build

Build the Linux release binary:

cargo build --release

## Install

sudo ./deploy/linux/install.sh

## Start

sudo systemctl start monceda-edge

## Enable at boot

sudo systemctl enable monceda-edge

## Logs

sudo journalctl -u monceda-edge -f

## Restart

sudo systemctl restart monceda-edge

## Stop

sudo systemctl stop monceda-edge

## Status

sudo systemctl status monceda-edge

## Health

./deploy/linux/health-check.sh

## ACME safety

Keep:

environment = "staging"

until staging issuance has successfully completed.

Do not switch to production ACME until:

- DNS points to the node.
- TCP 80 is publicly reachable.
- TCP 443 is publicly reachable.
- /_edge/acme-activation reports runtime_ready=true.
- Staging issuance succeeds.
- Certificate cache persists across restart.
