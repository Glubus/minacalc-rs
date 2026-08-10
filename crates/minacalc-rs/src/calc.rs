use crate::error::Error;
use crate::types::{AllRates, CalcMode, Note, SkillsetScores};
use minacalc_sys::CalcHandle;

/// Safe RAII wrapper around the `MinaCalc` calculator.
///
/// Not `Send` — the underlying C++ `Calc` is not thread-safe.
/// Instantiate one per thread.
pub struct Calc {
    handle: *mut CalcHandle,
}

impl Calc {
    /// # Errors
    /// Returns [`Error::AllocationFailed`] if the C++ allocator returns null.
    pub fn new() -> Result<Self, Error> {
        let handle = unsafe { minacalc_sys::create_calc() };
        if handle.is_null() {
            return Err(Error::AllocationFailed);
        }
        Ok(Self { handle })
    }

    #[must_use]
    pub fn version() -> i32 {
        unsafe { minacalc_sys::calc_version() }
    }

    /// Change the maximum score goal used for SSR calculations.
    ///
    /// The calculator defaults to Etterna's `0.965` cap. Setting this to
    /// `1.0` allows SSR calculations above 96.5%.
    pub fn set_ssr_goal_cap(&mut self, goal_cap: f32) {
        unsafe { minacalc_sys::set_ssr_goal_cap(self.handle, goal_cap) }
    }

    /// Change the score threshold below which SSR values are downscaled.
    ///
    /// The calculator defaults to Etterna's `0.9` cutoff.
    pub fn set_low_acc_cutoff(&mut self, cutoff: f32) {
        unsafe { minacalc_sys::set_low_acc_cutoff(self.handle, cutoff) }
    }

    /// Change the maximum value applied to individual SSR skillsets.
    ///
    /// The calculator defaults to Etterna's `40.0` cap. Overall is aggregated
    /// after this cap is applied and can therefore be slightly higher.
    pub fn set_ssr_rating_cap(&mut self, rating_cap: f32) {
        unsafe { minacalc_sys::set_ssr_rating_cap(self.handle, rating_cap) }
    }

    /// Change the score goal used by [`Self::calc_all_rates`] in SSR mode.
    ///
    /// The calculator defaults to Etterna's `0.93` score goal. This setting
    /// does not replace the explicit goal passed to [`Self::calc_at_rate`].
    pub fn set_default_score_goal(&mut self, score_goal: f32) {
        unsafe { minacalc_sys::set_default_score_goal(self.handle, score_goal) }
    }

    /// Enable or disable the grind scaling penalty applied to SSR results.
    ///
    /// Grind scaling is enabled by default and reduces ratings for short or
    /// inconsistently dense charts. It does not affect MSD calculations.
    pub fn set_grind_scaling_enabled(&mut self, enabled: bool) {
        unsafe { minacalc_sys::set_grind_scaling_enabled(self.handle, enabled) }
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
        if notes.is_empty() {
            return Err(Error::EmptyNotes);
        }
        let mut raw: Vec<minacalc_sys::NoteInfo> = notes.iter().map(|&n| n.into()).collect();
        let result = unsafe {
            minacalc_sys::calc_at_rate(
                self.handle,
                raw.as_mut_ptr(),
                raw.len(),
                rate,
                goal,
                keys,
                mode.into(),
            )
        };
        Ok(result.into())
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
        if notes.is_empty() {
            return Err(Error::EmptyNotes);
        }
        let raw: Vec<minacalc_sys::NoteInfo> = notes.iter().map(|&n| n.into()).collect();
        let result = unsafe {
            minacalc_sys::calc_all_rates(self.handle, raw.as_ptr(), raw.len(), keys, mode.into())
        };
        Ok(result.into())
    }
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

        calc.set_ssr_goal_cap(0.965);
        let capped = calc
            .calc_at_rate(&notes, 1.0, 1.0, 4, CalcMode::Ssr)
            .expect("capped calculation should succeed");

        calc.set_ssr_goal_cap(1.0);
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

        calc.set_low_acc_cutoff(0.9);
        let downscaled = calc
            .calc_at_rate(&notes, 1.0, 0.85, 4, CalcMode::Ssr)
            .expect("downscaled calculation should succeed");

        calc.set_low_acc_cutoff(0.85);
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

        calc.set_ssr_rating_cap(1.0);
        let capped = calc
            .calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Ssr)
            .expect("capped calculation should succeed");

        calc.set_ssr_rating_cap(40.0);
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

        calc.set_default_score_goal(0.9);
        let lower_goal = calc
            .calc_all_rates(&notes, 4, CalcMode::Ssr)
            .expect("lower-goal calculation should succeed");

        calc.set_default_score_goal(0.93);
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

        calc.set_default_score_goal(0.9);
        let msd_with_lower_default = calc
            .calc_all_rates(&notes, 4, CalcMode::Msd)
            .expect("MSD calculation should succeed");
        calc.set_default_score_goal(0.93);
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
}
