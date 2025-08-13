use rosu_map::{Beatmap};
use rosu_map::section::general::GameMode;
use rosu_map::section::hit_objects::HitObject;
use rosu_map::section::hit_objects::HitObjectKind;

use minacalc_rs::{Calc, Note};

/// Converts X position of a note to bitflag for 4K
fn get_columns(x: f32) -> Result<u32, String> {
    match x {
        64.0 => Ok(1),  // bit flag 0b0001
        192.0 => Ok(2), // bit flag 0b0010
        320.0 => Ok(4), // bit flag 0b0100
        448.0 => Ok(8), // bit flag 0b1000
        _ => Err(format!("not supported columns {x}"))
    }
}

/// Converts a HitObject to Note for MinaCalc
fn hit_object_to_note(hit_object: HitObject) -> Result<Note, String> {
    let time = (hit_object.start_time as f32) / 1000.0; // Convert ms to seconds
    match hit_object.kind {
        HitObjectKind::Circle(hit_object) => Ok(Note{notes: get_columns(hit_object.pos.x)?, row_time: time}),
        HitObjectKind::Hold(hit_object) => Ok(Note{notes: get_columns(hit_object.pos_x)?, row_time: time}),
        _ => Err(format!("not supported kind {:#?}", hit_object.kind))
    }
}

/// Merges notes that have the same time by adding their bitflags
fn merge_notes_at_same_time(mut raw_notes: Vec<Note>) -> Vec<Note> {
    raw_notes.sort_by(|a, b| a.row_time.partial_cmp(&b.row_time).unwrap());
    
    let mut notes = Vec::new();
    let mut current_time = -1.0;
    let mut current_notes = 0u32;
    
    for note in &raw_notes {
        if note.row_time == current_time {
            // Same time: add bitflags
            current_notes |= note.notes;
        } else {
            // New time: save previous note and start a new one
            if current_time >= 0.0 {
                notes.push(Note {
                    notes: current_notes,
                    row_time: current_time,
                });
            }
            current_time = note.row_time;
            current_notes = note.notes;
        }
    }
    
    // Don't forget the last note
    if current_time >= 0.0 {
        notes.push(Note {
            notes: current_notes,
            row_time: current_time,
        });
    }
    
    notes
}

fn main() {
    // Load and parse the .osu file
    let map = include_str!("../assets/test.osu");
    let beatmap: Beatmap = rosu_map::from_str(map).unwrap();
    
    // Check that it's a 4K Mania map
    if beatmap.mode != GameMode::Mania {
        println!("Map is not mania");
        return;
    }
    if beatmap.circle_size != 4.0 {
        println!("Map is not 4K");
        return;
    }
    
    // Convert HitObjects to Notes
    let mut raw_notes = Vec::new();
    for hit_object in beatmap.hit_objects {
        match hit_object_to_note(hit_object) {
            Ok(note) => raw_notes.push(note),
            Err(e) => println!("Error: {}", e)
        }
    }
    
    // Merge notes that have the same time
    let notes = merge_notes_at_same_time(raw_notes);
    
    // Calculate MSD scores
    let calc = Calc::new().unwrap();
    let msd = calc.calc_msd(&notes).unwrap();
    
    println!("📊 MSD Results:");
    println!("{:?}", msd);

    
}