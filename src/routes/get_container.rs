#![warn(unused_variables)]
use axum::{
    body::Body, extract::Request, http::StatusCode, response::IntoResponse, routing::post, Router,
};

use crate::utils::{
    db,
    res::{m_resp, Respond},
    validation,
};

pub async fn handler(req: Request<Body>) -> impl IntoResponse {
    let (validated, username) = validation::validate_request(&req.headers()).await;
    if !validated {
        return m_resp(StatusCode::UNAUTHORIZED, "Invalid token");
    }
    Respond::Containers(
        StatusCode::OK,
        match db::get_user_containers(&username) {
            Ok(c) => c,
            Err(e) => match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    return Respond::Containers(StatusCode::OK, Vec::new())
                }
                _ => {
                    eprintln!("An error occurred while getting user containers (get_containers.rs:17): {}", e);
                    return m_resp(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "Please contact support for help.",
                    );
                }
            },
        },
    )
}

pub fn get_routes() -> Router {
    Router::new().route("/api/get_containers", post(handler))
}
