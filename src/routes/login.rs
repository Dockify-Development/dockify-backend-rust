use axum::{http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use chrono::{Duration, Utc};
use rusqlite::Error;

use crate::utils::{
    db,
    res::{jwt_resp, m_resp},
    validation,
};

fn default_str() -> String {
    "".to_string()
}

#[derive(serde::Deserialize, Debug)]
pub struct LoginParams {
    #[serde(default = "default_str")]
    username: String,
    #[serde(default = "default_str")]
    email: String,
    #[serde(default = "default_str")]
    password: String,
}

pub async fn handler(Json(payload): Json<LoginParams>) -> impl IntoResponse {
    let id: &String = if payload.email.is_empty() {
        &payload.username.to_lowercase()
    } else {
        &payload.email.to_lowercase()
    };
    let password: &String = &payload.password;

    if id.is_empty() || password.is_empty() {
        return m_resp(
            StatusCode::UNAUTHORIZED,
            "Invalid username/email or password",
        );
    }
    let (hash, username, _) = match db::get_user_info(&id) {
        Ok((hash, verified, username, email)) => {
            if verified == 0 {
                return m_resp(
                    StatusCode::UNAUTHORIZED,
                    "Invalid username/email or password",
                );
            } else {
                (hash, username, email)
            }
        }
        Err(err) => match err {
            Error::QueryReturnedNoRows => {
                return m_resp(
                    StatusCode::UNAUTHORIZED,
                    "Invalid username/email or password",
                )
            }
            _ => {
                eprintln!(
                    "An error occurred while getting user info in login: {}",
                    err
                );
                return m_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Please contact support for help.",
                );
            }
        },
    };
    match validation::verify_password(password, &hash) {
        Ok(_) => (),
        Err(_) => {
            return m_resp(
                StatusCode::UNAUTHORIZED,
                "Invalid username/email or password",
            )
        }
    }
    let jwt: String = match validation::generate_jwt(username, Utc::now() + Duration::hours(24)) {
        Ok(jwt) => jwt,
        Err(err) => {
            eprintln!(
                "An error occurred while generating a jwt for login: {}",
                err
            );
            return m_resp(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Please contact support for help.",
            );
        }
    };
    return jwt_resp(StatusCode::OK, jwt);
}

pub fn get_routes() -> Router {
    Router::new().route("/api/login", post(handler))
}
