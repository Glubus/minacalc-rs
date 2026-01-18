//! Timing utilities for BPM-aware chart processing
//!
//! This module handles timing point extraction and precise beat-based quantization.
//! It uses a beat-space coordinate system (similar to StepMania) to ensure notes
//! are snapped to correct musical divisions (up to 192nd notes) regardless of BPM changes.

use rhythm_open_exchange::RoxChart;

/// Standard snap divisions (notes per measure)
/// 4 = quarter notes (red)
/// 8 = eighth notes (blue)
/// ...
/// 192 = very fine snap
const SNAP_DIVISOR: f64 = 192.0;

/// Represents a BPM section extracted from timing points
#[derive(Debug, Clone, Copy)]
pub struct BpmSection {
    /// Start time in microseconds
    pub start_time_us: i64,
    /// Beats per minute
    pub bpm: f32,
    /// Beat position where this section starts
    pub start_beat: f64,
}

impl BpmSection {
    /// Scales the BPM of this section by a given rate.
    pub fn scale_tempo(&mut self, rate: f32) {
        log::debug!("Scaling tempo by factor {}", rate);
        self.bpm *= rate;
    }
}

/// Extracts BPM sections from a RoxChart
pub fn extract_bpm_sections_from_chart(chart: &RoxChart) -> Vec<BpmSection> {
    // Collect and sort valid timing points
    let mut points: Vec<_> = chart
        .timing_points
        .iter()
        .filter(|tp| !tp.is_inherited && tp.bpm > 0.0)
        .collect();

    points.sort_by_key(|tp| tp.time_us);

    if points.is_empty() {
        log::debug!("No BPM sections found, using default 120 BPM");
        return vec![BpmSection {
            start_time_us: 0,
            bpm: 120.0,
            start_beat: 0.0,
        }];
    }

    let mut sections = Vec::with_capacity(points.len());

    // Handle initial section if chart starts before first timing point (rare but possible)
    // Assuming 0 offset for now or first point covers 0.
    // Usually first point is at 0 or earlier.

    // First section always starts at its defined time.
    // If it's > 0, we might need a default section before it?
    // StepMania usually assumes 60 or 120 BPM before first point, or extends first point backwards.
    // For simplicity, we just process points as they are.

    // We need to calculate start_beat for each section cumulatively
    // The first point is the anchor.

    // Pre-calculate beats for each section
    // We iterate points. For point N, its start_beat is determined by point N-1.

    // Use the first point as the base
    sections.push(BpmSection {
        start_time_us: points[0].time_us,
        bpm: points[0].bpm,
        start_beat: 0.0, // We can define the first timing point as beat 0 for relative calculation
                         // Or if we want strict SM behavior, we might need to handle negative time.
                         // For MinaCalc, consistent relative time is usually enough.
    });

    for i in 1..points.len() {
        let prev_section = &sections[i - 1];
        let curr_point = points[i];

        let delta_time = curr_point.time_us - prev_section.start_time_us;
        // duration * bpm / 60
        let delta_beats = (delta_time as f64 / 1_000_000.0) * (prev_section.bpm as f64 / 60.0);

        let new_start_beat = prev_section.start_beat + delta_beats;

        sections.push(BpmSection {
            start_time_us: curr_point.time_us,
            bpm: curr_point.bpm,
            start_beat: new_start_beat,
        });
    }

    log::debug!("Extracted {} BPM sections", sections.len());
    sections
}

/// Convert microseconds to beat position.
/// This matches the logic from the user's provided `us_to_beat` but uses our pre-calculated BpmSections.
pub fn us_to_beat(time_us: i64, sections: &[BpmSection]) -> f64 {
    if sections.is_empty() {
        return 0.0;
    }

    // Find the section that covers this time
    // Sections are sorted by time.
    // We want the last section where start_time <= time_us
    let section_idx = sections
        .partition_point(|s| s.start_time_us <= time_us)
        .saturating_sub(1);

    let section = &sections[section_idx];

    // If time is before the first section, we project backwards using first section's BPM
    // (delta will be negative)
    let delta_time = time_us - section.start_time_us;
    let delta_beats = (delta_time as f64 / 1_000_000.0) * (section.bpm as f64 / 60.0);

    section.start_beat + delta_beats
}

/// Convert beat position to microseconds.
/// Inverse of `us_to_beat`.
pub fn beat_to_us(beat: f64, sections: &[BpmSection]) -> i64 {
    if sections.is_empty() {
        return 0;
    }

    // Find section covering this beat
    // We assume sections are sorted by start_beat (which they should be if sorted by time)
    let section_idx = sections
        .partition_point(|s| s.start_beat <= beat)
        .saturating_sub(1);

    let section = &sections[section_idx];

    let delta_beats = beat - section.start_beat;
    let delta_seconds = delta_beats * (60.0 / section.bpm as f64);

    section.start_time_us + (delta_seconds * 1_000_000.0).round() as i64
}

/// Quantizes a time to the nearest 1/192nd beat (or beat-grid resolution).
/// Then returns the time in microseconds for that snapped beat.
pub fn quantize_adaptive(time_us: i64, sections: &[BpmSection]) -> i64 {
    let raw_beat = us_to_beat(time_us, sections);

    // Snap to 192nd grid
    let grid_res = SNAP_DIVISOR;
    let snapped_beat = (raw_beat * grid_res).round() / grid_res;

    let snapped_time = beat_to_us(snapped_beat, sections);

    log::trace!(
        "quantize: {}us -> beat {:.4} -> snapped {:.4} -> {}us",
        time_us,
        raw_beat,
        snapped_beat,
        snapped_time
    );

    snapped_time
}
