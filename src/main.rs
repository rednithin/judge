use hyper::{Body, Request, Response, Server, StatusCode};
// Import the routerify prelude traits.
use routerify::prelude::*;
use routerify::{Middleware, RequestInfo, Router, RouterService};
use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::{convert::Infallible, net::SocketAddr};
use tokio::net::UdpSocket;
use tracing::{info, Span};
use tracing::{instrument, Instrument};
use tracing_bunyan_formatter::{BunyanFormattingLayer, JsonStorageLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::Registry;
use uuid::Uuid;

mod traits;
mod util;

// Define an app state to share it across the route handlers and middlewares.
struct State(u64);

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

// A handler for "/" page.
#[tracing::instrument(skip(req))]
async fn home_handler(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Access the app state.
    let state = req.data::<State>().unwrap();
    let span = req.extensions().get::<Span>().cloned();

    println!("State value: {}", state.0);
    info!("Orphan event without a parent span");
    let mut response = Response::new(Body::from("Home page"));
    response.extensions_mut().insert(span);
    Ok(response)
}

// A handler for "/users/:userId" page.
async fn user_handler(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let user_id = req.param("userId").unwrap();
    Ok(Response::new(Body::from(format!("Hello {}", user_id))))
}

// A middleware which logs an http request.
async fn logger(mut req: Request<Body>) -> anyhow::Result<Request<Body>> {
    let request_id = Uuid::new_v4();
    let span =
        tracing::info_span!("Http Request", http.method = %req.method(), request_id = %request_id);
    req.extensions_mut().insert(span.clone());
    println!(
        "{} {} {}",
        req.remote_addr(),
        req.method(),
        req.uri().path()
    );
    Ok(req)
}

async fn logger2(res: Response<Body>, req: RequestInfo) -> anyhow::Result<Response<Body>> {
    Ok(res)
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
        .middleware(Middleware::pre(logger))
        .middleware(Middleware::post_with_info(logger2))
        .get("/users/:userId", user_handler)
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
