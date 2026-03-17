use minacalc_rs::{Calc, CalcMode, Note};
use std::thread;

/// Calc is !Send — each thread owns its own instance.
fn main() {
    let notes = vec![
        Note { notes: 0b0001, row_time: 0.0 },
        Note { notes: 0b0010, row_time: 0.15 },
        Note { notes: 0b0100, row_time: 0.30 },
        Note { notes: 0b1000, row_time: 0.45 },
    ];

    let handles: Vec<_> = (0..4)
        .map(|i| {
            let notes = notes.clone();
            thread::spawn(move || {
                let calc = Calc::new().expect("failed to create calculator");
                let rate = 0.8 + i as f32 * 0.2;
                let scores = calc
                    .calc_at_rate(&notes, rate, 0.93, 4, CalcMode::Ssr)
                    .expect("calc failed");
                (rate, scores.overall)
            })
        })
        .collect();

    for handle in handles {
        let (rate, overall) = handle.join().expect("thread panicked");
        println!("SSR @ {:.1}x  overall: {:.2}", rate, overall);
    }
}
