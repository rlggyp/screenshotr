mod auth;
mod app_state;
mod config;
mod screenshotr;

type Error = Box<dyn std::error::Error + Send + Sync>;

use crate::{app_state::AppState, screenshotr::handler::screenshot};

use std::sync::Arc;

use axum::{middleware, routing};
use tokio::signal::unix::{signal, SignalKind};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let log_config_file = std::env::var("LOG_CONFIG_FILE")
        .expect("Environment variable 'LOG_CONFIG_FILE' not found");

    log4rs::init_file(&log_config_file, Default::default())
        .expect("Failed to init log4rs");

    let config = config::Config::get_config()?;
    let app_state = Arc::new(AppState::new(config)?);

    let app = axum::Router::new()
        .route("/api/screenshotr",
            routing::post(screenshot)
                .layer(middleware::from_fn_with_state(app_state.clone(), auth::middleware::hmac_middleware))
                .layer(middleware::from_fn_with_state(app_state.clone(), auth::middleware::basic_auth_middleware))
        )
        .nest_service("/screenshotr/images", tower_http::services::ServeDir::new("assets/screenshots"))
        .with_state(app_state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:12009").await?;
    log::info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    let mut sigint = signal(SignalKind::interrupt()).expect("failed to bind SIGINT handler");
    let mut sigterm = signal(SignalKind::terminate()).expect("failed to bind SIGTERM handler");

    tokio::select! {
        _ = sigint.recv() => {
            log::info!("SIGINT received, Gracefully shutting down.");
        }
        _ = sigterm.recv() => {
            log::info!("SIGTERM received, Gracefully shutting down.");
        }
    }
}