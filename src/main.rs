use axum::{http::HeaderMap, routing::get, Router};
use axum_extra::extract::CookieJar;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
async fn hello(header: HeaderMap, jar: CookieJar) -> String {
    let headers = header
        .into_iter()
        .map(|item| format!("{:?} = {:?}", item.0, item.1))
        .collect::<Vec<String>>()
        .join("\n");
    let cookies = jar
        .iter()
        .map(|item| {
            format!(
                "{:?} = {:?}",
                item.name_value_trimmed(),
                item.value_trimmed()
            )
        })
        .collect::<Vec<String>>()
        .join("\n");
    format!("Cookies\n=======\n{cookies}\nHeaders\n=======\n{headers}")
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
