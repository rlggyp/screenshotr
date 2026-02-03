use crate::app_state::AppState;
use std::sync::Arc;

use axum::{
    extract::{Json, State}, http::StatusCode, response::IntoResponse
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Payload {
    url: String,
}

pub async fn screenshot(
    State(app_state): State<Arc<AppState>>,
    Json(payload): Json<serde_json::Value>
) -> impl IntoResponse {
    let result: Result<Payload, _> = serde_json::from_value(payload);

    let payload = match result {
        Err(e) => {
            return (StatusCode::BAD_REQUEST, format!("Error parse: {e:#?}")).into_response();
        },
        Ok(payload) => payload,
    };

    match app_state.screenshot.take_screenshot(&payload.url).await {
        Ok(filepath) => (
            StatusCode::OK,
            format!(r#"{{"status":"ok","filepath":"{}"}}"#, filepath)
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!(r#"{{"status":"error","message":"{}"}}"#, e)
        ).into_response(),
    }
}