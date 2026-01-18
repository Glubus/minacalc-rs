//! Timing utilities for BPM-aware chart processing
//!
//! This module handles timing point extraction and adaptive row quantization
//! to match Etterna's internal representation (up to 192nd notes).

use rhythm_open_exchange::{RoxChart, TimingPoint};

/// Standard snap divisions in ascending order of precision
/// These match StepMania/Etterna's supported quantizations
const SNAP_DIVISIONS: [u32; 10] = [4, 8, 12, 16, 24, 32, 48, 64, 96, 192];

/// Maximum allowed timing error in microseconds for snap detection
const SNAP_TOLERANCE_US: i64 = 1000; // 1ms tolerance

/// Represents a BPM section extracted from timing points
#[derive(Debug, Clone, Copy)]
pub struct BpmSection {
    /// Start time in microseconds
    pub start_time_us: i64,
    /// Beats per minute
    pub bpm: f32,
    /// Time signature (beats per measure, e.g. 4 for 4/4)
    pub signature: u8,
}

impl BpmSection {
    /// Duration of one beat in microseconds
    #[inline]
    pub fn beat_duration_us(&self) -> f64 {
        if self.bpm <= 0.0 {
            return 60_000_000.0 / 120.0; // Default to 120 BPM
        }
        60_000_000.0 / self.bpm as f64
    }

    /// Duration of one measure in microseconds
    #[inline]
    pub fn measure_duration_us(&self) -> f64 {
        self.beat_duration_us() * self.signature as f64
    }

    /// Duration of a specific snap division in microseconds
    #[inline]
    pub fn snap_duration_us(&self, division: u32) -> f64 {
        self.measure_duration_us() / division as f64
    }
}

/// Extracts BPM sections from ROX timing points
pub fn extract_bpm_sections(timing_points: &[TimingPoint]) -> Vec<BpmSection> {
    let mut sections: Vec<BpmSection> = timing_points
        .iter()
        .filter(|tp| !tp.is_inherited && tp.bpm > 0.0)
        .map(|tp| BpmSection {
            start_time_us: tp.time_us,
            bpm: tp.bpm,
            signature: if tp.signature > 0 { tp.signature } else { 4 },
        })
        .collect();

    sections.sort_by_key(|s| s.start_time_us);

    if sections.is_empty() {
        sections.push(BpmSection {
            start_time_us: 0,
            bpm: 120.0,
            signature: 4,
        });
    }

    sections
}

/// Gets the BPM section active at a given time
pub fn bpm_section_at_time(sections: &[BpmSection], time_us: i64) -> &BpmSection {
    sections
        .iter()
        .rev()
        .find(|s| s.start_time_us <= time_us)
        .unwrap_or(&sections[0])
}

/// Finds the minimum snap division needed to accurately represent a note time.
///
/// Starts with 4ths and increases precision until the note fits within tolerance.
/// Returns the quantized time and the division used.
pub fn find_best_snap(time_us: i64, sections: &[BpmSection]) -> (i64, u32) {
    let section = bpm_section_at_time(sections, time_us);
    let offset_from_section = time_us - section.start_time_us;

    // Try each snap division from coarsest to finest
    for &division in &SNAP_DIVISIONS {
        let snap_duration = section.snap_duration_us(division);
        if snap_duration <= 0.0 {
            continue;
        }

        // Calculate which snap index this time falls on
        let snap_index = (offset_from_section as f64 / snap_duration).round() as i64;
        let quantized_offset = (snap_index as f64 * snap_duration) as i64;
        let quantized_time = section.start_time_us + quantized_offset;

        // Check if this snap is close enough
        let error = (time_us - quantized_time).abs();
        if error <= SNAP_TOLERANCE_US {
            return (quantized_time, division);
        }
    }

    // If no snap fits within tolerance, use the finest (192nd) snap
    let snap_duration = section.snap_duration_us(192);
    let snap_index = (offset_from_section as f64 / snap_duration).round() as i64;
    let quantized_offset = (snap_index as f64 * snap_duration) as i64;
    (section.start_time_us + quantized_offset, 192)
}

/// Quantizes a time to the best fitting snap based on BPM
///
/// Uses adaptive snapping to find minimum division needed.
pub fn quantize_adaptive(time_us: i64, sections: &[BpmSection]) -> i64 {
    find_best_snap(time_us, sections).0
}

/// Extracts BPM sections from a RoxChart
pub fn extract_bpm_sections_from_chart(chart: &RoxChart) -> Vec<BpmSection> {
    extract_bpm_sections(&chart.timing_points)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_120bpm_section() -> Vec<BpmSection> {
        vec![BpmSection {
            start_time_us: 0,
            bpm: 120.0,
            signature: 4,
        }]
    }

    #[test]
    fn test_snap_duration() {
        let section = BpmSection {
            start_time_us: 0,
            bpm: 120.0,
            signature: 4,
        };
        // At 120 BPM, one beat = 500ms, one measure = 2000ms
        // 4th note = 500ms, 16th = 125ms, 192nd ≈ 10.4ms
        assert!((section.snap_duration_us(4) - 500_000.0).abs() < 1.0);
        assert!((section.snap_duration_us(16) - 125_000.0).abs() < 1.0);
    }

    #[test]
    fn test_adaptive_snap_prefers_coarse() {
        let sections = make_120bpm_section();
        // A note exactly on a 4th should use 4th division
        let (_, div) = find_best_snap(500_000, &sections); // Exactly on beat 2
        assert_eq!(div, 4);
    }

    #[test]
    fn test_adaptive_snap_16th() {
        let sections = make_120bpm_section();
        // A 16th note at 125ms should use 16th or coarser
        let (quantized, _) = find_best_snap(125_000, &sections);
        assert!((quantized - 125_000).abs() <= SNAP_TOLERANCE_US);
    }
}
