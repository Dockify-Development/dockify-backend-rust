use crate::utils::{db, validation};
use axum::http::{header, HeaderMap};
use axum::{extract::Query, http::StatusCode, response::IntoResponse, routing::get, Router};
use base64::engine::general_purpose;
use base64::Engine;

fn default_str() -> String {
    "".to_string()
}

#[derive(serde::Deserialize, Debug)]
pub struct VerifyParams {
    #[serde(default = "default_str")]
    code: String,
}
pub async fn handler(Query(params): Query<VerifyParams>) -> impl IntoResponse {
    let encoded_code = params.code;
    let mut headers = HeaderMap::new();

    if encoded_code.is_empty() {
        return (StatusCode::BAD_REQUEST, headers);
    }
    let decoded_code = match general_purpose::STANDARD.decode(encoded_code) {
        Ok(code) => match String::from_utf8(code) {
            Ok(decoded_string) => decoded_string,
            Err(_) => {
                return (StatusCode::BAD_REQUEST, headers);
            }
        },
        Err(_) => {
            return (StatusCode::BAD_REQUEST, headers);
        }
    };

    match db::check_exists(&decoded_code, "verification_code", "verification_codes") {
        Ok(exists) if !exists => return (StatusCode::BAD_REQUEST, headers),
        Err(e) => {
            eprintln!(
                "An error occurred while checking if verification code exists in db: {}",
                e
            );
            return (StatusCode::INTERNAL_SERVER_ERROR, headers);
        }
        _ => {}
    }

    match validation::verify_jwt(&decoded_code).await {
        Ok(claims) => {
            let username = claims.sub.clone();
            match db::verify_user(&username) {
                Ok(_) => (),
                Err(err) => {
                    eprintln!("An error occurred while verifying a user: {}", err);
                    return (StatusCode::INTERNAL_SERVER_ERROR, headers);
                }
            }
        }
        Err(_) => {
            return (StatusCode::BAD_REQUEST, headers);
        }
    }
    match db::remove_code(&decoded_code) {
        Ok(_) => {
            headers.insert(
                header::LOCATION,
                "https://dockify.xyz/login".parse().unwrap(),
            );
            return (StatusCode::TEMPORARY_REDIRECT, headers);
        }
        Err(err) => {
            eprintln!("An error occurred while removing vcode from db: {}", err);
            return (StatusCode::INTERNAL_SERVER_ERROR, headers);
        }
    }
}
pub fn get_routes() -> Router {
    Router::new().route("/api/verify", get(handler))
}
