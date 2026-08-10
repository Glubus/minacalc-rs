# minacalc-rs

Safe Rust wrapper around [MinaCalc](https://github.com/etternagame/etterna), the
difficulty calculator used by [Etterna](https://etternaonline.com). It builds on
`minacalc-sys`; unsafe FFI stays inside the wrapper.

## Installation

```toml
[dependencies]
minacalc-rs = "515.1"
```

The major version follows the MinaCalc algorithm version. `515.x` wraps
MinaCalc v515.

## Usage

```rust
use minacalc_rs::{Calc, CalcMode, Note};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut calc = Calc::new()?;

    // Optional per-instance overrides.
    calc.set_ssr_goal_cap(1.0);
    calc.set_low_acc_cutoff(0.85);
    calc.set_ssr_rating_cap(100.0);
    calc.set_default_score_goal(0.95);
    calc.set_grind_scaling_enabled(false);

    let notes = vec![
        Note {
            notes: 0b0001,
            row_time: 0.00,
        },
        Note {
            notes: 0b0010,
            row_time: 0.15,
        },
        Note {
            notes: 0b0100,
            row_time: 0.30,
        },
        Note {
            notes: 0b1000,
            row_time: 0.45,
        },
    ];

    let ssr = calc.calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Ssr)?;
    println!("SSR overall: {:.2}", ssr.overall);

    let all_rates = calc.calc_all_rates(&notes, 4, CalcMode::Msd)?;
    println!("1.0x MSD overall: {:.2}", all_rates.rates[3].overall);

    Ok(())
}
```

## API summary

- `Calc::calc_at_rate`: calculate one music rate.
- `Calc::calc_all_rates`: calculate fourteen rates from 0.7x through 2.0x.
- `Calc::set_ssr_goal_cap`: change the SSR score-goal cap (`0.965` by default).
- `Calc::set_low_acc_cutoff`: change the low-accuracy downscaling threshold
  (`0.9` by default).
- `Calc::set_ssr_rating_cap`: change the per-skillset SSR rating cap (`40.0` by
  default; overall aggregation happens afterward).
- `Calc::set_default_score_goal`: change the score goal used by all-rates SSR
  calculations (`0.93` by default); MSD is unaffected.
- `Calc::set_grind_scaling_enabled`: enable or disable the SSR penalty for
  short or inconsistently dense charts (enabled by default).
- `CalcMode::Ssr`: score-relative difficulty using the supplied score goal.
- `CalcMode::Msd`: raw difficulty; the score goal is ignored.

Each `SkillsetScores` value contains `overall`, `stream`, `jumpstream`,
`handstream`, `stamina`, `jackspeed`, `chordjack`, and `technical`.

`Note::notes` is a column bitmask and `Note::row_time` is an absolute time in
seconds. MinaCalc supports 4K, 6K, and 7K.

`Calc` is not `Send` or `Sync`; create one calculator per thread.

## Build requirements

- Rust with Cargo
- A C++20 compiler
- `libclang` for bindgen

## License

MIT
