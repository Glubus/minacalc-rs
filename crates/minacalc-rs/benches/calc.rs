use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use minacalc_rs::{Calc, CalcMode, Note};

/// Generate a simple stream of alternating columns at a given NPS.
fn stream(note_count: usize, nps: f32) -> Vec<Note> {
    let step = 1.0 / nps;
    (0..note_count)
        .map(|i| Note {
            notes: 1 << (i % 4),
            row_time: i as f32 * step,
        })
        .collect()
}

fn bench_calc_at_rate(c: &mut Criterion) {
    let calc = Calc::new().unwrap();
    let notes = stream(500, 8.0);

    c.bench_function("calc_at_rate/1.0x SSR", |b| {
        b.iter(|| calc.calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Ssr).unwrap());
    });

    c.bench_function("calc_at_rate/1.5x MSD", |b| {
        b.iter(|| calc.calc_at_rate(&notes, 1.5, 0.93, 4, CalcMode::Msd).unwrap());
    });
}

fn bench_calc_all_rates(c: &mut Criterion) {
    let calc = Calc::new().unwrap();
    let notes = stream(500, 8.0);

    c.bench_function("calc_all_rates/SSR", |b| {
        b.iter(|| calc.calc_all_rates(&notes, 4, CalcMode::Ssr).unwrap());
    });

    c.bench_function("calc_all_rates/MSD", |b| {
        b.iter(|| calc.calc_all_rates(&notes, 4, CalcMode::Msd).unwrap());
    });
}

fn bench_note_count_scaling(c: &mut Criterion) {
    let calc = Calc::new().unwrap();
    let mut group = c.benchmark_group("note_count_scaling");

    for count in [100, 500, 1000, 2000] {
        let notes = stream(count, 8.0);
        group.bench_with_input(BenchmarkId::new("all_rates MSD", count), &notes, |b, n| {
            b.iter(|| calc.calc_all_rates(n, 4, CalcMode::Msd).unwrap());
        });
    }

    group.finish();
}

criterion_group!(benches, bench_calc_at_rate, bench_calc_all_rates, bench_note_count_scaling);
criterion_main!(benches);
