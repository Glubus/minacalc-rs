use crate::error::Error;
use crate::types::{AllRates, CalcMode, DetailedResult, Note, SkillsetScores};
use crate::CalcConfig;
use minacalc_sys::CalcHandle;

/// Safe RAII wrapper around the `MinaCalc` calculator.
///
/// Not `Send` — the underlying C++ `Calc` is not thread-safe.
/// Instantiate one per thread.
pub struct Calc {
    handle: *mut CalcHandle,
    config: CalcConfig,
}

impl Calc {
    /// # Errors
    /// Returns [`Error::AllocationFailed`] if the C++ allocator returns null.
    pub fn new() -> Result<Self, Error> {
        Self::with_config(CalcConfig::default())
    }

    /// Create a calculator with a validated configuration.
    pub fn with_config(config: CalcConfig) -> Result<Self, Error> {
        config.validate()?;
        let handle = unsafe { minacalc_sys::create_calc() };
        if handle.is_null() {
            return Err(Error::AllocationFailed);
        }
        let mut calc = Self {
            handle,
            config: CalcConfig::default(),
        };
        calc.apply_config(config);
        Ok(calc)
    }

    #[must_use]
    pub fn version() -> i32 {
        unsafe { minacalc_sys::calc_version() }
    }

    /// Change the maximum score goal used for SSR calculations.
    ///
    /// The calculator defaults to Etterna's `0.965` cap. Setting this to
    /// `1.0` allows SSR calculations above 96.5%.
    pub fn set_ssr_goal_cap(&mut self, goal_cap: f32) -> Result<(), Error> {
        let mut config = self.config;
        config.ssr_goal_cap = goal_cap;
        self.set_config(config)
    }

    /// Change the score threshold below which SSR values are downscaled.
    ///
    /// The calculator defaults to Etterna's `0.9` cutoff.
    pub fn set_low_acc_cutoff(&mut self, cutoff: f32) -> Result<(), Error> {
        let mut config = self.config;
        config.low_acc_cutoff = cutoff;
        self.set_config(config)
    }

    /// Change the maximum value applied to individual SSR skillsets.
    ///
    /// The calculator defaults to Etterna's `40.0` cap. Overall is aggregated
    /// after this cap is applied and can therefore be slightly higher.
    pub fn set_ssr_rating_cap(&mut self, rating_cap: f32) -> Result<(), Error> {
        let mut config = self.config;
        config.ssr_rating_cap = Some(rating_cap);
        self.set_config(config)
    }

    /// Disable the maximum value applied to individual SSR skillsets.
    pub fn disable_ssr_rating_cap(&mut self) {
        let mut config = self.config;
        config.ssr_rating_cap = None;
        self.apply_config(config);
    }

    /// Change the score goal used by [`Self::calc_all_rates`] in SSR mode.
    ///
    /// The calculator defaults to Etterna's `0.93` score goal. This setting
    /// does not replace the explicit goal passed to [`Self::calc_at_rate`].
    pub fn set_default_score_goal(&mut self, score_goal: f32) -> Result<(), Error> {
        let mut config = self.config;
        config.default_score_goal = score_goal;
        self.set_config(config)
    }

    /// Enable or disable the grind scaling penalty applied to SSR results.
    ///
    /// Grind scaling is enabled by default and reduces ratings for short or
    /// inconsistently dense charts. It does not affect MSD calculations.
    pub fn set_grind_scaling_enabled(&mut self, enabled: bool) {
        let mut config = self.config;
        config.grind_scaling = enabled;
        self.apply_config(config);
    }

    /// Replace all settings atomically after validation.
    pub fn set_config(&mut self, config: CalcConfig) -> Result<(), Error> {
        config.validate()?;
        self.apply_config(config);
        Ok(())
    }

    /// Return the currently active configuration.
    #[must_use]
    pub fn config(&self) -> CalcConfig {
        self.config
    }

    #[must_use]
    pub fn ssr_goal_cap(&self) -> f32 {
        self.config.ssr_goal_cap
    }

