pub mod error;
pub mod game;
pub mod server;

use std::{fs, net::TcpListener, path::PathBuf, sync::Arc, time::Duration};

use axum::{
    error_handling::HandleErrorLayer,
    http::{HeaderValue, StatusCode},
    routing::{any, post},
    BoxError, Router,
};
use axum_server::tls_rustls::RustlsConfig;
use clap::Parser;
use tokio::{signal, time::interval};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

use crate::server::{create_lobby, ws::ws_handler, ServerState};

/// Command-line arguments structure using Clap
#[derive(Parser, Debug)]
#[command(name = env!("CARGO_PKG_NAME"))]
struct Args {
    /// Host in the format ip:port
    host: String,

    /// To enable cross origin reguests from the frontend
    #[arg(short, default_value = "https://api.clash.nwrenger.dev")]
    frontend_origin: String,

    /// Path to the cache folder
    #[arg(long, default_value = "cache")]
    cache: PathBuf,

    /// Path to the SSL certificate
    #[arg(
        long,
        default_value = "/etc/letsencrypt/live/api.clash.nwrenger.dev/fullchain.pem"
    )]
    cert: PathBuf,

    /// Path to the SSL private key
    #[arg(
        long,
        default_value = "/etc/letsencrypt/live/api.clash.nwrenger.dev/privkey.pem"
    )]
    key: PathBuf,
}

#[tokio::main]
async fn main() {
    logging();

    let args = Args::parse();

    if !PathBuf::from(&args.cache).exists() {
        info!(
            "Cache folder didn't exist, creating one at path {:?}!",
            args.cache
        );
        fs::create_dir_all(&args.cache).unwrap();
    }

    if !PathBuf::from(&args.cert).exists() {
        error!("The SSL certificate path {:?} does not exist!", args.cert);
        std::process::exit(1);
    }

    if !PathBuf::from(&args.key).exists() {
        error!("The SSL key path {:?} does not exist!", args.key);
        std::process::exit(1);
    }

    // init some ws sockets and lobbies state
    let state = Arc::new(ServerState::new(args.cache));

    // spawn the janitor
    {
        let janitor = state.clone();
        tokio::spawn(async move {
            // tick every 30 minutes
            let mut tick = interval(Duration::from_secs(30 * 60));
            loop {
                tick.tick().await;
                let removed = janitor.clean_unused().await;
                info!("Pruned {} stale lobbies", removed);
            }
        });
    }

    let app = Router::new()
        .route("/ws/{uuid}", any(ws_handler).with_state(state.clone()))
        .route("/lobby", post(create_lobby).with_state(state))
        .layer(
            ServiceBuilder::new()
                .layer(
                    CorsLayer::new()
                        .allow_origin(args.host.parse::<HeaderValue>().unwrap())
                        .allow_origin(args.frontend_origin.parse::<HeaderValue>().unwrap())
                        .allow_headers(Any)
                        .allow_methods(Any),
                )
                .layer(CompressionLayer::new())
                .layer(HandleErrorLayer::new(|error: BoxError| async move {
                    if error.is::<tower::timeout::error::Elapsed>() {
                        Ok(StatusCode::REQUEST_TIMEOUT)
                    } else {
                        error!("Internal server error: {error}");
                        Err(StatusCode::INTERNAL_SERVER_ERROR)
                    }
                }))
                .timeout(Duration::from_secs(10))
                .layer(TraceLayer::new_for_http())
                .into_inner(),
        );

    let handle = axum_server::Handle::new();
    let shut = shutdown_signal();

    let tcp = TcpListener::bind(&args.host).unwrap();
    let tls = RustlsConfig::from_pem_file(&args.cert, &args.key)
        .await
        .unwrap();

    info!("Server started on \"{}\"", args.host);

    let server = axum_server::from_tcp_rustls(tcp, tls)
        .handle(handle.clone())
        .serve(app.into_make_service());

    tokio::select! {
        () = shut =>
            handle.graceful_shutdown(Some(Duration::from_secs(3))),
        res = server => res.unwrap(),
    }

    // For future referenc: Here can be stuff written which should happen after shutdown
}

/// Initialize tracing
fn logging() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();
}

/// Waits for a shutdown signal (`ctrl` + `c`)
async fn shutdown_signal() {
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
}
