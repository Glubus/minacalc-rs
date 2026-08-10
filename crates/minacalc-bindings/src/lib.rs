//! Stable C ABI used by the language bindings in `../../bindings`.
//!
//! This crate is intentionally small: it owns ABI validation and conversion,
//! while all difficulty calculation remains in [`minacalc_rs`].
//! Each calling OS thread lazily owns and reuses one calculator instance.

use std::{
    cell::RefCell,
    panic::{catch_unwind, AssertUnwindSafe},
};

use minacalc_rs::{Calc, CalcConfig, CalcMode, Note, SkillsetScalers, SkillsetScores};

thread_local! {
    static THREAD_CALCULATOR: RefCell<Option<Calc>> = const { RefCell::new(None) };
}

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

/// ABI-safe calculator configuration. Use [`minacalc_default_config`].
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct MinaCalcConfig {
    pub ssr_goal_cap: f32,
    pub low_acc_cutoff: f32,
    pub ssr_rating_cap: f32,
    pub default_score_goal: f32,
    pub stream_scaler: f32,
    pub jumpstream_scaler: f32,
    pub handstream_scaler: f32,
    pub stamina_scaler: f32,
    pub jackspeed_scaler: f32,
    pub chordjack_scaler: f32,
    pub technical_scaler: f32,
    pub grind_scaling: u8,
    pub ssr_rating_cap_enabled: u8,
    pub reserved: [u8; 2],
}

/// Scores and the effective grind multiplier from one calculation.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct MinaCalcDetailedResult {
    pub scores: MinaCalcScores,
    pub grind_scaler: f32,
}

impl From<MinaCalcConfig> for CalcConfig {
    fn from(value: MinaCalcConfig) -> Self {
        Self {
            ssr_goal_cap: value.ssr_goal_cap,
            low_acc_cutoff: value.low_acc_cutoff,
            ssr_rating_cap: (value.ssr_rating_cap_enabled != 0).then_some(value.ssr_rating_cap),
            default_score_goal: value.default_score_goal,
            grind_scaling: value.grind_scaling != 0,
            skillset_scalers: SkillsetScalers {
                stream: value.stream_scaler,
                jumpstream: value.jumpstream_scaler,
                handstream: value.handstream_scaler,
                stamina: value.stamina_scaler,
                jackspeed: value.jackspeed_scaler,
                chordjack: value.chordjack_scaler,
                technical: value.technical_scaler,
            },
        }
    }
}

