# MinaCalc Rust bindings

[![Crates.io](https://img.shields.io/crates/v/minacalc-rs)](https://crates.io/crates/minacalc-rs)
[![Documentation](https://docs.rs/minacalc-rs/badge.svg)](https://docs.rs/minacalc-rs)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Rust and multi-language bindings for MinaCalc v515, the difficulty calculator
used by [Etterna](https://etternaonline.com). The original C++ calculator is
vendored and compiled as part of `minacalc-sys`.

## Workspace

| Path | Purpose |
| --- | --- |
| [`crates/minacalc-rs`](crates/minacalc-rs) | Safe, idiomatic Rust API |
| [`crates/minacalc-sys`](crates/minacalc-sys) | Raw generated Rust/C++ FFI |
| [`crates/minacalc-bindings`](crates/minacalc-bindings) | Stable C ABI for other languages |
| [`bindings`](bindings) | TypeScript, Python, C#, and Go wrappers |

## Requirements

- Rust with Cargo (edition 2021 or newer)
- A C++20 compiler
- `libclang`, used by bindgen
- Git, used to patch only the temporary native build copy

## Rust installation

```toml
[dependencies]
minacalc-rs = "515.1"
```

The major version follows the MinaCalc algorithm version: `515.x` wraps
MinaCalc v515.

## Quick start

```rust
use minacalc_rs::{Calc, CalcMode, Note};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let calc = Calc::new()?;
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

    // SSR at 1.0x with a 93% score goal.
    let ssr = calc.calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Ssr)?;
    println!("SSR overall: {:.2}", ssr.overall);

    // Raw MSD for all rates from 0.7x through 2.0x.
    let all_rates = calc.calc_all_rates(&notes, 4, CalcMode::Msd)?;
    println!("1.0x MSD overall: {:.2}", all_rates.rates[3].overall);

    Ok(())
}
```

`Note::notes` is a column bitmask. For a 4K chart, `0b0001` is the
leftmost column, `0b1000` the rightmost column, and combined bits represent a
chord. `row_time` is the absolute row time in seconds. Supported key counts are
4, 6, and 7.

## Calculation modes

| Mode | Behavior |
| --- | --- |
| `CalcMode::Ssr` | Score-relative rating. The `goal` passed to `calc_at_rate` is used. |
| `CalcMode::Msd` | Raw difficulty. The score goal is ignored. |

Every result contains `overall`, `stream`, `jumpstream`, `handstream`,
`stamina`, `jackspeed`, `chordjack`, and `technical` scores.

## Configurable calculator settings

Use one validated configuration instead of applying unrelated setters:

```rust
# use minacalc_rs::{Calc, CalcConfig, SkillsetScalers};
# fn configure() -> Result<(), minacalc_rs::Error> {
let config = CalcConfig {
    ssr_goal_cap: 1.0,
    low_acc_cutoff: 0.85,
    ssr_rating_cap: None, // no cap; default is Some(40.0)
    default_score_goal: 0.95,
    grind_scaling: false,
    skillset_scalers: SkillsetScalers { stream: 1.05, ..Default::default() },
};
let mut calc = Calc::with_config(config)?;
assert_eq!(calc.config(), config);
calc.reset_config();
# Ok(())
# }
```

`set_ssr_goal_cap(1.0)` removes the usual 96.5% SSR goal cap.
`set_low_acc_cutoff` changes the score threshold below which low-accuracy SSR
values are downscaled. `set_ssr_rating_cap` changes the cap applied to individual
SSR skillsets before overall aggregation. `set_default_score_goal` controls
`calc_all_rates` in SSR mode without affecting MSD. Disabling grind scaling
removes the short/inconsistently-dense chart penalty from SSR results.
Invalid, non-finite, negative, or out-of-range values return an error. Use
`calc_rates` for arbitrary rate lists and `calc_at_rate_detailed` to retrieve
the effective grind scaler alongside the scores.

`Calc` is not `Send` or `Sync` because the underlying C++ instance is not
thread-safe. Create one calculator per thread; see
[`examples/multithread.rs`](crates/minacalc-rs/examples/multithread.rs).

## Other language bindings

The stable C ABI powers wrappers for:

- [TypeScript (Node.js, Bun, and Deno)](bindings/typescript/README.md)
- [Python](bindings/python/README.md)
- [C#/.NET](bindings/csharp/README.md)
- [Go](bindings/go/README.md)

Build the shared native library with:

```sh
cargo build --release -p minacalc-bindings
```

See the [bindings guide](bindings/README.md) for library names and loading
instructions.

## Development

```sh
cargo test --workspace
cargo run -p minacalc-rs --example single_rate
cargo run -p minacalc-rs --example all_rates
```

The checked-in `c_code` directory stays identical to upstream MinaCalc. During
`minacalc-sys` compilation, `build.rs` copies it under Cargo's `OUT_DIR` in
`target/`, applies the versioned
[`configurable-calc.patch`](crates/minacalc-sys/patches/configurable-calc.patch),
compiles and runs bindgen against that copy, then removes the patched source
tree. After updating the vendored sources, verify that the patch still applies:

```sh
just patch-minacalc
just test
```

This never writes into the vendored source directory. A patch conflict fails the
build and identifies the patch that must be rebased. Details are in
[`patches/README.md`](patches/README.md).

## License

MIT
