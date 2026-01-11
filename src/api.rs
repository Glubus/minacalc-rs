use crate::error::MinaCalcError;
use crate::rox::RoxCalcExt;
use crate::wrapper::{AllRates, Calc, Note, SkillsetScores};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;

/// Opaque pointer to the calculator
pub struct MinaCalcHandle(Calc);

/// C-compatible version of SkillsetScores
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CMinaCalcScores {
    pub overall: f32,
    pub stream: f32,
    pub jumpstream: f32,
    pub handstream: f32,
    pub stamina: f32,
    pub jackspeed: f32,
    pub chordjack: f32,
    pub technical: f32,
}

impl From<SkillsetScores> for CMinaCalcScores {
    fn from(s: SkillsetScores) -> Self {
        Self {
            overall: s.overall,
            stream: s.stream,
            jumpstream: s.jumpstream,
            handstream: s.handstream,
            stamina: s.stamina,
            jackspeed: s.jackspeed,
            chordjack: s.chordjack,
            technical: s.technical,
        }
    }
}

/// C-compatible version of Note
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CMinaCalcNote {
    pub notes: u32,
    pub row_time: f32,
}

impl From<CMinaCalcNote> for Note {
    fn from(n: CMinaCalcNote) -> Self {
        Self {
            notes: n.notes,
            row_time: n.row_time,
        }
    }
}

#[no_mangle]
pub extern "C" fn minacalc_version() -> i32 {
    Calc::version()
}

#[no_mangle]
pub extern "C" fn minacalc_new() -> *mut MinaCalcHandle {
    match Calc::new() {
        Ok(calc) => Box::into_raw(Box::new(MinaCalcHandle(calc))),
        Err(_) => std::ptr::null_mut(),
    }
}

#[no_mangle]
pub extern "C" fn minacalc_free(handle: *mut MinaCalcHandle) {
    if !handle.is_null() {
        unsafe {
            drop(Box::from_raw(handle));
        }
    }
}

#[no_mangle]
pub extern "C" fn minacalc_calculate_ssr(
    handle: *const MinaCalcHandle,
    notes: *const CMinaCalcNote,
    notes_len: usize,
    music_rate: f32,
    score_goal: f32,
    result: *mut CMinaCalcScores,
) -> i32 {
    if handle.is_null() || notes.is_null() || result.is_null() {
        return -1;
    }

    let calc = unsafe { &(*handle).0 };
    let notes_slice = unsafe { slice::from_raw_parts(notes, notes_len) };

    // Convert C notes to Rust notes
    let rust_notes: Vec<Note> = notes_slice.iter().map(|&n| n.into()).collect();

    match calc.calc_ssr(&rust_notes, music_rate, score_goal) {
        Ok(scores) => {
            unsafe {
                *result = scores.into();
            }
            0
        }
        Err(_) => -2,
    }
}

#[repr(C)]
pub struct CMinaCalcAllRates {
    pub msds: [CMinaCalcScores; 14],
}

#[no_mangle]
pub extern "C" fn minacalc_calculate_all_rates(
    handle: *const MinaCalcHandle,
    notes: *const CMinaCalcNote,
    notes_len: usize,
    result: *mut CMinaCalcAllRates,
) -> i32 {
    if handle.is_null() || notes.is_null() || result.is_null() {
        return -1;
    }

    let calc = unsafe { &(*handle).0 };
    let notes_slice = unsafe { slice::from_raw_parts(notes, notes_len) };

    // Convert C notes to Rust notes
    let rust_notes: Vec<Note> = notes_slice.iter().map(|&n| n.into()).collect();

    match calc.calc_msd(&rust_notes) {
        Ok(all_rates) => {
            unsafe {
                for (i, scores) in all_rates.msds.iter().enumerate() {
                    (*result).msds[i] = (*scores).into();
                }
            }
            0
        }
        Err(_) => -2,
    }
}

