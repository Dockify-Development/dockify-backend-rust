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
    routing::post,
    Router,
};
use bollard::Docker;
use serde_json::from_slice;

use crate::utils::{
    container::{self, ContainerName},
    db::get_user_containers,
    res::m_resp,
    validation,
};

pub async fn handler(req: Request<Body>) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let (validated, username) = validation::validate_request(&parts.headers).await;
    if !validated {
        return m_resp(StatusCode::UNAUTHORIZED, "Invalid token");
    }
    let delete_params: ContainerName =
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
    let docker = match Docker::connect_with_local_defaults() {
        Ok(docker) => docker,
        Err(err) => {
            eprintln!("Error connecting to Docker: {}", err);
            return m_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Please contact support for help.",
            );
        }
    };
    let containers = match get_user_containers(&username) {
        Ok(v) => v,
        Err(e) => match e {
            rusqlite::Error::QueryReturnedNoRows => {
                return m_resp(StatusCode::NOT_FOUND, "No container found with this name.")
            }
            _ => {
                eprintln!(
                    "An error occurred while getting user containers (delete:56): {}",
                    e
                );
                return m_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Please contact support for help.",
                );
            }
        },
    };
    if !container::container_exists(&containers, &delete_params.name) || containers.is_empty() {
        return m_resp(StatusCode::NOT_FOUND, "No container found with this name.");
    }
    match container::delete_container_by_name(&docker, &delete_params.name).await {
        Ok(_) => (),
        Err(e) => {
            eprintln!("An error occurred while deleting a container: {}", e);
            return m_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Please contact support for help.",
            );
        }
    }
    return m_resp(StatusCode::OK, "");
}

pub fn get_routes() -> Router {
    Router::new().route("/api/delete_container", post(handler))
}
