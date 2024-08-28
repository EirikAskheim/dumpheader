use axum::{http::HeaderMap, routing::{get, post, post_service}, Router};
use axum_extra::extract::CookieJar;
use std::env;
use tokio::signal;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
async fn hello(header: HeaderMap, jar: CookieJar) -> String {
    let max_header_width = header.iter().map(|x| x.0.as_str().len()).max().unwrap();
    let mut headers = header
        .into_iter()
        .map(|item| {
            let header_name = match item.0 {
                Some(value) => value.as_str().to_owned(),
                None => "no-header-name".to_owned(),
            };
            let header_value = item.1.to_str().unwrap().to_owned();
            format!("{header_name:<max_header_width$} = {header_value:}")
        })
        .collect::<Vec<String>>();
    headers.sort();
    let headers = headers.join("\n");
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
    let pod_name = match env::var("POD_NAME") {
        Ok(name) => name,
        Err(_) => "unknown".to_string(),
    };

    format!("Pod name: {pod_name}\nCookies\n=======\n{cookies}\nHeaders\n=======\n{headers}\n")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .with_max_level(tracing::Level::DEBUG)
        .init();
    // build our application with a single route
    let app = Router::new().fallback_service(get(hello)).fallback_service(post(hello)).layer(
        TraceLayer::new_for_http()
            .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
            .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
    );

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
