/*
    This source file is a part of Dockify
    Dockify is licensed under the Server Side Public License (SSPL), Version 1.
    Find the LICENSE file in the root of this repository for more details.
*/

use axum::{
    body,
    extract::Request,
    http::StatusCode,
    response::IntoResponse,
    routing::{post, Router},
};
use rand::distributions::{Alphanumeric, DistString};
use serde::Deserialize;
use serde_json::from_slice;

use crate::utils::{
    container::{self, user_container_count, validate_container_resources},
    db,
    res::m_resp,
    resources::ContainerResources,
    validation,
};

fn default_shares() -> i64 {
    512
}
fn default_image() -> String {
    "dorowu/ubuntu-desktop-lxde-vnc".to_string()
}
#[derive(Deserialize)]
pub struct ContainerInfo {
    #[serde(default = "default_image")]
    pub image: String,
    pub memory: i64,
    pub memory_swap: i64,
    pub cpu_cores: i64,
    #[serde(default = "default_shares")]
    pub cpu_shares: i64,
}

async fn handler(req: Request<axum::body::Body>) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let (validated, username) = validation::validate_request(&parts.headers).await;
    if !validated {
        return m_resp(StatusCode::UNAUTHORIZED, "Invalid token");
    }

    let container_count = match user_container_count(&username) {
        Ok(count) => count,
        Err(err) => {
            return err;
        }
    };
    // TODO: Create user-specific container counts
    if container_count > 1 {
        return m_resp(
            StatusCode::FORBIDDEN,
            "User's plan has reached container limit, please delete existing containers.",
        );
    }
    let credits = match db::get_user_credits(&username) {
        Ok(credits) => credits,
        Err(err) => match err {
            rusqlite::Error::QueryReturnedNoRows => 0,
            _ => {
                eprintln!("Error while getting user credits: {}", err);
                return m_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Please contact support for help.",
                );
            }
        },
    };
    let container_info: ContainerInfo =
        match from_slice::<ContainerInfo>(&match body::to_bytes(body, usize::MAX).await {
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
    let resources = ContainerResources {
        cpu_shares: 512,
        memory: container_info.memory,
        memory_swap: container_info.memory_swap,
        cpu_cores: container_info.cpu_cores,
    };
    let validate = match validate_container_resources(credits, &resources, Some(&username)) {
        Ok(b) => b,
        Err(err) => {
            eprintln!(
                "Error occurred while validating user's container resources: {}",
                err
            );
            return m_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Please contact support for help.",
            );
        }
    };
    if !validate {
        return m_resp(
            StatusCode::PAYMENT_REQUIRED,
            "Not enough credits in user's account.",
        );
    }

    let name: String = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);
    tokio::task::spawn(container::create_container(
        resources,
        container_info,
        name.clone(),
        username,
    ));
    return m_resp(StatusCode::ACCEPTED, &name);
}

pub fn get_routes() -> Router {
    Router::new().route("/api/create_new_container", post(handler))
}
