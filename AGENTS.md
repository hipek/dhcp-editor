# AGENTS.md - DHCP Editor

A web-based editor for ISC DHCP `dhcpd.conf` files, built with Rust and Axum.

## Project Structure

```
dhcp-editor/
├── src/
│   ├── main.rs      # Entry point, CLI parsing, server setup
│   ├── models.rs    # Data structures (DhcpConfig, Subnet, Host, etc.)
│   ├── parser.rs    # dhcpd.conf parsing and serialization
│   └── handlers.rs   # Axum HTTP handlers and embedded HTML editor
├── Cargo.toml
└── README.md
```

## Build Commands

```bash
# Build debug
cargo build

# Build release
cargo build --release

# Run
cargo run
cargo run -- --config /path/to/dhcpd.conf --port 9000

# Run single test
cargo test <test_name>
cargo test --test <integration_test_name>
```

## Lint and Format

```bash
# Format code (uses rustfmt defaults)
cargo fmt

# Run clippy lints
cargo clippy

# Run clippy with all warnings as errors
cargo clippy -- -D warnings

# Check formatting and linting
cargo fmt -- --check && cargo clippy
```

## Testing

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run doc tests
cargo test --doc
```

## Code Style Guidelines

### General

- **Edition**: Rust 2024 (set in Cargo.toml)
- **Formatting**: Follow rustfmt default conventions
- **No 2024-style semicolons**: Use `fn main() -> Result<(), Box<dyn Error>>` (not `-> Result<(), Box<dyn Error>>!`)

### Imports

```rust
// Order: standard library, external crates, local modules
use std::net::SocketAddr;
use axum::{extract::State, Router};
use serde::{Deserialize, Serialize};

use crate::models::DhcpConfig;
use crate::parser::{parse_from_file, serialize_to_string};
```

### Types and Structs

- Use `pub struct` for public API types
- Derive `Debug, Clone, Serialize, Deserialize` for all model types
- Use `Option<T>` for optional fields (not `Option<T>` with default `None`)
- Use `Vec<T>` for collections (not `HashMap` unless needed)

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Host {
    pub name: String,
    pub hardware_address: Option<String>,  // NOT Option<String> with init
    pub fixed_address: Option<String>,
    pub options: Vec<DhcpOption>,
}
```

### Naming Conventions

- **Structs**: PascalCase (`DhcpConfig`, `SharedNetwork`)
- **Fields**: snake_case (`hardware_address`, `fixed_address`)
- **Enums**: PascalCase
- **Functions**: snake_case (`parse_dhcpd_conf`, `serialize_to_string`)
- **Constants**: SCREAMING_SNAKE_CASE (if any)

### Error Handling

- Use `Result<T, E>` for fallible operations
- For main/async functions: `-> Result<(), Box<dyn std::error::Error>>`
- Convert errors with `.map_err(|e| e.to_string())` or `.map_err(|e| format!("...: {}", e))`
- In handlers: Return `(StatusCode, String)` tuples for API errors

```rust
// Parser errors return String
pub fn parse_from_file<P: AsRef<Path>>(path: P) -> Result<DhcpConfig, String> {
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    Ok(parse_dhcpd_conf(&content))
}

// Handler errors return tuple
async fn get_config(...) -> Result<Json<DhcpConfig>, (StatusCode, String)> {
    parse_from_file(&state.config_path)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))
}
```

### Async/Await

- Use `#[tokio::main]` for the main async entry point
- Prefer `async fn` for handler functions
- Use `?` operator for error propagation

### Serde

- Use `#[derive(Serialize, Deserialize)]` on all model types
- Use field attributes sparingly; prefer matching dhcpd.conf naming

### CLI Arguments

- Use `clap` with `#[derive(Parser)]` and `#[command]` attributes
- Keep defaults sensible (port 8080, config `/etc/dhcp/dhcpd.conf`)

```rust
#[derive(Parser, Debug)]
#[command(name = "dhcp-editor")]
#[command(about = "Web interface for editing dhcpd.conf files")]
struct Args {
    #[arg(short, long, default_value = "/etc/dhcp/dhcpd.conf")]
    config: PathBuf,

    #[arg(short, long, default_value = "8080")]
    port: u16,
}
```

### Axum Patterns

- Use `State<Arc<AppState>>` for shared state extraction
- Return `Result<Json<T>, (StatusCode, String)>` for API endpoints
- Use `Html(String)` for HTML responses
- Chain `.layer(TraceLayer::new_for_http())` for request tracing

```rust
pub fn create_routes(state: AppState) -> Router {
    Router::new()
        .route("/api/config", get(get_config))
        .route("/api/config", post(save_config))
        .with_state(Arc::new(state))
}
```

### Logging

- Use `tracing` crate with `tracing::info!` for startup/runtime info
- Initialize with `tracing_subscriber::fmt::init()` in main
