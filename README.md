# DHCP Config Editor

A web-based editor for ISC DHCP `dhcpd.conf` files, built with Rust and Axum.

## Prerequisites

### Install Rust on macOS

```bash
# Using Homebrew
brew install rustup-init
rustup-init

# Follow the prompts, then restart your shell
source ~/.zshrc

# Verify installation
rustc --version
cargo --version
```

## Build & Run

```bash
# Navigate to project
cd dhcp-editor

# Build
cargo build --release

# Run with default config path on port 8080
cargo run --release

# Or specify custom path and port:
cargo run --release -- --config /path/to/dhcpd.conf --port 9000
```

## Usage

Open your browser at `http://localhost:8080` (or your specified port).

### CLI Options

| Flag | Description | Default |
|------|-------------|---------|
| `-c, --config` | Path to dhcpd.conf | `/etc/dhcp/dhcpd.conf` |
| `-p, --port` | Server port | `8080` |

## Features

- Structured editor for subnets, hosts, shared-networks, groups
- Add/remove global options and declarations
- Save changes back to dhcpd.conf

---

**Note**: No authentication - use behind a firewall only.
