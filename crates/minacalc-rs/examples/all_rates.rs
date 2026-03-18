use minacalc_rs::{Calc, CalcMode, Note};

fn main() {
    let calc = Calc::new().expect("failed to create calculator");

    let notes = vec![
        Note { notes: 0b0001, row_time: 0.0 },
        Note { notes: 0b0010, row_time: 0.15 },
        Note { notes: 0b0100, row_time: 0.30 },
        Note { notes: 0b1000, row_time: 0.45 },
        Note { notes: 0b0001, row_time: 0.60 },
        Note { notes: 0b0010, row_time: 0.75 },
    ];

    let all = calc
        .calc_all_rates(&notes, 4, CalcMode::Msd)
        .expect("calc failed");

    println!("MSD for all rates:");
    for (i, scores) in all.rates.iter().enumerate() {
        let rate = 0.7 + i as f32 * 0.1;
        println!("  {:.1}x  overall: {:.2}  stream: {:.2}", rate, scores.overall, scores.stream);
    }
}
