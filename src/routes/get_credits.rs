use axum::{
    body::Body, extract::Request, http::StatusCode, response::IntoResponse, routing::get, Router,
};

use crate::utils::{
    db,
    res::{credits_resp, m_resp},
    validation,
};

pub async fn handler(req: Request<Body>) -> impl IntoResponse {
    let (validated, username) = validation::validate_request(&req.headers()).await;

    if !validated {
        return m_resp(StatusCode::UNAUTHORIZED, "Invalid token");
    }

    let credits = match db::get_user_credits(&username) {
        Ok(c) => c,
        Err(err) => match err {
            rusqlite::Error::QueryReturnedNoRows => 0,
            _ => {
                eprintln!("An error occurred while getting user's credits: {}", err);
                return m_resp(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Please contact support for help.",
                );
            }
        },
    };
    println!("Validated");

    return credits_resp(credits);
}

pub fn get_routes() -> Router {
    Router::new().route("/api/get_credits", get(handler))
}
