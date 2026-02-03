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
            log::error!("Invalid request body: {e:#?}");
            return (StatusCode::BAD_REQUEST, "Invalid request body".to_string()).into_response();
        },
        Ok(payload) => payload,
    };

    match app_state.screenshot.take_screenshot(&payload.url).await {
        Ok(image_url) => {
            let response_json = serde_json::json!({
                "image_url": image_url,
            });

            let response = serde_json::to_string(&response_json).unwrap_or_default();

           (StatusCode::OK, response).into_response()
        },
        Err(e) => {
            let response_json = serde_json::json!({
                "message": e.to_string(),
            });

            let response = serde_json::to_string(&response_json).unwrap_or_default();

           (StatusCode::INTERNAL_SERVER_ERROR, response).into_response()
        }
    }
}