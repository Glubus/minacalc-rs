use minacalc_rs::Note;
use rhythm_open_exchange::{NoteType, RoxChart};

use crate::error::ConversionError;

/// Converts a ROX chart to MinaCalc rows at the chart's native timestamps.
///
/// Notes at the same timestamp are merged with bitwise OR. Mines are ignored.
/// The requested music rate must be passed separately to MinaCalc; pre-scaling
/// timestamps here would apply that rate twice.
///
/// # Errors
///
/// Returns an error for an empty chart, an unsupported key count, a note outside
/// the declared columns, or a chart containing no playable notes.
pub fn chart_to_notes(chart: &RoxChart) -> Result<Vec<Note>, ConversionError> {
    validate_chart_shape(chart)?;
    let mut pairs = playable_note_pairs(chart)?;
    pairs.sort_unstable_by_key(|&(time_us, _)| time_us);
    Ok(merge_sorted_pairs_into_notes(pairs))
}

fn validate_chart_shape(chart: &RoxChart) -> Result<(), ConversionError> {
    let key_count = chart.key_count();
    if !matches!(key_count, 4 | 6 | 7) {
        return Err(ConversionError::UnsupportedKeyCount(key_count));
    }
    if chart.notes.is_empty() {
        return Err(ConversionError::EmptyChart);
    }
    Ok(())
}

fn playable_note_pairs(chart: &RoxChart) -> Result<Vec<(i64, u32)>, ConversionError> {
    let key_count = chart.key_count();
    let mut pairs = Vec::with_capacity(chart.notes.len());

    for note in &chart.notes {
        if matches!(note.note_type, NoteType::Mine) {
            continue;
        }
        if note.column >= key_count {
            return Err(ConversionError::InvalidColumn {
                column: note.column,
                key_count,
            });
        }
        pairs.push((note.time_us, 1_u32 << note.column));
    }

    if pairs.is_empty() {
        return Err(ConversionError::NoPlayableNotes);
    }
    Ok(pairs)
}

fn merge_sorted_pairs_into_notes(pairs: Vec<(i64, u32)>) -> Vec<Note> {
    let mut notes: Vec<Note> = Vec::with_capacity(pairs.len());
    let mut last_time_us = i64::MIN;

    for (time_us, column_bit) in pairs {
        if time_us == last_time_us {
            add_column_to_current_row(&mut notes, column_bit);
        } else {
            start_new_row(&mut notes, time_us, column_bit);
            last_time_us = time_us;
        }
    }
    notes
}

fn add_column_to_current_row(notes: &mut [Note], column_bit: u32) {
    if let Some(row) = notes.last_mut() {
        row.notes |= column_bit;
    }
}

fn start_new_row(notes: &mut Vec<Note>, time_us: i64, column_bit: u32) {
    notes.push(Note {
        notes: column_bit,
        row_time: time_us as f32 / 1_000_000.0,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rhythm_open_exchange::Note as RoxNote;

    #[test]
    fn sorts_merges_rows_and_ignores_mines() {
        let mut chart = RoxChart::new(4);
        chart.notes = vec![
            RoxNote::tap(2_000_000, 3),
            RoxNote::mine(1_000_000, 2),
            RoxNote::tap(1_000_000, 0),
            RoxNote::hold(1_000_000, 500_000, 1),
        ];

        let notes = chart_to_notes(&chart).expect("chart should convert");

        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].notes, 0b0011);
        assert_eq!(notes[0].row_time, 1.0);
        assert_eq!(notes[1].notes, 0b1000);
        assert_eq!(notes[1].row_time, 2.0);
    }

    #[test]
    fn rejects_unsupported_key_counts() {
        let mut chart = RoxChart::new(5);
        chart.notes.push(RoxNote::tap(0, 0));

        assert_eq!(
            chart_to_notes(&chart).unwrap_err(),
            ConversionError::UnsupportedKeyCount(5)
        );
    }
}
