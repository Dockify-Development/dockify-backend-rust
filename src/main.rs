#![allow(dead_code)]

use std::time::Duration;

use axum::{error_handling::HandleErrorLayer, http::StatusCode, BoxError, Router};
use dockify_backend::{routes, utils::db::create_db};
use dotenvy::dotenv;
use tower::{buffer::BufferLayer, limit::RateLimitLayer, ServiceBuilder};

#[tokio::main]
async fn main() {
    dotenv().ok();

    let routes: Vec<Router> = routes::get_routes();

    let app: Router = routes
        .into_iter()
        .fold(Router::new(), |router: Router, route: Router| {
            router.merge(route)
        })
        .layer(
            ServiceBuilder::new()
                .layer(HandleErrorLayer::new(|err: BoxError| async move {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled error: {}", err),
                    )
                }))
                .layer(BufferLayer::new(1024))
                .layer(RateLimitLayer::new(5, Duration::from_secs(5))),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    create_db().await;
    println!("Dockify backend is running...");
    axum::serve(listener, app).await.unwrap();
}
