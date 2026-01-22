//! Thread-safe calculator using thread-local singleton pattern.
//!
//! When this feature is enabled, each thread gets exactly ONE `Calc` instance
//! that is reused for all calculations. This matches the C++ MinaCalc pattern.

use crate::error::{MinaCalcError, MinaCalcResult};
use crate::wrapper::{AllRates, Note, SkillsetScores};
use crate::{CalcHandle, NoteInfo};
use log::debug;
use std::cell::RefCell;

// Thread-local calculator singleton (matches C++ pattern exactly)
thread_local! {
    static THREAD_CALC: RefCell<Option<*mut CalcHandle>> = const { RefCell::new(None) };
}

/// Thread-safe calculator that uses thread-local storage.
///
/// This struct can be created multiple times, but all instances on the same
/// thread share the same underlying C++ calculator. This is the recommended
/// way to use MinaCalc in multi-threaded contexts.
///
/// # Example
/// ```ignore
/// use minacalc::thread::ThreadCalc;
///
/// // First call initializes the thread-local calculator
/// let calc = ThreadCalc::new()?;
///
/// // Subsequent calls reuse the same calculator
/// let calc2 = ThreadCalc::new()?;
///
/// // Both calc and calc2 use the same underlying C++ instance
/// ```
pub struct ThreadCalc {
    // Marker to prevent Send/Sync (thread-local only)
    _marker: std::marker::PhantomData<*mut ()>,
}

impl ThreadCalc {
    /// Creates or gets the thread-local calculator.
    ///
    /// First call on a thread initializes the calculator.
    /// Subsequent calls return immediately without allocation.
    pub fn new() -> MinaCalcResult<Self> {
        THREAD_CALC.with(|calc_cell| {
            let mut calc_ref = calc_cell.borrow_mut();

            if calc_ref.is_none() {
                debug!("Initializing thread-local Calc instance");
                let handle = unsafe { crate::create_calc() };
                if handle.is_null() {
                    return Err(MinaCalcError::CalculatorCreationFailed);
                }
                *calc_ref = Some(handle);
                debug!("Thread-local Calc initialized successfully");
            }

            Ok(ThreadCalc {
                _marker: std::marker::PhantomData,
            })
        })
    }

    /// Gets the calculator version.
    pub fn version() -> i32 {
        unsafe { crate::calc_version() }
    }

    /// Calculates SSR at a specific rate.
    pub fn calc_ssr(
        &self,
        notes: &[Note],
        music_rate: f32,
        score_goal: f32,
        key_count: u32,
    ) -> MinaCalcResult<SkillsetScores> {
        self.calc_at_rate(notes, music_rate, score_goal, key_count, true)
    }

    /// Calculates MSD at a specific rate.
    pub fn calc_msd(
        &self,
        notes: &[Note],
        music_rate: f32,
        score_goal: f32,
        key_count: u32,
    ) -> MinaCalcResult<SkillsetScores> {
        self.calc_at_rate(notes, music_rate, score_goal, key_count, false)
    }

    /// Calculates scores for a specific music rate.
    pub fn calc_at_rate(
        &self,
        notes: &[Note],
        music_rate: f32,
        score_goal: f32,
        key_count: u32,
        capped: bool,
    ) -> MinaCalcResult<SkillsetScores> {
        if notes.is_empty() {
            return Err(MinaCalcError::NoNotesProvided);
        }
        if music_rate <= 0.0 {
            return Err(MinaCalcError::InvalidMusicRate(music_rate));
        }
        if !(0.0..=1.0).contains(&score_goal) {
            return Err(MinaCalcError::InvalidScoreGoal(score_goal));
        }

        for note in notes {
            note.validate()?;
        }

        let mut note_infos: Vec<NoteInfo> = notes.iter().map(|&n| n.into()).collect();
        let cap_int = i32::from(capped);

        THREAD_CALC.with(|calc_cell| {
            let calc_ref = calc_cell.borrow();
            let handle = calc_ref.ok_or(MinaCalcError::CalculatorCreationFailed)?;

            let result = unsafe {
                crate::calc_at_rate(
                    handle,
                    note_infos.as_mut_ptr(),
                    note_infos.len(),
                    music_rate,
                    score_goal,
                    key_count,
                    cap_int,
                )
            };

            let scores: SkillsetScores = result.into();
            scores.validate()?;
            Ok(scores)
        })
    }