    #[must_use]
    pub fn low_acc_cutoff(&self) -> f32 {
        self.config.low_acc_cutoff
    }

    #[must_use]
    pub fn ssr_rating_cap(&self) -> Option<f32> {
        self.config.ssr_rating_cap
    }

    #[must_use]
    pub fn default_score_goal(&self) -> f32 {
        self.config.default_score_goal
    }

    #[must_use]
    pub fn grind_scaling_enabled(&self) -> bool {
        self.config.grind_scaling
    }

    /// Restore upstream defaults.
    pub fn reset_config(&mut self) {
        self.apply_config(CalcConfig::default());
    }

    fn apply_config(&mut self, config: CalcConfig) {
        unsafe {
            minacalc_sys::set_ssr_goal_cap(self.handle, config.ssr_goal_cap);
            minacalc_sys::set_low_acc_cutoff(self.handle, config.low_acc_cutoff);
            if let Some(cap) = config.ssr_rating_cap {
                minacalc_sys::set_ssr_rating_cap(self.handle, cap);
                minacalc_sys::set_ssr_rating_cap_enabled(self.handle, true);
            } else {
                minacalc_sys::set_ssr_rating_cap_enabled(self.handle, false);
            }
            minacalc_sys::set_default_score_goal(self.handle, config.default_score_goal);
            minacalc_sys::set_grind_scaling_enabled(self.handle, config.grind_scaling);
            for (index, scaler) in config.skillset_scalers.values().into_iter().enumerate() {
                minacalc_sys::set_skillset_scaler(self.handle, (index + 1) as u32, scaler);
            }
        }
        self.config = config;
    }

    /// Calculate difficulty at a single rate.
    ///
    /// - `notes`: rows of note data
    /// - `rate`: music rate (e.g. 1.0 for 1x)
    /// - `goal`: score goal, only relevant for [`CalcMode::Ssr`] (typically 0.93)
    /// - `keys`: key count (4, 6, or 7)
    /// - `mode`: [`CalcMode::Msd`] for raw difficulty, [`CalcMode::Ssr`] for score-relative
    ///
    /// # Errors
    /// Returns [`Error::EmptyNotes`] if `notes` is empty.
    pub fn calc_at_rate(
        &self,
        notes: &[Note],
        rate: f32,
        goal: f32,
        keys: u32,
        mode: CalcMode,
    ) -> Result<SkillsetScores, Error> {
        validate_calculation(notes, rate, goal, keys)?;
        let result = unsafe {
            minacalc_sys::calc_at_rate(
                self.handle,
                notes.as_ptr().cast(),
                notes.len(),
                rate,
                goal,
                keys,
                mode.into(),
            )
        };
        Ok(result.into())
    }

    /// Calculate scores and expose the effective grind multiplier.
    pub fn calc_at_rate_detailed(
        &self,
        notes: &[Note],
        rate: f32,
        goal: f32,
        keys: u32,
        mode: CalcMode,
    ) -> Result<DetailedResult, Error> {
        let scores = self.calc_at_rate(notes, rate, goal, keys, mode)?;
        let grind_scaler = unsafe { minacalc_sys::get_last_grind_scaler(self.handle) };
        Ok(DetailedResult {
            scores,
            grind_scaler,
        })
    }

    /// Calculate a caller-supplied list of music rates.
    pub fn calc_rates(
        &self,
        notes: &[Note],
        rates: &[f32],
        keys: u32,
        mode: CalcMode,
    ) -> Result<Vec<SkillsetScores>, Error> {
        if rates.is_empty() {
            return Err(Error::EmptyRates);
        }
        validate_notes_and_keys(notes, keys)?;
        if rates.iter().any(|rate| !rate.is_finite() || *rate <= 0.0) {
            return Err(Error::InvalidArgument(
                "rates must be finite and greater than 0",
            ));
        }
        let mut output: Vec<minacalc_sys::Ssr> = (0..rates.len())
            .map(|_| unsafe { std::mem::zeroed() })
            .collect();
        unsafe {
            minacalc_sys::calc_rates(
                self.handle,
                notes.as_ptr().cast(),
                notes.len(),
                rates.as_ptr(),
                rates.len(),
                keys,
                mode.into(),
                output.as_mut_ptr(),
            );
        }
        Ok(output.into_iter().map(SkillsetScores::from).collect())
    }

