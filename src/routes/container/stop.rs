/*
    This source file is a part of Dockify
    Dockify is licensed under the Server Side Public License (SSPL), Version 1.
    Find the LICENSE file in the root of this repository for more details.
*/

use axum::{
    body::{self, Body},
    extract::Request,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Router,
};
use serde_json::from_slice;

use crate::utils::{
    container::{self, ContainerName},
    db,
    res::m_resp,
    validation,
};

pub async fn handler(req: Request<Body>) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let (validated, username) = validation::validate_request(&parts.headers).await;
    if !validated {
        return m_resp(StatusCode::UNAUTHORIZED, "Invalid token");
    }
    let containers = match db::get_user_containers(&username) {
        Ok(c) => c,
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                return m_resp(StatusCode::NOT_FOUND, "No container found with this name.")
            }
            _ => {
                eprintln!("An error occurred while getting a user's containers: {}", e);
                return m_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Please contact support for help.",
                );
            }
        },
    };
    let start_params: ContainerName =
        match from_slice::<ContainerName>(&match body::to_bytes(body, usize::MAX).await {
            Ok(bytes) => bytes,
            Err(_) => {
                return m_resp(
                    StatusCode::BAD_REQUEST,
                    "Failed to parse bytes from request body",
                )
            }
        }) {
            Ok(info) => info,
            Err(_) => {
                return m_resp(
                StatusCode::BAD_REQUEST,
                "Failed to parse JSON from request body. Ensure the correct parameters are given.",
            );
            }
        };
    if !container::container_exists(&containers, &start_params.name) || containers.is_empty() {
        return m_resp(StatusCode::NOT_FOUND, "No container found with this name.");
    }
    match container::stop_container(&start_params.name).await {
        Ok(_) => m_resp(StatusCode::OK, &start_params.name),
        Err(e) => {
            eprintln!("An error occurred while starting user container: {}", e);
            m_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Please contact support for help.",
            )
        }
    }
}

pub fn get_routes() -> Router {
    Router::new().route("/api/stop_container", get(handler))
}