impl From<CalcConfig> for MinaCalcConfig {
    fn from(value: CalcConfig) -> Self {
        Self {
            ssr_goal_cap: value.ssr_goal_cap,
            low_acc_cutoff: value.low_acc_cutoff,
            ssr_rating_cap: value.ssr_rating_cap.unwrap_or(0.0),
            default_score_goal: value.default_score_goal,
            stream_scaler: value.skillset_scalers.stream,
            jumpstream_scaler: value.skillset_scalers.jumpstream,
            handstream_scaler: value.skillset_scalers.handstream,
            stamina_scaler: value.skillset_scalers.stamina,
            jackspeed_scaler: value.skillset_scalers.jackspeed,
            chordjack_scaler: value.skillset_scalers.chordjack,
            technical_scaler: value.skillset_scalers.technical,
            grind_scaling: u8::from(value.grind_scaling),
            ssr_rating_cap_enabled: u8::from(value.ssr_rating_cap.is_some()),
            reserved: [0; 2],
        }
    }
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

unsafe fn config_from_raw(config: *const MinaCalcConfig) -> Result<CalcConfig, MinaCalcStatus> {
    if config.is_null() {
        return Ok(CalcConfig::default());
    }
    // SAFETY: callers promise a readable, aligned configuration value.
    let config: CalcConfig = unsafe { config.read() }.into();
    config
        .validate()
        .map_err(|_| MinaCalcStatus::InvalidArgument)?;
    Ok(config)
}

fn with_thread_calculator<T>(
    config: CalcConfig,
    calculate: impl FnOnce(&mut Calc) -> Result<T, MinaCalcStatus>,
) -> Result<T, MinaCalcStatus> {
    THREAD_CALCULATOR.with(|slot| {
        let mut slot = slot.try_borrow_mut().map_err(|_| MinaCalcStatus::Panic)?;
        if slot.is_none() {
            let calc = Calc::new().map_err(|error| match error {
                minacalc_rs::Error::AllocationFailed => MinaCalcStatus::AllocationFailed,
                _ => MinaCalcStatus::InvalidArgument,
            })?;
            *slot = Some(calc);
        }

        let calc = slot.as_mut().ok_or(MinaCalcStatus::AllocationFailed)?;
        calc.set_config(config)
            .map_err(|_| MinaCalcStatus::InvalidArgument)?;
        calculate(calc)
    })
}

fn calculate_at_rate(
    notes: &[MinaCalcNote],
    rate: f32,
    goal: f32,
    keys: u32,
    mode: i32,
    config: CalcConfig,
) -> Result<MinaCalcScores, MinaCalcStatus> {
    if !rate.is_finite() || !goal.is_finite() {
        return Err(MinaCalcStatus::InvalidArgument);
    }
    validate(notes, keys)?;
    let notes: Vec<Note> = notes
        .iter()
        .map(|note| Note {
            notes: note.notes,
            row_time: note.row_time,
        })
        .collect();
    let mode = mode_from_raw(mode)?;
    with_thread_calculator(config, |calc| {
        calc.calc_at_rate(&notes, rate, goal, keys, mode)
            .map(MinaCalcScores::from)
            .map_err(|_| MinaCalcStatus::EmptyNotes)
    })
}

fn calculate_all_rates(
    notes: &[MinaCalcNote],
    keys: u32,
    mode: i32,
    config: CalcConfig,
) -> Result<MinaCalcAllRates, MinaCalcStatus> {
    validate(notes, keys)?;
    let notes: Vec<Note> = notes
        .iter()
        .map(|note| Note {
            notes: note.notes,
            row_time: note.row_time,
        })
        .collect();
    let mode = mode_from_raw(mode)?;
    with_thread_calculator(config, |calc| {
        let all_rates = calc
            .calc_all_rates(&notes, keys, mode)
            .map_err(|_| MinaCalcStatus::EmptyNotes)?;
        Ok(MinaCalcAllRates {
            rates: all_rates.rates.map(MinaCalcScores::from),
        })
    })
}

/// Returns the MinaCalc engine version.
#[no_mangle]
pub extern "C" fn minacalc_version() -> i32 {
    Calc::version()
}

/// Return the upstream-compatible default configuration.
#[no_mangle]
pub extern "C" fn minacalc_default_config() -> MinaCalcConfig {
    CalcConfig::default().into()
}

/// Calculates one music rate.
///
/// `mode` is `0` for MSD and `1` for SSR. `out_scores` must point to writable
/// storage. The function never takes ownership of `notes`.
///
/// # Safety
///
/// When `len` is non-zero, `notes` must point to `len` contiguous, initialized
/// [`MinaCalcNote`] values that remain readable for the duration of this call.
/// `out_scores` must be non-null, properly aligned, and valid for writing one
/// [`MinaCalcScores`] value. The pointed-to memory must not be concurrently
/// mutated while this function executes.
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
        calculate_at_rate(notes, rate, goal, keys, mode, CalcConfig::default())
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
///
/// # Safety
///
/// When `len` is non-zero, `notes` must point to `len` contiguous, initialized
/// [`MinaCalcNote`] values that remain readable for the duration of this call.
/// `out_scores` must be non-null, properly aligned, and valid for writing one
/// [`MinaCalcAllRates`] value. The pointed-to memory must not be concurrently
/// mutated while this function executes.
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
        calculate_all_rates(notes, keys, mode, CalcConfig::default())
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

/// Configurable variant of [`minacalc_calc_at_rate`]. A null config uses defaults.
///
/// # Safety
///
/// All non-null pointers must be aligned and readable/writable for their stated
/// lengths. Output storage must remain exclusively writable for the call.
#[no_mangle]
pub unsafe extern "C" fn minacalc_calc_at_rate_with_config(
    notes: *const MinaCalcNote,
    len: usize,
    rate: f32,
    goal: f32,
    keys: u32,
    mode: i32,
    config: *const MinaCalcConfig,
    out_result: *mut MinaCalcDetailedResult,
) -> MinaCalcStatus {
    if out_result.is_null() {
        return MinaCalcStatus::NullPointer;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        let notes = unsafe { notes_from_raw(notes, len) }?;
        let config = unsafe { config_from_raw(config) }?;
        if !rate.is_finite() || !goal.is_finite() {
            return Err(MinaCalcStatus::InvalidArgument);
        }
        validate(notes, keys)?;
        let notes: Vec<Note> = notes
            .iter()
            .map(|note| Note {
                notes: note.notes,
                row_time: note.row_time,
            })
            .collect();
        let mode = mode_from_raw(mode)?;
        with_thread_calculator(config, |calc| {
            let result = calc
                .calc_at_rate_detailed(&notes, rate, goal, keys, mode)
                .map_err(|_| MinaCalcStatus::InvalidArgument)?;
            Ok(MinaCalcDetailedResult {
                scores: result.scores.into(),
                grind_scaler: result.grind_scaler,
            })
        })
    })) {
        Ok(Ok(result)) => {
            unsafe { out_result.write(result) };
            MinaCalcStatus::Ok
        }
        Ok(Err(status)) => status,
        Err(_) => MinaCalcStatus::Panic,
    }
}

