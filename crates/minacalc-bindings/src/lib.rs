//! Stable C ABI used by the language bindings in `../../bindings`.
//!
//! This crate is intentionally small: it owns ABI validation and conversion,
//! while all difficulty calculation remains in [`minacalc_rs`].

use std::panic::{catch_unwind, AssertUnwindSafe};

use minacalc_rs::{Calc, CalcMode, Note, SkillsetScores};

/// A note row consumed by the C ABI.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MinaCalcNote {
    pub notes: u32,
    pub row_time: f32,
}

/// Difficulty scores returned by the C ABI.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MinaCalcScores {
    pub overall: f32,
    pub stream: f32,
    pub jumpstream: f32,
    pub handstream: f32,
    pub stamina: f32,
    pub jackspeed: f32,
    pub chordjack: f32,
    pub technical: f32,
}

/// Scores for rates 0.7 through 2.0 inclusive, in increments of 0.1.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MinaCalcAllRates {
    pub rates: [MinaCalcScores; 14],
}

/// Status returned by every fallible FFI function.
#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MinaCalcStatus {
    Ok = 0,
    NullPointer = 1,
    EmptyNotes = 2,
    InvalidArgument = 3,
    AllocationFailed = 4,
    Panic = 5,
}

impl From<SkillsetScores> for MinaCalcScores {
    fn from(value: SkillsetScores) -> Self {
        Self {
            overall: value.overall,
            stream: value.stream,
            jumpstream: value.jumpstream,
            handstream: value.handstream,
            stamina: value.stamina,
            jackspeed: value.jackspeed,
            chordjack: value.chordjack,
            technical: value.technical,
        }
    }
}

fn mode_from_raw(mode: i32) -> Result<CalcMode, MinaCalcStatus> {
    match mode {
        0 => Ok(CalcMode::Msd),
        1 => Ok(CalcMode::Ssr),
        _ => Err(MinaCalcStatus::InvalidArgument),
    }
}

fn validate(notes: &[MinaCalcNote], keys: u32) -> Result<(), MinaCalcStatus> {
    if notes.is_empty() {
        return Err(MinaCalcStatus::EmptyNotes);
    }
    if !matches!(keys, 4 | 6 | 7) || notes.iter().any(|note| !note.row_time.is_finite()) {
        return Err(MinaCalcStatus::InvalidArgument);
    }
    Ok(())
}

unsafe fn notes_from_raw<'a>(
    notes: *const MinaCalcNote,
    len: usize,
) -> Result<&'a [MinaCalcNote], MinaCalcStatus> {
    if len == 0 {
        return Ok(&[]);
    }
    if notes.is_null() {
        return Err(MinaCalcStatus::NullPointer);
    }
    // SAFETY: caller supplies `len` contiguous `MinaCalcNote` values; a null
    // pointer has been rejected above. The public functions never retain it.
    Ok(unsafe { std::slice::from_raw_parts(notes, len) })
}

fn calculate_at_rate(
    notes: &[MinaCalcNote],
    rate: f32,
    goal: f32,
    keys: u32,
    mode: i32,
) -> Result<MinaCalcScores, MinaCalcStatus> {
    if !rate.is_finite() || !goal.is_finite() {
        return Err(MinaCalcStatus::InvalidArgument);
    }
    validate(notes, keys)?;
    let calc = Calc::new().map_err(|_| MinaCalcStatus::AllocationFailed)?;
    let notes: Vec<Note> = notes
        .iter()
        .map(|note| Note {
            notes: note.notes,
            row_time: note.row_time,
        })
        .collect();
    calc.calc_at_rate(&notes, rate, goal, keys, mode_from_raw(mode)?)
        .map(MinaCalcScores::from)
        .map_err(|_| MinaCalcStatus::EmptyNotes)
}

fn calculate_all_rates(
    notes: &[MinaCalcNote],
    keys: u32,
    mode: i32,
) -> Result<MinaCalcAllRates, MinaCalcStatus> {
    validate(notes, keys)?;
    let calc = Calc::new().map_err(|_| MinaCalcStatus::AllocationFailed)?;
    let notes: Vec<Note> = notes
        .iter()
        .map(|note| Note {
            notes: note.notes,
            row_time: note.row_time,
        })
        .collect();
    let all_rates = calc
        .calc_all_rates(&notes, keys, mode_from_raw(mode)?)
        .map_err(|_| MinaCalcStatus::EmptyNotes)?;
    Ok(MinaCalcAllRates {
        rates: all_rates.rates.map(MinaCalcScores::from),
    })
}

/// Returns the MinaCalc engine version.
#[no_mangle]
pub extern "C" fn minacalc_version() -> i32 {
    Calc::version()
}

/// Calculates one music rate.
///
/// `mode` is `0` for MSD and `1` for SSR. `out_scores` must point to writable
/// storage. The function never takes ownership of `notes`.
#[no_mangle]
pub unsafe extern "C" fn minacalc_calc_at_rate(
    notes: *const MinaCalcNote,
    len: usize,
    rate: f32,
    goal: f32,
    keys: u32,
    mode: i32,
    out_scores: *mut MinaCalcScores,
) -> MinaCalcStatus {
    if out_scores.is_null() {
        return MinaCalcStatus::NullPointer;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        let notes = unsafe { notes_from_raw(notes, len) }?;
        calculate_at_rate(notes, rate, goal, keys, mode)
    })) {
        Ok(Ok(scores)) => {
            // SAFETY: pointer nullability was checked and caller owns writable output.
            unsafe { out_scores.write(scores) };
            MinaCalcStatus::Ok
        }
        Ok(Err(status)) => status,
        Err(_) => MinaCalcStatus::Panic,
    }
}

/// Calculates every rate from 0.7x to 2.0x.
#[no_mangle]
pub unsafe extern "C" fn minacalc_calc_all_rates(
    notes: *const MinaCalcNote,
    len: usize,
    keys: u32,
    mode: i32,
    out_scores: *mut MinaCalcAllRates,
) -> MinaCalcStatus {
    if out_scores.is_null() {
        return MinaCalcStatus::NullPointer;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        let notes = unsafe { notes_from_raw(notes, len) }?;
        calculate_all_rates(notes, keys, mode)
    })) {
        Ok(Ok(scores)) => {
            // SAFETY: pointer nullability was checked and caller owns writable output.
            unsafe { out_scores.write(scores) };
            MinaCalcStatus::Ok
        }
        Ok(Err(status)) => status,
        Err(_) => MinaCalcStatus::Panic,
    }
}

/// Returns a static, UTF-8 error description for a status value.
#[no_mangle]
pub extern "C" fn minacalc_status_message(status: MinaCalcStatus) -> *const std::ffi::c_char {
    match status {
        MinaCalcStatus::Ok => c"ok".as_ptr(),
        MinaCalcStatus::NullPointer => c"null pointer".as_ptr(),
        MinaCalcStatus::EmptyNotes => c"notes must not be empty".as_ptr(),
        MinaCalcStatus::InvalidArgument => c"invalid calculation argument".as_ptr(),
        MinaCalcStatus::AllocationFailed => c"failed to allocate calculator".as_ptr(),
        MinaCalcStatus::Panic => c"internal calculator panic".as_ptr(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_input_without_writing_output() {
        let mut scores = MinaCalcScores {
            overall: -1.0,
            ..MinaCalcScores::default()
        };
        let status =
            unsafe { minacalc_calc_at_rate(std::ptr::null(), 0, 1.0, 0.93, 4, 0, &mut scores) };
        assert_eq!(status, MinaCalcStatus::EmptyNotes);
        assert_eq!(scores.overall, -1.0);
    }
}
