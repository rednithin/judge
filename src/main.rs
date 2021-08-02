use axum::{extract::Json, prelude::*};

use std::net::SocketAddr;

use uuid::Uuid;

use crate::traits::{LanguageExecutor, Python};

mod traits;
mod util;

use util::CustomError;

#[derive(Debug, serde::Deserialize)]
struct CodeExecutionRequest {
    code: String,
    language: String,
}

#[derive(Debug, serde::Serialize)]
struct CodeExecutionResponse {
    stdout: Option<String>,
    stderr: Option<String>,
}

#[tracing::instrument(skip(payload))]
async fn exec_handler(
    Json(payload): Json<CodeExecutionRequest>,
) -> Result<response::Json<CodeExecutionResponse>, CustomError> {
    // Access the app state.

    let uuid = Uuid::new_v4();
    let response = match payload.language.as_str() {
        "PYTHON" => {
            let python = Python::new(uuid, payload.code.clone());
            python.prepare().await?;
            let output = python.execute().await?;
            python.teardown().await?;
            CodeExecutionResponse {
                stdout: Some(String::from_utf8(output.stdout).unwrap()),
                stderr: Some(String::from_utf8(output.stderr).unwrap()),
            }
        }
        _ => CodeExecutionResponse {
            stderr: None,
            stdout: None,
        },
    };

    Ok(response.into())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::initialize_tracing();

    let app = route("/api/exec-code", get(exec_handler));
    // Create a Service from the router above to handle incoming requests.

    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));

    let server = hyper::Server::bind(&addr).serve(app.into_make_service());

    let _ = tokio::join!(server);
    Ok(())
}
