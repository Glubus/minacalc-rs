use std::str::FromStr;

use minacalc_rs::{CalcConfig, CalcMode, SkillsetScores};
use serde::Serialize;

use crate::error::ApiError;

#[derive(Clone)]
pub(crate) struct AppState {
    pub(crate) http: reqwest::Client,
}

pub(crate) struct ChartPayload {
    pub(crate) bytes: Vec<u8>,
    pub(crate) file_name: Option<String>,
}

pub(crate) struct RatingRequest {
    pub(crate) chart: ChartPayload,
    pub(crate) rates: Vec<f32>,
    pub(crate) mode: RatingMode,
    pub(crate) config: CalcConfig,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum RatingMode {
    Msd,
    Ssr,
}

impl RatingMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Msd => "msd",
            Self::Ssr => "ssr",
        }
    }
}

impl FromStr for RatingMode {
    type Err = ApiError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "msd" => Ok(Self::Msd),
            "ssr" => Ok(Self::Ssr),
            _ => Err(ApiError::bad_request("mode must be either msd or ssr")),
        }
    }
}

impl From<RatingMode> for CalcMode {
    fn from(mode: RatingMode) -> Self {
        match mode {
            RatingMode::Msd => Self::Msd,
            RatingMode::Ssr => Self::Ssr,
        }
    }
}

#[derive(Serialize)]
pub(crate) struct HealthResponse {
    pub(crate) status: &'static str,
}

#[derive(Serialize)]
pub(crate) struct RatingResponse {
    pub(crate) file_name: Option<String>,
    pub(crate) title: String,
    pub(crate) artist: String,
    pub(crate) difficulty: String,
    pub(crate) creator: String,
    pub(crate) cover_url: Option<String>,
    pub(crate) key_count: u8,
    pub(crate) duration_seconds: f32,
    pub(crate) source_note_count: usize,
    pub(crate) row_count: usize,
    pub(crate) mode: &'static str,
    pub(crate) score_goal: f32,
    pub(crate) results: Vec<RateResult>,
}

#[derive(Serialize)]
pub(crate) struct RateResult {
    pub(crate) rate: f32,
    pub(crate) scores: Scores,
}

#[derive(Serialize)]
pub(crate) struct Scores {
    overall: f32,
    stream: f32,
    jumpstream: f32,
    handstream: f32,
    stamina: f32,
    jackspeed: f32,
    chordjack: f32,
    technical: f32,
}

impl From<SkillsetScores> for Scores {
    fn from(scores: SkillsetScores) -> Self {
        Self {
            overall: scores.overall,
            stream: scores.stream,
            jumpstream: scores.jumpstream,
            handstream: scores.handstream,
            stamina: scores.stamina,
            jackspeed: scores.jackspeed,
            chordjack: scores.chordjack,
            technical: scores.technical,
        }
    }
}
