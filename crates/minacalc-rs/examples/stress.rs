use minacalc_rs::{Calc, CalcMode, Note};
use std::time::Instant;

fn gen_notes(count: usize, nps: f32, jitter: f32) -> Vec<Note> {
    let step = 1.0 / nps;
    let mut seed: u64 = 0xdeadbeef;
    (0..count)
        .map(|i| {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let noise = (seed >> 33) as f32 / u32::MAX as f32 * jitter - jitter / 2.0;
            Note {
                notes: 1 << (i % 4),
                row_time: i as f32 * step + noise,
            }
        })
        .collect()
}

fn main() {
    let calc = Calc::new().unwrap();

    // realistic dense chart: 5min at 8 NPS = ~2400 notes
    let chart = gen_notes(2400, 8.0, 0.01);
    println!("=== realistic chart ({} notes, ~5min, 8 NPS) ===", chart.len());

    let t = Instant::now();
    let ssr = calc.calc_at_rate(&chart, 1.0, 0.93, 4, CalcMode::Ssr).unwrap();
    println!("calc_at_rate  1.0x SSR : overall={:.2}  ({:.2?})", ssr.overall, t.elapsed());

    let t = Instant::now();
    let all = calc.calc_all_rates(&chart, 4, CalcMode::Msd).unwrap();
    let overall_at_1x = all.rates[3].overall; // index 3 = 1.0x
    println!("calc_all_rates    MSD  : overall@1.0x={:.2}  ({:.2?})", overall_at_1x, t.elapsed());

    // stress: max valid size (~50 000s at 8 NPS = 400k notes, but C++ caps at 100k intervals = 50 000s)
    // 100k intervals × 0.5s × 8 NPS × 0.5s = ~200k notes before cap
    let stress = gen_notes(200_000, 8.0, 0.005);
    println!("\n=== stress chart ({} notes, ~7h, 8 NPS) ===", stress.len());

    let t = Instant::now();
    let ssr = calc.calc_at_rate(&stress, 1.0, 0.93, 4, CalcMode::Ssr).unwrap();
    println!("calc_at_rate  1.0x SSR : overall={:.2}  ({:.2?})", ssr.overall, t.elapsed());

    let t = Instant::now();
    let all = calc.calc_all_rates(&stress, 4, CalcMode::Msd).unwrap();
    println!("calc_all_rates    MSD  : overall@1.0x={:.2}  ({:.2?})", all.rates[3].overall, t.elapsed());
}