    /// Calculates scores for all music rates.
    pub fn calc_all_rates(
        &self,
        notes: &[Note],
        key_count: u32,
        capped: bool,
    ) -> MinaCalcResult<AllRates> {
        if notes.is_empty() {
            return Err(MinaCalcError::NoNotesProvided);
        }

        for note in notes {
            note.validate()?;
        }

        let mut note_infos: Vec<NoteInfo> = notes.iter().map(|&n| n.into()).collect();
        let cap_int = i32::from(capped);

        THREAD_CALC.with(|calc_cell| {
            let calc_ref = calc_cell.borrow();
            let handle = calc_ref.ok_or(MinaCalcError::CalculatorCreationFailed)?;

            let result = unsafe {
                crate::calc_all_rates(
                    handle,
                    note_infos.as_mut_ptr(),
                    note_infos.len(),
                    key_count,
                    cap_int,
                )
            };

            let msd: AllRates = result.into();
            msd.validate()?;
            Ok(msd)
        })
    }

    /// Checks if this thread has an initialized calculator.
    pub fn is_initialized() -> bool {
        THREAD_CALC.with(|calc_cell| calc_cell.borrow().is_some())
    }
}

// ============================================================================
// Conditional trait implementations based on features
// ============================================================================

/// When `rox` feature is enabled, implement RoxCalcExt for ThreadCalc
#[cfg(feature = "rox")]
use crate::rox::calc::high_level::RoxCalcExt;

#[cfg(feature = "rox")]
impl RoxCalcExt for ThreadCalc {
    fn calculate_at_rate_from_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
        capped: bool,
    ) -> MinaCalcResult<SkillsetScores> {
        use crate::rox::convert::chart_to_notes;
        use rhythm_open_exchange::codec::auto_decode;

        let chart = auto_decode(path.as_ref()).map_err(|e| {
            crate::error::RoxError::DecodeFailed(format!("Failed to decode: {}", e))
        })?;
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
    ) -> MinaCalcResult<SkillsetScores> {
        use rhythm_open_exchange::codec::from_string;

        let chart = from_string(content).map_err(|e| {
            crate::error::RoxError::DecodeFailed(format!("Failed to decode: {}", e))
        })?;
        self.calculate_at_rate_from_rox_chart(&chart, music_rate, score_goal, chart_rate, capped)
    }

    fn calculate_at_rate_from_rox_chart(
        &self,
        chart: &rhythm_open_exchange::RoxChart,
        music_rate: f32,
        score_goal: f32,
        chart_rate: Option<f32>,
        capped: bool,
    ) -> MinaCalcResult<SkillsetScores> {
        use crate::rox::convert::chart_to_notes;

        let notes = chart_to_notes(chart, chart_rate)?;
        let keycount = chart.key_count() as u32;

        if keycount != 4 && keycount != 6 && keycount != 7 {
            return Err(crate::error::MinaCalcError::UnsupportedKeyCount(keycount));
        }

        let mut note_infos: Vec<NoteInfo> = notes.iter().map(|&n| n.into()).collect();
        let cap_int = i32::from(capped);

        THREAD_CALC.with(|calc_cell| {
            let calc_ref = calc_cell.borrow();
            let handle = calc_ref.ok_or(MinaCalcError::CalculatorCreationFailed)?;

            let result = unsafe {
                crate::calc_at_rate(
                    handle,
                    note_infos.as_mut_ptr(),
                    note_infos.len(),
                    music_rate,
                    score_goal,
                    keycount,
                    cap_int,
                )
            };

            let scores: SkillsetScores = result.into();
            scores.validate()?;
            Ok(scores)
        })
    }

    fn calculate_all_rates_from_file<P: AsRef<std::path::Path>>(
        &self,
        path: P,
        capped: bool,
    ) -> MinaCalcResult<AllRates> {
        use rhythm_open_exchange::codec::auto_decode;

        let chart = auto_decode(path.as_ref()).map_err(|e| {
            crate::error::RoxError::DecodeFailed(format!("Failed to decode: {}", e))
        })?;
        self.calculate_all_rates_from_rox_chart(&chart, capped)
    }

    fn calculate_all_rates_from_string(
        &self,
        content: &str,
        _file_extension: &str,
        capped: bool,
    ) -> MinaCalcResult<AllRates> {
        use rhythm_open_exchange::codec::from_string;

        let chart = from_string(content).map_err(|e| {
            crate::error::RoxError::DecodeFailed(format!("Failed to decode: {}", e))
        })?;
        self.calculate_all_rates_from_rox_chart(&chart, capped)
    }

    fn calculate_all_rates_from_rox_chart(
        &self,
        chart: &rhythm_open_exchange::RoxChart,
        capped: bool,
    ) -> MinaCalcResult<AllRates> {
        use crate::rox::convert::chart_to_notes;

        let notes = chart_to_notes(chart, None)?;
        let keycount = chart.key_count() as u32;

        if keycount != 4 && keycount != 6 && keycount != 7 {
            return Err(crate::error::MinaCalcError::UnsupportedKeyCount(keycount));
        }

        let mut note_infos: Vec<NoteInfo> = notes.iter().map(|&n| n.into()).collect();
        let cap_int = i32::from(capped);

        THREAD_CALC.with(|calc_cell| {
            let calc_ref = calc_cell.borrow();
            let handle = calc_ref.ok_or(MinaCalcError::CalculatorCreationFailed)?;

            let result = unsafe {
                crate::calc_all_rates(
                    handle,
                    note_infos.as_mut_ptr(),
                    note_infos.len(),
                    keycount,
                    cap_int,
                )
            };

            let msd: AllRates = result.into();
            msd.validate()?;
            Ok(msd)
        })
    }
}

