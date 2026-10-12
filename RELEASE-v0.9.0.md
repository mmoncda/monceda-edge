# Monceda Edge v0.9.0

## Goal

Public-node readiness.

## Planned validation

- Real Linux host installation
- systemd start
- systemd restart
- reboot persistence
- HTTP port 80
- HTTPS port 443
- HTTP -> HTTPS redirect
- public DNS resolution
- ACME staging issuance
- ACME certificate persistence
- HTTPS health endpoint
- readiness endpoint
- route validation
- upstream failure behavior
- fallback TLS behavior
- basic security hardening

## Release gate

Production ACME must not be enabled until staging issuance
and public reachability tests pass.
