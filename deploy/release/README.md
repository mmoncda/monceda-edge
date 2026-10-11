# Monceda Edge Release Bundles

Supported Linux architectures:

- x86_64
- aarch64

Bundle format:

monceda-edge-vVERSION-linux-ARCH.tar.gz

Each bundle contains:

- monceda-edge
- monceda-edge.service
- monceda-edge.toml
- install.sh
- health-check.sh
- README.md
- RELEASE

## Install

Extract:

tar -xzf monceda-edge-vVERSION-linux-ARCH.tar.gz

Enter directory:

cd monceda-edge-vVERSION-linux-ARCH

Install:

./install.sh

Start:

sudo systemctl start monceda-edge

Enable at boot:

sudo systemctl enable monceda-edge

Logs:

sudo journalctl -u monceda-edge -f
