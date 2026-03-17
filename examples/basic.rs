use minacalc_rs::{
    calc_all_rates, calc_at_rate, create_calc, destroy_calc, CalcMode, NoteInfo,
};

fn main() {
    // Two notes: column 1 at t=0s, column 2 at t=0.5s
    let notes = vec![
        NoteInfo { notes: 1, rowTime: 0.0 },
        NoteInfo { notes: 2, rowTime: 0.5 },
    ];

    unsafe {
        let calc = create_calc();
        assert!(!calc.is_null());

        // Single rate: 1.0x, score goal 0.93, 4K, SSR (capped)
        let ssr = calc_at_rate(
            calc,
            notes.as_ptr() as *mut _,
            notes.len(),
            1.0,              // music rate
            0.93,             // score goal
            4,                // key count
            CalcMode::CALC_MODE_SSR,
        );
        println!("SSR overall: {:.2}", ssr.overall);
        println!("  stream:    {:.2}", ssr.stream);
        println!("  jackspeed: {:.2}", ssr.jackspeed);

        // All rates (0.7x to 2.0x), MSD (uncapped)
        let all = calc_all_rates(
            calc,
            notes.as_ptr() as *mut _,
            notes.len(),
            4,                // key count
            CalcMode::CALC_MODE_MSD,
        );
        for (i, msd) in all.msds.iter().enumerate() {
            let rate = 0.7 + i as f32 * 0.1;
            println!("MSD {:.1}x overall: {:.2}", rate, msd.overall);
        }

        destroy_calc(calc);
    }
}
