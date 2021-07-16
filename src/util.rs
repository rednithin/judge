use anyhow::bail;
use hyper::{Body, Request};
use serde::de::DeserializeOwned;

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

pub async fn get_request_body<T: DeserializeOwned>(req: &mut Request<Body>) -> anyhow::Result<T> {
    match hyper::body::to_bytes(req.body_mut()).await {
        Ok(bytes) => {
            let bytes = bytes.to_vec();
            let body = match serde_json::from_slice::<T>(bytes.as_slice()) {
                Ok(body) => Ok(body),
                Err(e) => bail!("Failed to parse request body: {}", e),
            };
            body
        }
        Err(e) => bail!("Failed to convert payload to bytes: {}", e),
    }
}

#[macro_export]
macro_rules! traceroute {
    ($fn: expr) => {
        |req| async move {
            let span = req.extensions().get::<Span>().cloned();
            match span {
                Some(x) => $fn(req).instrument(x).await,
                None => $fn(req).await,
            }
        }
    };
}
