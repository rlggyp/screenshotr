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
            let response_json = serde_json::json!({
                "error": "Invalid request body",
            });
            return (StatusCode::BAD_REQUEST, Json(response_json)).into_response();
        },
        Ok(payload) => payload,
    };

    if !is_allowed_domains(&payload.url, &app_state.allowed_domains) {
        let response_json = serde_json::json!({
            "error": "URL not allowed",
        });
        return (StatusCode::FORBIDDEN, Json(response_json)).into_response();
    }

    match app_state.screenshot.take_screenshot(&payload.url).await {
        Ok(image_url) => {
            let response_json = serde_json::json!({
                "image_url": image_url,
            });

            (StatusCode::OK, Json(response_json)).into_response()
        },
        Err(e) => {
            let response_json = serde_json::json!({
                "error": e.to_string(),
            });

            (StatusCode::INTERNAL_SERVER_ERROR, Json(response_json)).into_response()
        }
    }
}

pub fn is_allowed_domains(url: &str, allowed_domains: &Vec<String>) -> bool {
    let parsed = match url::Url::parse(url) {
        Ok(u) => u,
        Err(_) => return false,
    };

    match parsed.scheme() {
        "http" | "https" => {},
        _ => return false,
    }

    let host = match parsed.host_str() {
        Some(h) => h,
        None => return false,
    };

    allowed_domains.iter().any(|d| host.ends_with(d))
}