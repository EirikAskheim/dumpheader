use axum::{
    body::Body,
    handler::Handler,
    http::{HeaderMap, Response},
    response::IntoResponse,
    routing::get,
    Router,
};
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
async fn hello(h: HeaderMap) -> String {
    let foo: String = h
        .into_iter()
        .map(|item| format!("{:?} = {:?}", item.0, item.1))
        .collect::<Vec<String>>()
        .join("\n");
    foo
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    // build our application with a single route
    let app = Router::new().route("/", get(hello)).layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
    );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