    /// Calculate difficulty for all rates (0.7x to 2.0x).
    ///
    /// # Errors
    /// Returns [`Error::EmptyNotes`] if `notes` is empty.
    pub fn calc_all_rates(
        &self,
        notes: &[Note],
        keys: u32,
        mode: CalcMode,
    ) -> Result<AllRates, Error> {
        validate_notes_and_keys(notes, keys)?;
        let result = unsafe {
            minacalc_sys::calc_all_rates(
                self.handle,
                notes.as_ptr().cast(),
                notes.len(),
                keys,
                mode.into(),
            )
        };
        Ok(result.into())
    }
}

fn validate_notes_and_keys(notes: &[Note], keys: u32) -> Result<(), Error> {
    if notes.is_empty() {
        return Err(Error::EmptyNotes);
    }
    if !matches!(keys, 4 | 6 | 7) {
        return Err(Error::InvalidArgument("keys must be 4, 6, or 7"));
    }
    if notes.iter().any(|note| !note.row_time.is_finite()) {
        return Err(Error::InvalidArgument("note times must be finite"));
    }
    Ok(())
}

fn validate_calculation(notes: &[Note], rate: f32, goal: f32, keys: u32) -> Result<(), Error> {
    validate_notes_and_keys(notes, keys)?;
    if !rate.is_finite() || rate <= 0.0 {
        return Err(Error::InvalidArgument(
            "rate must be finite and greater than 0",
        ));
    }
    if !goal.is_finite() || !(0.0..=1.0).contains(&goal) {
        return Err(Error::InvalidArgument(
            "goal must be finite and between 0 and 1",
        ));
    }
    Ok(())
}

