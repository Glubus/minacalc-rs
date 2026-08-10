# minacalc-rs

Safe Rust wrapper around [MinaCalc](https://github.com/etternagame/etterna), the
difficulty calculator used by [Etterna](https://etternaonline.com). It builds on
`minacalc-sys`; unsafe FFI stays inside the wrapper.

## Installation

```toml
[dependencies]
minacalc-rs = "515.2"
```

The major version follows the MinaCalc algorithm version. `515.x` wraps
MinaCalc v515.

## Usage

```rust
use minacalc_rs::{Calc, CalcConfig, CalcMode, Note};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let calc = Calc::with_config(CalcConfig {
        ssr_goal_cap: 1.0,
        ssr_rating_cap: None,
        ..CalcConfig::default()
    })?;

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
- `Calc::calc_rates`: calculate an arbitrary list of positive rates.
- `Calc::calc_at_rate_detailed`: return scores and the applied grind scaler.
- `Calc::with_config`, `config`, `set_config`, and `reset_config`: manage a
  validated `CalcConfig` as one unit.
- `Calc::set_ssr_goal_cap`: change the SSR score-goal cap (`0.965` by default).
- `Calc::set_low_acc_cutoff`: change the low-accuracy downscaling threshold
  (`0.9` by default).
- `Calc::set_ssr_rating_cap`: change the per-skillset SSR rating cap (`Some(40.0)`
  by default, `None` disables it; overall aggregation happens afterward).
- `Calc::set_default_score_goal`: change the score goal used by all-rates SSR
  calculations (`0.93` by default); MSD is unaffected.
- `Calc::set_grind_scaling_enabled`: enable or disable the SSR penalty for
  short or inconsistently dense charts (enabled by default).
- `CalcMode::Ssr`: score-relative difficulty using the supplied score goal,
  capped by `ssr_goal_cap`, with low-accuracy, rating-cap, and grind processing.
- `CalcMode::Msd`: file difficulty without SSR post-processing. Standard
  all-rates MSD uses a fixed `0.93` solver target. `calc_at_rate` still uses
  goals below `0.93` in the shared solver; higher goals are capped to `0.93`.

`ssr_goal_cap`, `low_acc_cutoff`, `ssr_rating_cap`, and `grind_scaling` only
affect SSR. Music rates and skillset scalers affect both modes.

Each `SkillsetScores` value contains `overall`, `stream`, `jumpstream`,
`handstream`, `stamina`, `jackspeed`, `chordjack`, and `technical`.

`Note::notes` is a column bitmask and `Note::row_time` is an absolute time in
seconds. MinaCalc supports 4K, 6K, and 7K.

`Calc` is not `Send` or `Sync`. Create it once on the thread that will use it,
then reuse that instance for subsequent charts and rates. For parallel work,
create exactly one independent calculator per worker thread.

## Build requirements

- Rust with Cargo
- A C++20 compiler
- `libclang` for bindgen
- Git for applying the native patch inside Cargo's temporary build directory

## License

MIT
