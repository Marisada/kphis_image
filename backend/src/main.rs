mod handlers;
mod route;

use axum::{handler::HandlerWithoutStateExt, http::StatusCode, Router};
use std::net::SocketAddr;
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "hyper=warn,tower_http=debug,axum=trace,backend=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let handle_404 = handle_404.into_service();
    let root_dir = ServeDir::new("volume/pwa")
        .precompressed_br()
        .precompressed_gzip()
        // .precompressed_deflate()
        // .precompressed_zstd()
        .not_found_service(handle_404);
    let images_dir = ServeDir::new("volume/images");
    let thumbs_dir = ServeDir::new("volume/thumbs");

    let app = Router::new()
        .nest("/api", route::router())
        .nest_service("/images", images_dir)
        .nest_service("/thumbs", thumbs_dir)
        .fallback_service(root_dir);
    serve_http(8088, app).await;
}

async fn serve_http(port: u16, app: Router) {
    let http_addr = SocketAddr::from(([0, 0, 0, 0], port));
    let listener = tokio::net::TcpListener::bind(http_addr).await.unwrap();
    info!(
        "HTTP server started listening on {}, please Ctrl-c to terminate server.",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Not found")
}