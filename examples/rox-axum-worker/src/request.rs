use std::str::FromStr;

use axum::extract::{multipart::Field, Multipart};
use minacalc_rs::CalcConfig;

use crate::{
    error::ApiError,
    models::{AppState, ChartPayload, RatingMode, RatingRequest},
    osu,
};

const MAX_RATES: usize = 64;

pub(crate) async fn parse(
    mut multipart: Multipart,
    state: &AppState,
) -> Result<RatingRequest, ApiError> {
    let mut builder = RequestBuilder::default();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|error| ApiError::bad_request(format!("invalid multipart body: {error}")))?
    {
        let name = field.name().unwrap_or_default().to_owned();
        if name == "chart" {
            builder.upload = read_upload(field).await?;
        } else {
            let value = read_text(field, &name).await?;
            builder.set_text_field(&name, &value)?;
        }
    }

    builder.finish(state).await
}

struct RequestBuilder {
    upload: Option<ChartPayload>,
    osu_url: Option<String>,
    rates: Vec<f32>,
    mode: RatingMode,
    config: CalcConfig,
}

impl Default for RequestBuilder {
    fn default() -> Self {
        Self {
            upload: None,
            osu_url: None,
            rates: vec![1.0],
            mode: RatingMode::Msd,
            config: CalcConfig::default(),
        }
    }
}

impl RequestBuilder {
    fn set_text_field(&mut self, name: &str, value: &str) -> Result<(), ApiError> {
        match name {
            "osu_url" if !value.trim().is_empty() => self.osu_url = Some(value.to_owned()),
            "rates" => self.rates = parse_rates(value)?,
            "mode" => self.mode = RatingMode::from_str(value)?,
            "score_goal" => self.config.default_score_goal = parse_number(name, value)?,
            "ssr_goal_cap" => self.config.ssr_goal_cap = parse_number(name, value)?,
            "low_acc_cutoff" => self.config.low_acc_cutoff = parse_number(name, value)?,
            "ssr_rating_cap" => self.config.ssr_rating_cap = parse_optional_number(name, value)?,
            "grind_scaling" => self.config.grind_scaling = parse_bool(name, value)?,
            "scaler_stream" => self.config.skillset_scalers.stream = parse_number(name, value)?,
            "scaler_jumpstream" => {
                self.config.skillset_scalers.jumpstream = parse_number(name, value)?;
            }
            "scaler_handstream" => {
                self.config.skillset_scalers.handstream = parse_number(name, value)?;
            }
            "scaler_stamina" => self.config.skillset_scalers.stamina = parse_number(name, value)?,
            "scaler_jackspeed" => {
                self.config.skillset_scalers.jackspeed = parse_number(name, value)?;
            }
            "scaler_chordjack" => {
                self.config.skillset_scalers.chordjack = parse_number(name, value)?;
            }
            "scaler_technical" => {
                self.config.skillset_scalers.technical = parse_number(name, value)?;
            }
            _ => {}
        }
        Ok(())
    }

    async fn finish(self, state: &AppState) -> Result<RatingRequest, ApiError> {
        self.config
            .validate()
            .map_err(|error| ApiError::bad_request(error.to_string()))?;
        let chart = resolve_chart_source(self.upload, self.osu_url, state).await?;

        Ok(RatingRequest {
            chart,
            rates: self.rates,
            mode: self.mode,
            config: self.config,
        })
    }
}

async fn read_upload(field: Field<'_>) -> Result<Option<ChartPayload>, ApiError> {
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

async fn read_text(field: Field<'_>, name: &str) -> Result<String, ApiError> {
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
        .map(|value| parse_number("rate", value))
        .collect::<Result<_, _>>()?;

    if rates.is_empty() {
        return Err(ApiError::bad_request("at least one rate is required"));
    }
    if rates.len() > MAX_RATES {
        return Err(ApiError::bad_request(format!(
            "at most {MAX_RATES} rates are accepted"
        )));
    }
    if rates.iter().any(|rate| *rate <= 0.0) {
        return Err(ApiError::bad_request(
            "rates must be finite and greater than zero",
        ));
    }
    Ok(rates)
}

fn parse_number(name: &str, value: &str) -> Result<f32, ApiError> {
    let number = value
        .trim()
        .parse::<f32>()
        .map_err(|_| ApiError::bad_request(format!("{name} must be a number")))?;
    if !number.is_finite() {
        return Err(ApiError::bad_request(format!("{name} must be finite")));
    }
    Ok(number)
}

fn parse_optional_number(name: &str, value: &str) -> Result<Option<f32>, ApiError> {
    let value = value.trim();
    if value.is_empty() || value.eq_ignore_ascii_case("none") {
        Ok(None)
    } else {
        parse_number(name, value).map(Some)
    }
}

fn parse_bool(name: &str, value: &str) -> Result<bool, ApiError> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "on" => Ok(true),
        "false" | "0" | "off" => Ok(false),
        _ => Err(ApiError::bad_request(format!(
            "{name} must be true or false"
        ))),
    }
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

    #[test]
    fn builder_maps_all_calculator_settings() {
        let mut builder = RequestBuilder::default();
        builder.set_text_field("ssr_goal_cap", "1.0").unwrap();
        builder.set_text_field("low_acc_cutoff", "0.85").unwrap();
        builder.set_text_field("ssr_rating_cap", "").unwrap();
        builder.set_text_field("score_goal", "0.95").unwrap();
        builder.set_text_field("grind_scaling", "false").unwrap();
        builder.set_text_field("scaler_stream", "1.05").unwrap();

        assert_eq!(builder.config.ssr_goal_cap, 1.0);
        assert_eq!(builder.config.low_acc_cutoff, 0.85);
        assert_eq!(builder.config.ssr_rating_cap, None);
        assert_eq!(builder.config.default_score_goal, 0.95);
        assert!(!builder.config.grind_scaling);
        assert_eq!(builder.config.skillset_scalers.stream, 1.05);
    }
}
