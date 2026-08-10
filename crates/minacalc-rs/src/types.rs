/// A single row of notes.
/// `notes` is a bitmask of active columns, `row_time` is in seconds.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Note {
    pub notes: u32,
    pub row_time: f32,
}

impl From<Note> for minacalc_sys::NoteInfo {
    fn from(n: Note) -> Self {
        minacalc_sys::NoteInfo {
            notes: n.notes,
            rowTime: n.row_time,
        }
    }
}

/// Difficulty scores for each skillset.
#[derive(Debug, Clone, Copy)]
pub struct SkillsetScores {
    pub overall: f32,
    pub stream: f32,
    pub jumpstream: f32,
    pub handstream: f32,
    pub stamina: f32,
    pub jackspeed: f32,
    pub chordjack: f32,
    pub technical: f32,
}

/// Scores plus metadata produced by the same calculation.
#[derive(Debug, Clone, Copy)]
pub struct DetailedResult {
    pub scores: SkillsetScores,
    /// Effective grind multiplier; `1.0` when it was not applied.
    pub grind_scaler: f32,
}

impl From<minacalc_sys::Ssr> for SkillsetScores {
    fn from(s: minacalc_sys::Ssr) -> Self {
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

/// Scores for all rates from 0.7x to 2.0x (14 rates, step 0.1).
#[derive(Debug, Clone, Copy)]
pub struct AllRates {
    pub rates: [SkillsetScores; 14],
}

impl From<minacalc_sys::MsdForAllRates> for AllRates {
    fn from(m: minacalc_sys::MsdForAllRates) -> Self {
        Self {
            rates: m.msds.map(SkillsetScores::from),
        }
    }
}

/// Calculation mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalcMode {
    /// Raw difficulty, uncapped.
    Msd,
    /// Score-relative difficulty, capped (score goal applies).
    Ssr,
}

impl From<CalcMode> for minacalc_sys::CalcMode {
    fn from(m: CalcMode) -> Self {
        match m {
            CalcMode::Msd => minacalc_sys::CalcMode::MSD,
            CalcMode::Ssr => minacalc_sys::CalcMode::SSR,
        }
    }
}
