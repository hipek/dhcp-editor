mod handlers;
mod models;
mod parser;

use axum::serve;
use clap::Parser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber;

#[derive(Parser, Debug)]
#[command(name = "dhcp-editor")]
#[command(about = "Web interface for editing dhcpd.conf files")]
struct Args {
    #[arg(short, long, default_value = "/etc/dhcp/dhcpd.conf")]
    config: PathBuf,

    #[arg(short, long, default_value = "8080")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    if !args.config.exists() {
        info!("Config file not found at {:?}, creating empty file", args.config);
        std::fs::write(&args.config, "# DHCP Configuration File\n")?;
    }

    let state = handlers::AppState {
        config_path: args.config,
    };

    let app = handlers::create_routes(state).layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([0, 0, 0, 0], args.port));
    info!("Starting server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve(listener, app).await?;

    Ok(())
}
