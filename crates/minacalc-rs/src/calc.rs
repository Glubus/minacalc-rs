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