/// Calculate an arbitrary list of rates with one configuration.
///
/// # Safety
///
/// `notes`, `rates`, and `out_scores` must reference aligned storage containing
/// `len`, `rate_count`, and `rate_count` elements respectively. A non-null
/// `config` must point to one initialized [`MinaCalcConfig`].
#[no_mangle]
pub unsafe extern "C" fn minacalc_calc_rates(
    notes: *const MinaCalcNote,
    len: usize,
    rates: *const f32,
    rate_count: usize,
    keys: u32,
    mode: i32,
    config: *const MinaCalcConfig,
    out_scores: *mut MinaCalcScores,
) -> MinaCalcStatus {
    if rates.is_null() || out_scores.is_null() {
        return MinaCalcStatus::NullPointer;
    }
    match catch_unwind(AssertUnwindSafe(|| {
        let notes = unsafe { notes_from_raw(notes, len) }?;
        validate(notes, keys)?;
        if rate_count == 0 {
            return Err(MinaCalcStatus::InvalidArgument);
        }
        let rates = unsafe { std::slice::from_raw_parts(rates, rate_count) };
        let config = unsafe { config_from_raw(config) }?;
        let notes: Vec<Note> = notes
            .iter()
            .map(|note| Note {
                notes: note.notes,
                row_time: note.row_time,
            })
            .collect();
        let mode = mode_from_raw(mode)?;
        with_thread_calculator(config, |calc| {
            calc.calc_rates(&notes, rates, keys, mode)
                .map_err(|_| MinaCalcStatus::InvalidArgument)
        })
    })) {
        Ok(Ok(scores)) => {
            for (index, score) in scores.into_iter().enumerate() {
                unsafe { out_scores.add(index).write(score.into()) };
            }
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

    #[test]
    fn reuses_and_reconfigures_one_calculator_per_thread() {
        THREAD_CALCULATOR.with(|slot| slot.borrow_mut().take());

        let first =
            with_thread_calculator(CalcConfig::default(), |calc| Ok(calc as *mut Calc as usize))
                .unwrap();
        let config = CalcConfig {
            ssr_goal_cap: 1.0,
            ..CalcConfig::default()
        };
        let second = with_thread_calculator(config, |calc| {
            assert_eq!(calc.config(), config);
            Ok(calc as *mut Calc as usize)
        })
        .unwrap();
        let other_thread = std::thread::spawn(|| {
            with_thread_calculator(CalcConfig::default(), |calc| Ok(calc as *mut Calc as usize))
                .unwrap()
        })
        .join()
        .unwrap();

        assert_eq!(first, second);
        assert_ne!(first, other_thread);
    }
}
