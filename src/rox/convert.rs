use crate::error::{RoxError, RoxResult};
use crate::Note;
use rhythm_open_exchange::RoxChart;
use std::collections::HashMap;

/// Converts ROX chart to MinaCalc notes with optional rate
///
/// This function uses adaptive BPM-aware quantization to snap notes to proper
/// row positions, matching Etterna's internal note representation.
pub fn chart_to_notes(chart: &RoxChart, rate: Option<f32>) -> RoxResult<Vec<Note>> {
    use crate::timing::{extract_bpm_sections_from_chart, quantize_adaptive};

    let rate = rate.unwrap_or(1.0);

    if rate <= 0.0 {
        return Err(RoxError::InvalidRate(rate));
    }

    // Extract BPM sections for quantization
    let bpm_sections = extract_bpm_sections_from_chart(chart);

    // Use HashMap to merge notes at the same time
    let mut time_notes: HashMap<i64, u32> = HashMap::new();

    // Convert ROX notes to MinaCalc format with adaptive BPM snapping
    for note in &chart.notes {
        // Quantize note time to best-fit snap based on BPM (4th → 192nd)
        let quantized_time_us = quantize_adaptive(note.time_us, &bpm_sections);

        // Apply rate scaling (rate > 1 = faster = shorter times)
        let scaled_time_us = (quantized_time_us as f64 / rate as f64) as i64;

        // Get column index and convert to bitflag
        // Column 0 = 0b0001, Column 1 = 0b0010, Column 2 = 0b0100, etc.
        let column_bitflag = 1u32 << note.column;

        // Merge bitflags for notes at the same time using OR operation
        time_notes
            .entry(scaled_time_us)
            .and_modify(|existing_notes| *existing_notes |= column_bitflag)
            .or_insert(column_bitflag);
    }

    if time_notes.is_empty() {
        return Err(RoxError::NoNotes);
    }

    // Convert HashMap back to sorted Vec<Note>
    let mut notes: Vec<Note> = time_notes
        .into_iter()
        .map(|(time_us, notes)| Note {
            notes,
            row_time: (time_us as f32 / 1_000_000.0),
        })
        .collect();

    // Sort by time
    notes.sort_by(|a, b| a.row_time.partial_cmp(&b.row_time).unwrap());

    log::debug!(
        "Converted {} notes (0 filtered) for chart: '{}'",
        notes.len(),
        chart.metadata.title
    );
    // Validate all notes
    validate_notes(&notes)?;

    Ok(notes)
}

/// Validates a collection of notes
pub fn validate_notes(notes: &[Note]) -> RoxResult<()> {
    if notes.is_empty() {
        return Err(RoxError::NoNotes);
    }

    for (i, note) in notes.iter().enumerate() {
        if note.notes == 0 {
            return Err(RoxError::InvalidNote(format!("Note {} has no columns", i)));
        }
        if note.row_time < 0.0 {
            return Err(RoxError::InvalidNote(format!(
                "Note {} has negative time",
                i
            )));
        }
    }

    Ok(())
}
