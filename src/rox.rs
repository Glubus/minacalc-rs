use rhythm_open_exchange::{codec::auto_decode, RoxChart};
use std::collections::HashMap;
use std::path::Path;

use crate::error::{MinaCalcResult, RoxError, RoxResult};
use crate::{wrapper::AllRates, Calc, Note};

/// Extension trait for Calc to handle universal rhythm game chart operations
pub trait RoxCalcExt {
    /// Converts ROX chart to MinaCalc notes with optional rate
    fn chart_to_notes(chart: &RoxChart, rate: Option<f32>) -> RoxResult<Vec<Note>>;

    /// Calculates SSR (single rate) from any supported rhythm game file
    fn calculate_ssr_from_file<P: AsRef<Path>>(
        &self,
        path: P,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores>;

    /// Calculates SSR (single rate) from a loaded ROX chart
    fn calculate_ssr_from_rox_chart(
        &self,
        chart: &RoxChart,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores>;

    /// Calculates MSD for all rates (0.7x to 2.0x) from any supported rhythm game file
    fn calculate_all_rates_from_file<P: AsRef<Path>>(&self, path: P) -> MinaCalcResult<AllRates>;

    /// Calculates MSD for all rates (0.7x to 2.0x) from a loaded ROX chart
    fn calculate_all_rates_from_rox_chart(&self, chart: &RoxChart) -> MinaCalcResult<AllRates>;

    /// Validates a collection of notes
    fn validate_notes(notes: &[Note]) -> RoxResult<()>;
}

impl RoxCalcExt for Calc {
    /// Converts ROX chart to MinaCalc notes with optional rate
    fn chart_to_notes(chart: &RoxChart, rate: Option<f32>) -> RoxResult<Vec<Note>> {
        let rate = rate.unwrap_or(1.0);

        if rate <= 0.0 {
            return Err(RoxError::InvalidRate(rate));
        }

        // Use HashMap to merge notes at the same time
        let mut time_notes: HashMap<u64, u32> = HashMap::new();

        // Convert ROX notes to MinaCalc format
        for note in &chart.notes {
            // ROX uses microseconds, convert to seconds then apply rate
            let time_seconds = (note.time_us as f64 / 1_000_000.0) / rate as f64;

            // Convert back to microseconds for HashMap key (to preserve precision)
            let time_key = (time_seconds * 1_000_000.0) as u64;

            // Get column index and convert to bitflag
            // Column 0 = 0b0001, Column 1 = 0b0010, Column 2 = 0b0100, Column 3 = 0b1000
            let column_bitflag = 1u32 << note.column;

            // Merge bitflags for notes at the same time using OR operation
            time_notes
                .entry(time_key)
                .and_modify(|existing_notes| *existing_notes |= column_bitflag)
                .or_insert(column_bitflag);
        }

        if time_notes.is_empty() {
            return Err(RoxError::NoNotes);
        }

        // Convert HashMap back to sorted Vec<Note>
        let mut notes: Vec<Note> = time_notes
            .into_iter()
            .map(|(time_key, notes)| Note {
                notes,
                row_time: (time_key as f64 / 1_000_000.0) as f32,
            })
            .collect();

        // Sort by time
        notes.sort_by(|a, b| a.row_time.partial_cmp(&b.row_time).unwrap());

        // Validate all notes
        Self::validate_notes(&notes)?;

        Ok(notes)
    }

    /// Calculates SSR (single rate) from any supported rhythm game file
    fn calculate_ssr_from_file<P: AsRef<Path>>(
        &self,
        path: P,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores> {
        let path = path.as_ref();

        // Auto-decode the file (supports .osu, .sm, .rox, etc.)
        let chart = auto_decode(path)
            .map_err(|e| RoxError::DecodeFailed(format!("Failed to decode {:?}: {}", path, e)))?;

        self.calculate_ssr_from_rox_chart(&chart, music_rate, score_goal, chart_rate)
    }

    /// Calculates SSR (single rate) from a loaded ROX chart
    fn calculate_ssr_from_rox_chart(
        &self,
        chart: &RoxChart,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores> {
        // Convert chart to notes with rate
        let notes = Self::chart_to_notes(chart, chart_rate)?;

        // Get keycount from chart (auto-detected)
        let keycount = chart.key_count as u32;

        if keycount != 4 && keycount != 6 && keycount != 7 {
            return Err(crate::error::MinaCalcError::UnsupportedKeyCount(keycount));
        }

        // Calculate SSR with the detected keycount
        let ssr = self.calc_ssr_with_keycount(&notes, music_rate, score_goal, keycount)?;

        Ok(ssr)
    }

    /// Calculates MSD for all rates (0.7x to 2.0x) from any supported rhythm game file
    fn calculate_all_rates_from_file<P: AsRef<Path>>(&self, path: P) -> MinaCalcResult<AllRates> {
        let path = path.as_ref();

        // Auto-decode the file (supports .osu, .sm, .rox, etc.)
        let chart = auto_decode(path)
            .map_err(|e| RoxError::DecodeFailed(format!("Failed to decode {:?}: {}", path, e)))?;

        self.calculate_all_rates_from_rox_chart(&chart)
    }

    /// Calculates MSD for all rates (0.7x to 2.0x) from a loaded ROX chart
    fn calculate_all_rates_from_rox_chart(&self, chart: &RoxChart) -> MinaCalcResult<AllRates> {
        // Convert chart to notes with rate
        let notes = Self::chart_to_notes(chart, None)?;

        // Get keycount from chart (auto-detected)
        let keycount = chart.key_count as u32;

        if keycount != 4 && keycount != 6 && keycount != 7 {
            return Err(crate::error::MinaCalcError::UnsupportedKeyCount(keycount));
        }

        // Calculate MSD for all rates with the detected keycount
        let msd = self.calc_msd_with_keycount(&notes, keycount)?;

        Ok(msd)
    }

    fn validate_notes(notes: &[Note]) -> RoxResult<()> {
        if notes.is_empty() {
            return Err(RoxError::NoNotes);
        }

        for (i, note) in notes.iter().enumerate() {
            if note.notes == 0 {
                return Err(RoxError::InvalidNote(format!("Note {} has no columns", i)));
            }
            if note.row_time < 0.0 {
                return Err(RoxError::InvalidNote(format!(
                    "Note {} has negative time",
                    i
                )));
            }
        }

        Ok(())
    }
}

// Helper extension for Calc to support keycount parameter
impl Calc {
    /// Calculates MSD with configurable keycount
    pub fn calc_msd_with_keycount(
        &self,
        notes: &[Note],
        keycount: u32,
    ) -> MinaCalcResult<crate::wrapper::AllRates> {
        if notes.is_empty() {
            return Err(crate::error::MinaCalcError::NoNotesProvided);
        }

        if keycount != 4 && keycount != 6 && keycount != 7 {
            return Err(crate::error::MinaCalcError::UnsupportedKeyCount(keycount));
        }

        // Validate all notes
        for note in notes {
            note.validate()?;
        }

        // Convert notes to C format
        let note_infos: Vec<crate::NoteInfo> = notes.iter().map(|&note| note.into()).collect();

        let result = unsafe {
            crate::calc_msd(self.handle, note_infos.as_ptr(), note_infos.len(), keycount)
        };

        let msd: crate::wrapper::AllRates = result.into();
        msd.validate()?;
        Ok(msd)
    }

    /// Calculates SSR with configurable keycount
    pub fn calc_ssr_with_keycount(
        &self,
        notes: &[Note],
        music_rate: f32,
        score_goal: f32,
        keycount: u32,
    ) -> MinaCalcResult<crate::wrapper::SkillsetScores> {
        if notes.is_empty() {
            return Err(crate::error::MinaCalcError::NoNotesProvided);
        }

        if keycount != 4 && keycount != 6 && keycount != 7 {
            return Err(crate::error::MinaCalcError::UnsupportedKeyCount(keycount));
        }

        if music_rate <= 0.0 {
            return Err(crate::error::MinaCalcError::InvalidMusicRate(music_rate));
        }

        if score_goal <= 0.0 || score_goal > 100.0 {
            return Err(crate::error::MinaCalcError::InvalidScoreGoal(score_goal));
        }

        // Validate all notes
        for note in notes {
            note.validate()?;
        }

        // Convert notes to C format
        let mut note_infos: Vec<crate::NoteInfo> = notes.iter().map(|&note| note.into()).collect();

        let result = unsafe {
            crate::calc_ssr(
                self.handle,
                note_infos.as_mut_ptr(),
                note_infos.len(),
                music_rate,
                score_goal,
                keycount,
            )
        };

        let scores: crate::wrapper::SkillsetScores = result.into();
        scores.validate()?;
        Ok(scores)
    }
}