impl Drop for Calc {
    fn drop(&mut self) {
        unsafe { minacalc_sys::destroy_calc(self.handle) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stream(note_count: usize, notes_per_second: f32) -> Vec<Note> {
        let row_duration = 1.0 / notes_per_second;
        (0..note_count)
            .map(|index| Note {
                notes: 1 << (index % 4),
                row_time: index as f32 * row_duration,
            })
            .collect()
    }

    fn scores_as_array(scores: SkillsetScores) -> [f32; 8] {
        [
            scores.overall,
            scores.stream,
            scores.jumpstream,
            scores.handstream,
            scores.stamina,
            scores.jackspeed,
            scores.chordjack,
            scores.technical,
        ]
    }

    #[test]
    fn changing_ssr_goal_cap_changes_high_accuracy_scores_coherently() {
        let notes = stream(500, 8.0);
        let mut calc = Calc::new().expect("calculator allocation should succeed");

        calc.set_ssr_goal_cap(0.965).expect("valid cap");
        let capped = calc
            .calc_at_rate(&notes, 1.0, 1.0, 4, CalcMode::Ssr)
            .expect("capped calculation should succeed");

        calc.set_ssr_goal_cap(1.0).expect("valid cap");
        let uncapped = calc
            .calc_at_rate(&notes, 1.0, 1.0, 4, CalcMode::Ssr)
            .expect("uncapped calculation should succeed");

        let capped_scores = scores_as_array(capped);
        let uncapped_scores = scores_as_array(uncapped);

        assert!(
            capped_scores
                .iter()
                .chain(&uncapped_scores)
                .all(|score| score.is_finite()),
            "all skillset scores should remain finite"
        );
        assert!(
            uncapped_scores
                .iter()
                .zip(capped_scores)
                .all(|(uncapped, capped)| *uncapped >= capped),
            "a higher SSR goal cap should not lower any skillset score"
        );
        assert!(
            uncapped_scores
                .iter()
                .zip(capped_scores)
                .any(|(uncapped, capped)| *uncapped > capped),
            "changing the SSR goal cap from 0.965 to 1.0 should affect the result"
        );
    }

    #[test]
    fn changing_low_acc_cutoff_changes_low_accuracy_scores_coherently() {
        let notes = stream(500, 8.0);
        let mut calc = Calc::new().expect("calculator allocation should succeed");

        calc.set_low_acc_cutoff(0.9).expect("valid cutoff");
        let downscaled = calc
            .calc_at_rate(&notes, 1.0, 0.85, 4, CalcMode::Ssr)
            .expect("downscaled calculation should succeed");

        calc.set_low_acc_cutoff(0.85).expect("valid cutoff");
        let unchanged = calc
            .calc_at_rate(&notes, 1.0, 0.85, 4, CalcMode::Ssr)
            .expect("calculation at the cutoff should succeed");

        let downscaled_scores = scores_as_array(downscaled);
        let unchanged_scores = scores_as_array(unchanged);

        assert!(
            unchanged_scores
                .iter()
                .zip(downscaled_scores)
                .all(|(unchanged, downscaled)| *unchanged >= downscaled),
            "lowering the cutoff to the score goal should not lower any skillset score"
        );
        assert!(
            unchanged_scores
                .iter()
                .zip(downscaled_scores)
                .any(|(unchanged, downscaled)| *unchanged > downscaled),
            "changing the low accuracy cutoff should affect the result"
        );
    }

    #[test]
    fn changing_ssr_rating_cap_limits_skillset_scores() {
        let notes = stream(500, 8.0);
        let mut calc = Calc::new().expect("calculator allocation should succeed");

        calc.set_ssr_rating_cap(1.0).expect("valid cap");
        let capped = calc
            .calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Ssr)
            .expect("capped calculation should succeed");

        calc.set_ssr_rating_cap(40.0).expect("valid cap");
        let default_cap = calc
            .calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Ssr)
            .expect("default-cap calculation should succeed");

        let capped_scores = scores_as_array(capped);
        let default_scores = scores_as_array(default_cap);

        assert!(
            capped_scores[1..].iter().all(|score| *score <= 1.0),
            "the configured cap should limit every non-overall skillset"
        );
        assert!(
            default_scores
                .iter()
                .zip(capped_scores)
                .all(|(default, capped)| *default >= capped),
            "raising the SSR rating cap should not lower any score"
        );
        assert!(
            default_scores
                .iter()
                .zip(capped_scores)
                .any(|(default, capped)| *default > capped),
            "changing the SSR rating cap should affect the result"
        );
    }

    #[test]
    fn changing_default_score_goal_changes_all_rates_ssr_coherently() {
        let notes = stream(500, 8.0);
        let mut calc = Calc::new().expect("calculator allocation should succeed");

        calc.set_default_score_goal(0.9).expect("valid goal");
        let lower_goal = calc
            .calc_all_rates(&notes, 4, CalcMode::Ssr)
            .expect("lower-goal calculation should succeed");

        calc.set_default_score_goal(0.93).expect("valid goal");
        let default_goal = calc
            .calc_all_rates(&notes, 4, CalcMode::Ssr)
            .expect("default-goal calculation should succeed");

        assert!(
            default_goal
                .rates
                .iter()
                .zip(lower_goal.rates)
                .all(|(default, lower)| default.overall >= lower.overall),
            "a higher default score goal should not lower all-rates SSR"
        );
        assert!(
            default_goal
                .rates
                .iter()
                .zip(lower_goal.rates)
                .any(|(default, lower)| default.overall > lower.overall),
            "changing the default score goal should affect all-rates SSR"
        );

        calc.set_default_score_goal(0.9).expect("valid goal");
        let msd_with_lower_default = calc
            .calc_all_rates(&notes, 4, CalcMode::Msd)
            .expect("MSD calculation should succeed");
        calc.set_default_score_goal(0.93).expect("valid goal");
        let msd_with_default = calc
            .calc_all_rates(&notes, 4, CalcMode::Msd)
            .expect("MSD calculation should succeed");

        assert_eq!(
            msd_with_lower_default.rates.map(scores_as_array),
            msd_with_default.rates.map(scores_as_array),
            "the default SSR score goal should not affect MSD"
        );
    }

