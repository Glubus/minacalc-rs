use minacalc_rs::{Calc, CalcConfig};
use rhythm_open_exchange::from_bytes;

use crate::{
    conversion::chart_to_notes,
    error::ApiError,
    models::{RateResult, RatingRequest, RatingResponse},
};

pub(crate) fn rate(request: RatingRequest) -> Result<RatingResponse, ApiError> {
    let chart = from_bytes(&request.chart.bytes).map_err(|error| {
        ApiError::bad_request(format!("ROX could not parse the chart: {error}"))
    })?;
    let notes = chart_to_notes(&chart).map_err(|error| ApiError::bad_request(error.to_string()))?;

    let config = CalcConfig {
        default_score_goal: request.score_goal,
        ..CalcConfig::default()
    };
    let calc = Calc::with_config(config)
        .map_err(|error| ApiError::internal(format!("could not create MinaCalc: {error}")))?;
    let scores = calc
        .calc_rates(
            &notes,
            &request.rates,
            u32::from(chart.key_count()),
            request.mode.into(),
        )
        .map_err(|error| ApiError::bad_request(format!("MinaCalc rejected the chart: {error}")))?;

    let results = request
        .rates
        .into_iter()
        .zip(scores)
        .map(|(rate, scores)| RateResult {
            rate,
            scores: scores.into(),
        })
        .collect();

    Ok(RatingResponse {
        file_name: request.chart.file_name,
        title: chart.metadata.title.to_string(),
        artist: chart.metadata.artist.to_string(),
        difficulty: chart.metadata.difficulty_name.to_string(),
        key_count: chart.key_count(),
        source_note_count: chart.notes.len(),
        row_count: notes.len(),
        mode: request.mode.as_str(),
        score_goal: request.score_goal,
        results,
    })
}
