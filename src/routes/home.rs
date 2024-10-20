/*
    This source file is a part of Dockify
    Dockify is licensed under the Server Side Public License (SSPL), Version 1.
    Find the LICENSE file in the root of this repository for more details.
*/

use axum::{response::IntoResponse, routing::get, Router};

pub async fn handler() -> impl IntoResponse {
    return "Dockify is running...";
}

pub fn get_routes() -> Router {
    Router::new().route("/", get(handler))
}
