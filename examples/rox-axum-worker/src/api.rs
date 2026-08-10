use std::str::FromStr;

use axum::{extract::Multipart, extract::State, Json};

use crate::{
    calculator,
    error::ApiError,
    models::{AppState, ChartPayload, HealthResponse, RatingMode, RatingRequest, RatingResponse},
    osu,
};

const MAX_RATES: usize = 64;

pub(crate) async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

pub(crate) async fn rate_chart(
    State(state): State<AppState>,
    multipart: Multipart,
) -> Result<Json<RatingResponse>, ApiError> {
    let request = parse_request(multipart, &state).await?;
    let response = tokio::task::spawn_blocking(move || calculator::rate(request))
        .await
        .map_err(|error| ApiError::internal(format!("calculation worker failed: {error}")))??;

    Ok(Json(response))
}

async fn parse_request(
    mut multipart: Multipart,
    state: &AppState,
) -> Result<RatingRequest, ApiError> {
    let mut upload = None;
    let mut osu_url = None;
    let mut rates = None;
    let mut mode = RatingMode::Msd;
    let mut score_goal = 0.93;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::bad_request(format!("invalid multipart body: {error}")))?
    {
        let name = field.name().unwrap_or_default().to_owned();
        match name.as_str() {
            "chart" => upload = read_upload(field).await?,
            "osu_url" => {
                let value = read_text(field, "osu_url").await?;
                if !value.trim().is_empty() {
                    osu_url = Some(value);
                }
            }
            "rates" => rates = Some(parse_rates(&read_text(field, "rates").await?)?),
            "mode" => mode = RatingMode::from_str(&read_text(field, "mode").await?)?,
            "score_goal" => {
                score_goal = parse_score_goal(&read_text(field, "score_goal").await?)?;
            }
            _ => {}
        }
    }

    let chart = resolve_chart_source(upload, osu_url, state).await?;
    Ok(RatingRequest {
        chart,
        rates: rates.unwrap_or_else(|| vec![1.0]),
        mode,
        score_goal,
    })
}

async fn read_upload(
    field: axum::extract::multipart::Field<'_>,
) -> Result<Option<ChartPayload>, ApiError> {
    let file_name = field.file_name().map(str::to_owned);
    let bytes = field
        .bytes()
        .await
        .map_err(|error| ApiError::bad_request(format!("could not read chart: {error}")))?;

    if bytes.is_empty() {
        return Ok(None);
    }
    Ok(Some(ChartPayload {
        bytes: bytes.to_vec(),
        file_name,
    }))
}

async fn read_text(
    field: axum::extract::multipart::Field<'_>,
    name: &str,
) -> Result<String, ApiError> {
    field
        .text()
        .await
        .map_err(|error| ApiError::bad_request(format!("could not read {name}: {error}")))
}

async fn resolve_chart_source(
    upload: Option<ChartPayload>,
    osu_url: Option<String>,
    state: &AppState,
) -> Result<ChartPayload, ApiError> {
    match (upload, osu_url) {
        (Some(chart), None) => Ok(chart),
        (None, Some(url)) => osu::download(&state.http, &url).await,
        (Some(_), Some(_)) => Err(ApiError::bad_request(
            "provide either a chart file or an osu! URL, not both",
        )),
        (None, None) => Err(ApiError::bad_request(
            "provide a chart file or an osu! beatmap URL",
        )),
    }
}

fn parse_rates(value: &str) -> Result<Vec<f32>, ApiError> {
    let rates: Vec<f32> = value
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .parse::<f32>()
                .map_err(|_| ApiError::bad_request(format!("invalid rate: {value}")))
        })
        .collect::<Result<_, _>>()?;

    if rates.is_empty() {
        return Err(ApiError::bad_request("at least one rate is required"));
    }
    if rates.len() > MAX_RATES {
        return Err(ApiError::bad_request(format!(
            "at most {MAX_RATES} rates are accepted"
        )));
    }
    if rates.iter().any(|rate| !rate.is_finite() || *rate <= 0.0) {
        return Err(ApiError::bad_request(
            "rates must be finite and greater than zero",
        ));
    }
    Ok(rates)
}

fn parse_score_goal(value: &str) -> Result<f32, ApiError> {
    let goal = value
        .trim()
        .parse::<f32>()
        .map_err(|_| ApiError::bad_request("score_goal must be a number"))?;
    if !goal.is_finite() || !(0.0..=1.0).contains(&goal) {
        return Err(ApiError::bad_request(
            "score_goal must be between 0.0 and 1.0",
        ));
    }
    Ok(goal)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rate_parser_accepts_custom_rates() {
        assert_eq!(parse_rates("0.85, 1.0, 1.25").unwrap(), [0.85, 1.0, 1.25]);
    }

    #[test]
    fn rate_parser_rejects_non_positive_and_excessive_lists() {
        assert!(parse_rates("1.0, 0").is_err());
        assert!(parse_rates(&vec!["1.0"; MAX_RATES + 1].join(",")).is_err());
    }
}