/// Convenience function: calculate SSR without creating ThreadCalc explicitly.
pub fn calc_ssr(
    notes: &[Note],
    music_rate: f32,
    score_goal: f32,
    key_count: u32,
) -> MinaCalcResult<SkillsetScores> {
    ThreadCalc::new()?.calc_ssr(notes, music_rate, score_goal, key_count)
}

/// Convenience function: calculate MSD without creating ThreadCalc explicitly.
pub fn calc_msd(
    notes: &[Note],
    music_rate: f32,
    score_goal: f32,
    key_count: u32,
) -> MinaCalcResult<SkillsetScores> {
    ThreadCalc::new()?.calc_msd(notes, music_rate, score_goal, key_count)
}

/// Convenience function: calculate all rates SSR.
pub fn calc_all_rates_ssr(notes: &[Note], key_count: u32) -> MinaCalcResult<AllRates> {
    ThreadCalc::new()?.calc_all_rates(notes, key_count, true)
}

/// Convenience function: calculate all rates MSD.
pub fn calc_all_rates_msd(notes: &[Note], key_count: u32) -> MinaCalcResult<AllRates> {
    ThreadCalc::new()?.calc_all_rates(notes, key_count, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_calc_singleton() {
        // First new
        let calc1 = ThreadCalc::new().unwrap();
        assert!(ThreadCalc::is_initialized());

        // Second new - should reuse same underlying calc
        let calc2 = ThreadCalc::new().unwrap();
        assert!(ThreadCalc::is_initialized());

        // Both should work
        let notes = vec![
            Note {
                notes: 1,
                row_time: 0.0,
            },
            Note {
                notes: 2,
                row_time: 0.5,
            },
        ];

        let r1 = calc1.calc_ssr(&notes, 1.0, 0.93);
        let r2 = calc2.calc_ssr(&notes, 1.0, 0.93);

        assert!(r1.is_ok());
        assert!(r2.is_ok());
    }

    #[test]
    fn test_multithread_isolation() {
        use std::thread;

        let handles: Vec<_> = (0..4)
            .map(|i| {
                thread::spawn(move || {
                    // Each thread gets its own calc
                    let calc = ThreadCalc::new().unwrap();
                    assert!(ThreadCalc::is_initialized());

                    let notes = vec![
                        Note {
                            notes: 1,
                            row_time: 0.0,
                        },
                        Note {
                            notes: 2,
                            row_time: 0.5 * (i as f32 + 1.0),
                        },
                    ];

                    calc.calc_ssr(&notes, 1.0, 0.93)
                })
            })
            .collect();

        for handle in handles {
            assert!(handle.join().unwrap().is_ok());
        }
    }

    #[test]
    fn test_convenience_functions() {
        let notes = vec![
            Note {
                notes: 1,
                row_time: 0.0,
            },
            Note {
                notes: 2,
                row_time: 0.5,
            },
        ];

        assert!(calc_ssr(&notes, 1.0, 0.93).is_ok());
        assert!(calc_msd(&notes, 1.0, 0.93).is_ok());
    }
}
