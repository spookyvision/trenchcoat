use std::{
    collections::HashSet,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{
    body::Bytes,
    http::{HeaderMap, Request},
    response::{Html, Response},
    routing::{get, post},
    Router,
};
use pixelblaze_rs::forth;
use tokio::sync::broadcast;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "example_chat=trace".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/blazeit", post(blazeit))
        // `TraceLayer` is provided by tower-http so you have to add that as a dependency.
        // It provides good defaults but is also very customizable.
        //
        // See https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html for more details.
        .layer(TraceLayer::new_for_http())
        // If you want to customize the behavior using closures here is how
        //
        // This is just for demonstration, you don't need to add this middleware twice
        .layer(
            TraceLayer::new_for_http()
                .on_request(|_request: &Request<_>, _span: &Span| {
                    // ...
                })
                .on_response(|_response: &Response, _latency: Duration, _span: &Span| {
                    // ...
                })
                .on_body_chunk(|_chunk: &Bytes, _latency: Duration, _span: &Span| {
                    // ..
                })
                .on_eos(
                    |_trailers: Option<&HeaderMap>, _stream_duration: Duration, _span: &Span| {
                        // ...
                    },
                )
                .on_failure(
                    |_error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                        // ...
                    },
                ),
        );

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn blazeit() {}

use std::io::{self, BufRead};

fn to_quit(cmd: &str) -> bool {
    match cmd {
        "quit" | "q" | "exit" => true,
        _ => false,
    }
}

fn run_forth() {
    todo!();
    // let mut env = forth::env::ForthEnv::empty();
    // let intr = forth::inter::Interpreter::new();

    // let stdin = io::stdin();
    // for line in stdin.lock().lines() {
    //     let mut input = line.unwrap();
    //     input = input.trim().to_string();

    //     if to_quit(&input) {
    //         println!("Bye!");
    //         return;
    //     } else {
    //         intr.eval(&mut env, &input);
    //     }
    // }
}
