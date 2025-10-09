use axum::{
    routing::get,
    Router,
};
use std::net::SocketAddr;

pub async fn run_server() {
    // Build our application with routes
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler));

    // Get port from environment or use default
    let port: u16 = std::env::var("APP_PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse()
        .unwrap_or(3000);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("🚀 Server listening on http://{}", addr);

    // Run the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn root_handler() -> &'static str {
    "LH Traffic - Anti-Phishing Microservice"
}

async fn health_handler() -> &'static str {
    "OK"
}
