# minacalc-sys

Raw generated Rust bindings for the vendored MinaCalc v515 C++ calculator.
This crate exposes unsafe FFI directly. Prefer
[`minacalc-rs`](../minacalc-rs) for a safe Rust API.

The vendored `c_code` tree is pristine. At build time it is copied beneath
`OUT_DIR`, patched using `patches/configurable-calc.patch`, compiled, used by
bindgen, and then deleted. Published crates therefore retain both the upstream
source and the reproducible adaptations without editing either in place.

## Versioning

The major crate version follows the MinaCalc algorithm version. `515.x` wraps
MinaCalc v515.

## Exposed API

The C bridge currently exports:

```c
int calc_version(void);

CalcHandle *create_calc(void);
void destroy_calc(CalcHandle *calc);

void set_ssr_goal_cap(CalcHandle *calc, float goal_cap);
void set_low_acc_cutoff(CalcHandle *calc, float cutoff);
void set_ssr_rating_cap(CalcHandle *calc, float rating_cap);
void set_ssr_rating_cap_enabled(CalcHandle *calc, bool enabled);
void set_default_score_goal(CalcHandle *calc, float score_goal);
void set_grind_scaling_enabled(CalcHandle *calc, bool enabled);
void set_skillset_scaler(CalcHandle *calc, unsigned int skillset, float scaler);
float get_last_grind_scaler(const CalcHandle *calc);

Ssr calc_at_rate(
    CalcHandle *calc,
    const NoteInfo *rows,
    size_t num_rows,
    float music_rate,
    float score_goal,
    unsigned int keycount,
    CalcMode mode
);

void calc_rates(
    CalcHandle *calc,
    const NoteInfo *rows,
    size_t num_rows,
    const float *rates,
    size_t num_rates,
    unsigned int keycount,
    CalcMode mode,
    Ssr *out_scores
);

MsdForAllRates calc_all_rates(
    CalcHandle *calc,
    const NoteInfo *rows,
    size_t num_rows,
    unsigned int keycount,
    CalcMode mode
);
```

`Ssr` contains overall plus the seven skillsets. `MsdForAllRates` contains
fourteen `Ssr` values for rates 0.7x through 2.0x.

## Raw Rust example

```rust
use minacalc_sys::{
    calc_at_rate, create_calc, destroy_calc, set_ssr_goal_cap, CalcMode,
    NoteInfo,
};

fn main() {
    unsafe {
        let calc = create_calc();
        assert!(!calc.is_null());
        set_ssr_goal_cap(calc, 1.0);

        let mut notes = [
            NoteInfo {
                notes: 0b0001,
                rowTime: 0.0,
            },
            NoteInfo {
                notes: 0b0010,
                rowTime: 0.15,
            },
        ];

        let scores = calc_at_rate(
            calc,
            notes.as_mut_ptr(),
            notes.len(),
            1.0,
            1.0,
            4,
            CalcMode::SSR,
        );

        println!("Overall: {}", scores.overall);
        destroy_calc(calc);
    }
}
```

## Build requirements

- Rust with Cargo
- A C++20 compiler
- `libclang` for bindgen
- Git for applying `patches/configurable-calc.patch` to the temporary copy

## License

MIT
