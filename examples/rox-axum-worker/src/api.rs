use axum::{
    extract::{Multipart, State},
    Json,
};

use crate::{
    calculator,
    error::ApiError,
    models::{AppState, HealthResponse, RatingResponse},
    request,
};

pub(crate) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub(crate) async fn rate_chart(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Json<RatingResponse>, ApiError> {
    let request = request::parse(multipart, &state).await?;
    let response = tokio::task::spawn_blocking(move || calculator::rate(request))
        .await
        .map_err(|error| ApiError::internal(format!("calculation worker failed: {error}")))??;

    Ok(Json(response))
}
