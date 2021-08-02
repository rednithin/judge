use axum::response::{self, IntoResponse};
use http::StatusCode;
use hyper::Body;
use serde_json::json;
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

pub fn initialize_tracing() {
    let formatting_layer = BunyanFormattingLayer::new("judge".into(), std::io::stdout);
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let subscriber = Registry::default()
        .with(JsonStorageLayer)
        .with(env_filter)
        .with(formatting_layer);
    tracing::subscriber::set_global_default(subscriber).unwrap();
}

pub struct CustomError(anyhow::Error);

impl From<anyhow::Error> for CustomError {
    fn from(anyhow_err: anyhow::Error) -> Self {
        Self(anyhow_err)
    }
}

impl IntoResponse for CustomError {
    fn into_response(self) -> http::Response<Body> {
        let (status, error_json) = match self.0 {
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                json!("Something went wrong"),
            ),
        };

        let mut response = response::Json(json!({
            "error": error_json,
        }))
        .into_response();

        *response.status_mut() = status;

        response
    }
}
