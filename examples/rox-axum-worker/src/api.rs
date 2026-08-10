use axum::{
    extract::{Multipart, State},
    Json,
};

use crate::{
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
    let response = state.calculators.rate(request).await?;

    Ok(Json(response))
}
