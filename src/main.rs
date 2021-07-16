use hyper::{Body, Request, Response, Server, StatusCode};
// use routerify::prelude::*;
use routerify::{Middleware, RequestInfo, Router, RouterService};
use tracing::Instrument;

use std::net::SocketAddr;

use tracing::Span;

use uuid::Uuid;

use crate::traits::{LanguageExecutor, Python};
use crate::util::get_request_body;

mod traits;
mod util;

// Define an app state to share it across the route handlers and middlewares.
struct State(u64);

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

// A handler for "/" page.
#[tracing::instrument(skip(req))]
async fn exec_handler(mut req: Request<Body>) -> anyhow::Result<Response<Body>> {
    // Access the app state.
    let processed_request = get_request_body::<CodeExecutionRequest>(&mut req).await?;
    let span = req.extensions().get::<Span>().cloned();

    let uuid = Uuid::new_v4();
    let response = match processed_request.language.as_str() {
        "PYTHON" => {
            let python = Python::new(uuid, processed_request.code);
            python.prepare().await?;
            let output = python.execute().await?;
            python.teardown().await?;
            CodeExecutionResponse {
                stdout: Some(String::from_utf8(output.stdout)?),
                stderr: Some(String::from_utf8(output.stderr)?),
            }
        }
        _ => CodeExecutionResponse {
            stderr: None,
            stdout: None,
        },
    };

    let mut response = Response::new(Body::from(serde_json::to_string(&response)?));
    response
        .headers_mut()
        .append("Content-Type", "application/json".parse()?);
    response.extensions_mut().insert(span);
    Ok(response)
}

// A middleware which logs an http request.
async fn create_tracing_span(mut req: Request<Body>) -> anyhow::Result<Request<Body>> {
    let request_id = Uuid::new_v4();
    let span =
        tracing::info_span!("Http Request", http.method = %req.method(), request_id = %request_id);
    req.extensions_mut().insert(span.clone());
    Ok(req)
}

// Define an error handler function which will accept the `routerify::Error`
// and the request information and generates an appropriate response.
async fn error_handler(err: routerify::RouteError, _: RequestInfo) -> Response<Body> {
    eprintln!("{}", err);
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from(format!("Something went wrong: {}", err)))
        .unwrap()
}

// Create a `Router<Body, Infallible>` for response body type `hyper::Body`
// and for handler error type `Infallible`.
fn router() -> Router<Body, anyhow::Error> {
    Router::builder()
        .data(State(100))
        .middleware(Middleware::pre(create_tracing_span))
        .get("/api/exec-code", traceroute!(exec_handler))
        .err_handler_with_info(error_handler)
        .build()
        .unwrap()
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    util::initialize_tracing();

    let router = router();

    // Create a Service from the router above to handle incoming requests.
    let service = RouterService::new(router).unwrap();

    // The address on which the server will be listening.
    let addr = SocketAddr::from(([0, 0, 0, 0], 3030));

    // Create a server by passing the created service to `.serve` method.
    let server = Server::bind(&addr).serve(service);

    let _ = tokio::join!(server);
    Ok(())
}
