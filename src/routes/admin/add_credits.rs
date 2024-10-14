use axum::{
    body::{self, Body},
    extract::Request,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use serde::Deserialize;
use serde_json::from_slice;

use crate::utils::{
    db::{self, is_admin},
    res::m_resp,
    validation,
};
#[derive(Deserialize)]
struct CreditsBody {
    username: String,
    credits: i64,
}
pub async fn handler(req: Request<Body>) -> impl IntoResponse {
    let (parts, body) = req.into_parts();
    let (validated, username) = validation::validate_request(&parts.headers).await;
    if !validated {
        return m_resp(StatusCode::UNAUTHORIZED, "Invalid token");
    }
    if !match is_admin(username) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error occurred while checking for admin role: {}", e);
            return m_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Please contact support for help.",
            );
        }
    } {
        return m_resp(
            StatusCode::FORBIDDEN,
            "User doesn't have admin permissions.",
        );
    }
    let body: CreditsBody = match from_slice::<CreditsBody>(&match body::to_bytes(body, usize::MAX)
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
    if let Err(rusqlite::Error::QueryReturnedNoRows) = db::get_user_info(&body.username) {
        return m_resp(StatusCode::BAD_REQUEST, "User not found.");
    }
    match db::set_user_credits(&body.username, body.credits) {
        Ok(_) => {
            return m_resp(
                StatusCode::OK,
                format!(
                    "Successfully set {}'s credits to {}",
                    body.username, body.credits
                ),
            )
        }
        Err(e) => {
            eprintln!("Error while setting user credits: {}", e);
            return m_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Please contact support for help.",
            );
        }
    }
}

pub fn get_routes() -> Router {
    Router::new().route("/api/admin/add_credits", post(handler))
}
