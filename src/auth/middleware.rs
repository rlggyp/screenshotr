use crate::auth::basic_auth::BasicAuth;
use crate::app_state::AppState;

use std::sync::Arc;
use axum::{
  extract::{Request, State},
  http::StatusCode,
  middleware::Next,
  response::{IntoResponse, Response},
  body::{Body, to_bytes}
};

const MAX_PAYLOAD_BODY_SIZE: usize = 256 * 1024;

pub async fn basic_auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next
) -> Result<Response, impl IntoResponse> {
    let is_valid_userpass;

    {
        if state.basic_auth.credentials.is_empty() {
            log::debug!("[middleware][basic_auth] No credentials configured, skipping auth");
            return Ok(next.run(request).await);
        }

        let auth_header = request.headers()
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());
    
        let Some(auth_header) = auth_header else {
            log::debug!("[middleware][basic_auth] No Authorization header found");
            return Err(StatusCode::UNAUTHORIZED)
        };

        let Some(credential) = BasicAuth::is_valid_basic_auth_header(auth_header) else {
            return Err(StatusCode::UNAUTHORIZED)
        };

        {
            let cache = state.basic_auth.auth_header_cache.read().await;
            if cache.contains(&credential.encoded) {
                log::debug!("[middleware][basic_auth] Authorization header found in cache, skipping further verification");
                return Ok(next.run(request).await);
            }
        }

        is_valid_userpass = state.basic_auth.verify(credential).await;
    }

    if is_valid_userpass {
        log::debug!("[middleware][basic_auth] Authorization successful");
        Ok(next.run(request).await)
    } else {
        log::debug!("[middleware][basic_auth] Authorization failed");
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn hmac_middleware(
    State(state): State<Arc<AppState>>,
    request: Request<axum::body::Body>,
    next: Next
) -> Result<Response, impl IntoResponse> {
    let signature = request.headers()
        .get("Signature-256")
        .and_then(|h|
            h.to_str().ok().and_then(|x| x.strip_prefix("sha256="))
        )
        .map(|x| x.trim().to_string())
        .unwrap_or_default();

    let (parts, body) = request.into_parts();

    let body = match to_bytes(body, MAX_PAYLOAD_BODY_SIZE).await {
        Ok(b) => b,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    if state.hmac.verify_payload(&body, &signature) {
        log::error!("[middleware][hmac] Signature valid");

        let request = Request::from_parts(parts, Body::from(body));
        Ok(next.run(request).await)
    } else {
        log::error!("[middleware][hmac] Unauthorized, signature isn't valid");
        Err(StatusCode::UNAUTHORIZED)
    }
}
