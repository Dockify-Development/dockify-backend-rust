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
use serde_json::from_slice;

use crate::utils::{
    res::{m_resp, GenericResponse, Respond},
    resources::ContainerResources,
};

pub async fn handler(req: Request<Body>) -> impl IntoResponse {
    let container_info: ContainerResources =
        match from_slice::<ContainerResources>(&match body::to_bytes(req.into_body(), usize::MAX)
            .await
        {
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
    return Respond::Generic(
        StatusCode::OK,
        GenericResponse::Credits {
            credits: container_info.calculate_price(),
        },
    );
}

pub fn get_routes() -> Router {
    Router::new().route("/api/calculate", post(handler))
}
