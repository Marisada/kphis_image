mod handlers;
mod route;

use axum::{handler::HandlerWithoutStateExt, http::StatusCode, Router};
use std::{
    net::SocketAddr, 
    sync::{atomic::{AtomicU32, Ordering}, Arc, Mutex},
};
use tower_http::services::ServeDir;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use model::ImageData;

static GLOBAL_COUNT: AtomicU32 = AtomicU32::new(1);

#[derive(Clone)]
pub struct AppState {
    pub images: Arc<Mutex<Vec<ImageData>>>,
    pub first_table: Arc<Mutex<Vec<ImageData>>>,
    pub second_table: Arc<Mutex<Vec<ImageData>>>,
}

impl AppState {
    fn new() -> Self {
        Self { 
            images: Arc::new(Mutex::new(Vec::new())),
            first_table: Arc::new(Mutex::new(Vec::new())),
            second_table: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

// return old value
pub fn add_count() -> u32 {
    GLOBAL_COUNT.fetch_add(1, Ordering::SeqCst)
}

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

    let state = AppState::new();
    let app = Router::new()
        .nest("/api", route::router(state))
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