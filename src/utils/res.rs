use crate::utils::db::Container;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

#[derive(Serialize)]
pub struct MReturn {
    pub message: String,
}

#[derive(Serialize)]
pub struct CReturn {
    pub id: String,
    pub port: u16,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum GenericResponse {
    Token { token: String },
    Pre { name: String },
    Credits { credits: i64 },
}

pub enum Respond {
    Container(StatusCode, CReturn),
    Message(StatusCode, String),
    Containers(StatusCode, Vec<Container>),
    Generic(StatusCode, GenericResponse),
}

fn json_resp<T: Serialize>(status_code: StatusCode, body: T) -> Response {
    (status_code, Json(body)).into_response()
}

impl IntoResponse for Respond {
    fn into_response(self) -> Response {
        match self {
            Respond::Container(status_code, c_return) => json_resp(status_code, c_return),
            Respond::Message(status_code, message) => json_resp(status_code, MReturn { message }),
            Respond::Containers(status_code, containers) => json_resp(status_code, containers),
            Respond::Generic(status_code, response) => json_resp(status_code, response),
        }
    }
}

pub fn m_resp(status_code: StatusCode, message: impl Into<String>) -> Respond {
    Respond::Message(status_code, message.into())
}

pub fn jwt_resp(status_code: StatusCode, token: String) -> Respond {
    Respond::Generic(status_code, GenericResponse::Token { token })
}

pub fn pre_resp(name: String) -> Respond {
    Respond::Generic(StatusCode::OK, GenericResponse::Pre { name })
}

pub fn credits_resp(credits: i64) -> Respond {
    Respond::Generic(StatusCode::OK, GenericResponse::Credits { credits })
}
