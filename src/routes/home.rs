use axum::{response::IntoResponse, routing::get, Router};

pub async fn handler() -> impl IntoResponse {
    return "Dockify is running...";
}

pub fn get_routes() -> Router {
    Router::new().route("/", get(handler))
}
