use axum::{
    extract::DefaultBodyLimit,
    http::header,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};

use crate::{
    api,
    models::AppState,
    pool::{CalculatorPool, CalculatorPoolInitError},
};

const MAX_UPLOAD_BYTES: usize = 16 * 1024 * 1024;
const INDEX_HTML: &str = include_str!("../static/index.html");
const APP_JS: &str = include_str!("../static/app.js");
const CHARTS_JS: &str = include_str!("../static/charts.js");
const STYLES_CSS: &str = include_str!("../static/styles.css");

/// Builds the Axum application used by the example binary.
pub fn app() -> Result<Router, CalculatorPoolInitError> {
    let http = reqwest::Client::builder()
        .user_agent("rox-minacalc-worker/0.1")
        .build()
        .expect("the fixed HTTP client configuration must be valid");
    let calculators = CalculatorPool::for_available_parallelism()?;

    Ok(Router::new()
        .route("/", get(index))
        .route("/app.js", get(javascript))
        .route("/charts.js", get(charts_javascript))
        .route("/styles.css", get(stylesheet))
        .route("/api/health", get(api::health))
        .route("/api/rate", post(api::rate_chart))
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .with_state(AppState { http, calculators }))
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn javascript() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        APP_JS,
    )
}

async fn charts_javascript() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        CHARTS_JS,
    )
}

async fn stylesheet() -> impl IntoResponse {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        STYLES_CSS,
    )
}