    #[test]
    fn disabling_grind_scaling_increases_short_chart_ssr() {
        let notes = stream(40, 8.0);
        let mut calc = Calc::new().expect("calculator allocation should succeed");

        calc.set_grind_scaling_enabled(true);
        let scaled = calc
            .calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Ssr)
            .expect("grind-scaled calculation should succeed");

        calc.set_grind_scaling_enabled(false);
        let unscaled = calc
            .calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Ssr)
            .expect("unscaled calculation should succeed");

        let scaled_scores = scores_as_array(scaled);
        let unscaled_scores = scores_as_array(unscaled);

        assert!(
            unscaled_scores
                .iter()
                .zip(scaled_scores)
                .all(|(unscaled, scaled)| *unscaled >= scaled),
            "disabling grind scaling should not lower any SSR score"
        );
        assert!(
            unscaled_scores
                .iter()
                .zip(scaled_scores)
                .any(|(unscaled, scaled)| *unscaled > scaled),
            "disabling grind scaling should affect a short chart"
        );
    }

    #[test]
    fn config_rejects_invalid_values_and_reset_restores_defaults() {
        let mut calc = Calc::new().expect("calculator allocation should succeed");
        assert!(calc.set_ssr_goal_cap(f32::NAN).is_err());
        assert!(calc.set_ssr_goal_cap(1.5).is_err());
        assert!(calc.set_low_acc_cutoff(-1.0).is_err());
        assert!(calc.set_ssr_rating_cap(f32::INFINITY).is_err());
        assert_eq!(calc.config(), CalcConfig::default());

        calc.set_ssr_goal_cap(1.0).expect("valid cap");
        calc.disable_ssr_rating_cap();
        assert_eq!(calc.config().ssr_rating_cap, None);
        calc.reset_config();
        assert_eq!(calc.config(), CalcConfig::default());
    }

    #[test]
    fn custom_rates_match_individual_calculations() {
        let notes = stream(200, 8.0);
        let calc = Calc::new().expect("calculator allocation should succeed");
        let rates = [0.85, 1.0, 1.25, 1.5];
        let batch = calc
            .calc_rates(&notes, &rates, 4, CalcMode::Msd)
            .expect("batch calculation should succeed");

        for (rate, batch_scores) in rates.into_iter().zip(batch) {
            let individual = calc
                .calc_at_rate(&notes, rate, 0.93, 4, CalcMode::Msd)
                .expect("individual calculation should succeed");
            assert_eq!(scores_as_array(batch_scores), scores_as_array(individual));
        }
    }

    #[test]
    fn detailed_result_reports_effective_grind_scaler() {
        let notes = stream(40, 8.0);
        let mut calc = Calc::new().expect("calculator allocation should succeed");
        let scaled = calc
            .calc_at_rate_detailed(&notes, 1.0, 0.93, 4, CalcMode::Ssr)
            .expect("detailed calculation should succeed");
        assert!((0.0..=1.0).contains(&scaled.grind_scaler));

        calc.set_grind_scaling_enabled(false);
        let disabled = calc
            .calc_at_rate_detailed(&notes, 1.0, 0.93, 4, CalcMode::Ssr)
            .expect("detailed calculation should succeed");
        assert_eq!(disabled.grind_scaler, 1.0);
    }

    #[test]
    fn skillset_scaler_changes_only_the_expected_direction() {
        let notes = stream(500, 8.0);
        let baseline = Calc::new()
            .expect("calculator allocation should succeed")
            .calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Msd)
            .expect("baseline calculation should succeed");
        let mut config = CalcConfig::default();
        config.skillset_scalers.stream = 1.1;
        let configured = Calc::with_config(config)
            .expect("configuration should be valid")
            .calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Msd)
            .expect("configured calculation should succeed");
        assert!(configured.stream > baseline.stream);
        assert!(configured.overall >= baseline.overall);
    }
}