// -------------------------------------------------------------------------
// New API methods for File/String support (ROX)
// -------------------------------------------------------------------------

#[no_mangle]
pub extern "C" fn minacalc_calculate_ssr_from_file(
    handle: *const MinaCalcHandle,
    path: *const c_char,
    music_rate: f32,
    score_goal: f32,
    result: *mut CMinaCalcScores,
) -> i32 {
    if handle.is_null() || path.is_null() || result.is_null() {
        return -1;
    }

    let calc = unsafe { &(*handle).0 };
    let c_path = unsafe { CStr::from_ptr(path) };
    let path_str = match c_path.to_str() {
        Ok(s) => s,
        Err(_) => return -2, // Invalid UTF-8
    };

    match calc.calculate_ssr_from_file(path_str, music_rate, score_goal, None) {
        Ok(scores) => {
            unsafe {
                *result = scores.into();
            }
            0
        }
        Err(_) => -3, // Calculation/IO error
    }
}

#[no_mangle]
pub extern "C" fn minacalc_calculate_all_rates_from_file(
    handle: *const MinaCalcHandle,
    path: *const c_char,
    result: *mut CMinaCalcAllRates,
) -> i32 {
    if handle.is_null() || path.is_null() || result.is_null() {
        return -1;
    }

    let calc = unsafe { &(*handle).0 };
    let c_path = unsafe { CStr::from_ptr(path) };
    let path_str = match c_path.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    match calc.calculate_all_rates_from_file(path_str) {
        Ok(all_rates) => {
            unsafe {
                for (i, scores) in all_rates.msds.iter().enumerate() {
                    (*result).msds[i] = (*scores).into();
                }
            }
            0
        }
        Err(_) => -3,
    }
}

#[no_mangle]
pub extern "C" fn minacalc_calculate_ssr_from_string(
    handle: *const MinaCalcHandle,
    content: *const c_char,
    file_hint: *const c_char,
    music_rate: f32,
    score_goal: f32,
    result: *mut CMinaCalcScores,
) -> i32 {
    if handle.is_null() || content.is_null() || result.is_null() {
        return -1;
    }

    let calc = unsafe { &(*handle).0 };

    let c_content = unsafe { CStr::from_ptr(content) };
    let content_str = match c_content.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    // file_hint can be null or empty
    let hint_str = if !file_hint.is_null() {
        let c_hint = unsafe { CStr::from_ptr(file_hint) };
        c_hint.to_str().ok()
    } else {
        None
    };

    match calc.calculate_ssr_from_string(
        content_str,
        hint_str.unwrap_or(""),
        music_rate,
        score_goal,
        None,
    ) {
        Ok(scores) => {
            unsafe {
                *result = scores.into();
            }
            0
        }
        Err(_) => -3,
    }
}

#[no_mangle]
pub extern "C" fn minacalc_calculate_all_rates_from_string(
    handle: *const MinaCalcHandle,
    content: *const c_char,
    file_hint: *const c_char,
    result: *mut CMinaCalcAllRates,
) -> i32 {
    if handle.is_null() || content.is_null() || result.is_null() {
        return -1;
    }

    let calc = unsafe { &(*handle).0 };

    let c_content = unsafe { CStr::from_ptr(content) };
    let content_str = match c_content.to_str() {
        Ok(s) => s,
        Err(_) => return -2,
    };

    // file_hint can be null or empty
    let hint_str = if !file_hint.is_null() {
        let c_hint = unsafe { CStr::from_ptr(file_hint) };
        c_hint.to_str().ok()
    } else {
        None
    };

    match calc.calculate_all_rates_from_string(content_str, hint_str.unwrap_or("")) {
        Ok(all_rates) => {
            unsafe {
                for (i, scores) in all_rates.msds.iter().enumerate() {
                    (*result).msds[i] = (*scores).into();
                }
            }
            0
        }
        Err(_) => -3,
    }
}
