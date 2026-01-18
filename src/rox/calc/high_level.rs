use crate::error::{MinaCalcResult, RoxError};
use crate::wrapper::AllRates;
use crate::{Calc, Note};
use rhythm_open_exchange::codec::auto_decode;
use rhythm_open_exchange::RoxChart;
use std::path::Path;

use crate::rox::convert::chart_to_notes;

/// Extension trait for Calc to handle universal rhythm game chart operations
pub trait RoxCalcExt {
    /// Helper to convert chart to notes
    fn chart_to_notes(chart: &RoxChart, rate: Option<f32>) -> crate::error::RoxResult<Vec<Note>> {
        chart_to_notes(chart, rate)
    }

    fn calculate_at_rate_from_file<P: AsRef<Path>>(
        &self,
        path: P,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
        capped: bool,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores>;

    fn calculate_at_rate_from_string(
        &self,
        content: &str,
        file_extension: &str,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
        capped: bool,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores>;

    fn calculate_at_rate_from_rox_chart(
        &self,
        chart: &RoxChart,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
        capped: bool,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores>;

    fn calculate_all_rates_from_file<P: AsRef<Path>>(
        &self,
        path: P,
        capped: bool,
    ) -> MinaCalcResult<AllRates>;

    fn calculate_all_rates_from_string(
        &self,
        content: &str,
        file_extension: &str,
        capped: bool,
    ) -> MinaCalcResult<AllRates>;

    fn calculate_all_rates_from_rox_chart(
        &self,
        chart: &RoxChart,
        capped: bool,
    ) -> MinaCalcResult<AllRates>;
}

impl RoxCalcExt for Calc {
    fn calculate_at_rate_from_file<P: AsRef<Path>>(
        &self,
        path: P,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
        capped: bool,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores> {
        let path = path.as_ref();
        log::debug!("calculate_at_rate_from_file: {:?}", path);
        let chart = auto_decode(path)
            .map_err(|e| RoxError::DecodeFailed(format!("Failed to decode {:?}: {}", path, e)))?;

        self.calculate_at_rate_from_rox_chart(&chart, music_rate, score_goal, chart_rate, capped)
    }

    fn calculate_at_rate_from_string(
        &self,
        content: &str,
        _file_extension: &str,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
        capped: bool,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores> {
        use rhythm_open_exchange::codec::from_string;
        log::debug!("calculate_at_rate_from_string (len: {})", content.len());
        let chart = from_string(content)
            .map_err(|e| RoxError::DecodeFailed(format!("Failed to decode from string: {}", e)))?;

        self.calculate_at_rate_from_rox_chart(&chart, music_rate, score_goal, chart_rate, capped)
    }

    fn calculate_at_rate_from_rox_chart(
        &self,
        chart: &RoxChart,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
        capped: bool,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores> {
        log::debug!(
            "calculate_at_rate_from_rox_chart: '{}', rate: {}, goal: {}, capped: {}",
            chart.metadata.title,
            music_rate,
            score_goal,
            capped
        );
        let notes = chart_to_notes(chart, chart_rate)?;
        let keycount = chart.key_count() as u32;

        if keycount != 4 && keycount != 6 && keycount != 7 {
            return Err(crate::error::MinaCalcError::UnsupportedKeyCount(keycount));
        }

        if music_rate <= 0.0 {
            return Err(crate::error::MinaCalcError::InvalidMusicRate(music_rate));
        }

        if capped && (score_goal <= 0.0 || score_goal > 1.0) {
            return Err(crate::error::MinaCalcError::InvalidScoreGoal(score_goal));
        }

        // Convert notes to C format
        let mut note_infos: Vec<crate::NoteInfo> = notes.iter().map(|&note| note.into()).collect();
        let cap_int = if capped { 1 } else { 0 };

        log::debug!(
            "Calling FFI calc_at_rate (ROX) with {} notes",
            note_infos.len()
        );
        let result = unsafe {
            crate::calc_at_rate(
                self.handle,
                note_infos.as_mut_ptr(),
                note_infos.len(),
                music_rate,
                score_goal,
                keycount,
                cap_int,
            )
        };

        let scores: crate::wrapper::SkillsetScores = result.into();
        scores.validate()?;
        log::debug!("calculate_at_rate_from_rox_chart success: {:?}", scores);
        Ok(scores)
    }

    fn calculate_all_rates_from_file<P: AsRef<Path>>(
        &self,
        path: P,
        capped: bool,
    ) -> MinaCalcResult<AllRates> {
        let path = path.as_ref();
        log::debug!("calculate_all_rates_from_file: {:?}", path);
        let chart = auto_decode(path)
            .map_err(|e| RoxError::DecodeFailed(format!("Failed to decode {:?}: {}", path, e)))?;

        self.calculate_all_rates_from_rox_chart(&chart, capped)
    }

    fn calculate_all_rates_from_string(
        &self,
        content: &str,
        _file_extension: &str,
        capped: bool,
    ) -> MinaCalcResult<AllRates> {
        use rhythm_open_exchange::codec::from_string;
        log::debug!("calculate_all_rates_from_string");
        let chart = from_string(content)
            .map_err(|e| RoxError::DecodeFailed(format!("Failed to decode from string: {}", e)))?;

        self.calculate_all_rates_from_rox_chart(&chart, capped)
    }

    fn calculate_all_rates_from_rox_chart(
        &self,
        chart: &RoxChart,
        capped: bool,
    ) -> MinaCalcResult<AllRates> {
        log::debug!(
            "calculate_all_rates_from_rox_chart: '{}', capped: {}",
            chart.metadata.title,
            capped
        );
        let notes = chart_to_notes(chart, None)?;
        let keycount = chart.key_count() as u32;

        if keycount != 4 && keycount != 6 && keycount != 7 {
            return Err(crate::error::MinaCalcError::UnsupportedKeyCount(keycount));
        }

        // Convert notes to C format
        let mut note_infos: Vec<crate::NoteInfo> = notes.iter().map(|&note| note.into()).collect();
        let cap_int = if capped { 1 } else { 0 };

        log::debug!(
            "Calling FFI calc_all_rates (ROX) with {} notes",
            note_infos.len()
        );
        let result = unsafe {
            crate::calc_all_rates(
                self.handle,
                note_infos.as_mut_ptr(),
                note_infos.len(),
                keycount,
                cap_int,
            )
        };

        let msd: crate::wrapper::AllRates = result.into();
        msd.validate()?;
        log::debug!("calculate_all_rates_from_rox_chart success");
        Ok(msd)
    }
}
