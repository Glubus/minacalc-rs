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

    // SSR: score-relative difficulty at 1.0x, score goal 93%
    let ssr = calc
        .calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Ssr)
        .expect("calc failed");

    println!("SSR @ 1.0x");
    println!("  overall:    {:.2}", ssr.overall);
    println!("  stream:     {:.2}", ssr.stream);
    println!("  jackspeed:  {:.2}", ssr.jackspeed);
    println!("  technical:  {:.2}", ssr.technical);

    // MSD: raw difficulty at 1.5x
    let msd = calc
        .calc_at_rate(&notes, 1.5, 0.93, 4, CalcMode::Msd)
        .expect("calc failed");

    println!("MSD @ 1.5x");
    println!("  overall:    {:.2}", msd.overall);
}
