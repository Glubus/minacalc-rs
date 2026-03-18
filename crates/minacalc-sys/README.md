# minacalc-sys

Raw FFI bindings for [MinaCalc](https://github.com/etternagame/etterna) — the difficulty rating calculator used by [Etterna](https://etternaonline.com).

This crate is intentionally minimal: it exposes the C API surface as-is, with no safety guarantees. Everything here is `unsafe`. For a safe wrapper, see [`minacalc-rs`](../minacalc-rs) (coming soon).

## Versioning

The **major version tracks the MinaCalc algorithm version**. `515.x.x` wraps calc v515, `516.x.x` will wrap calc v516, etc. Multiple major versions can coexist in a Cargo workspace since they are semver-incompatible by design.

## What's exposed

```c
CalcHandle* create_calc();
void        destroy_calc(CalcHandle*);

SkillsetRatings calc_at_rate(CalcHandle*, NoteInfo*, size_t, float rate, float goal, uint32_t keys, CalcMode);
AllRates        calc_all_rates(CalcHandle*, NoteInfo*, size_t, uint32_t keys, CalcMode);
```

### Types

| C type | Description |
|--------|-------------|
| `CalcHandle` | Opaque calculator instance |
| `NoteInfo` | One row: `notes: u32` (column bitmask), `row_time: f32` (seconds) |
| `SkillsetRatings` | 8 floats: overall + 7 skillsets |
| `AllRates` | 14 × `SkillsetRatings` (0.7× to 2.0× in 0.1 steps) |
| `CalcMode` | `CalcMode_SSR` (capped) or `CalcMode_MSD` (uncapped) |

## Usage

```rust
use minacalc_sys::*;

fn main() {
    unsafe {
        let calc = create_calc();
        assert!(!calc.is_null());

        let notes = vec![
            NoteInfo { notes: 0b0001, row_time: 0.0 },
            NoteInfo { notes: 0b0010, row_time: 0.5 },
        ];

        let result = calc_at_rate(
            calc,
            notes.as_ptr() as *mut _,
            notes.len(),
            1.0,
            0.93,
            4,
            CalcMode_SSR,
        );

        println!("Overall: {}", result.overall);
        destroy_calc(calc);
    }
}
```

## Build requirements

- A C++ compiler (g++ or clang++) — MinaCalc source is bundled
- `libclang` for bindgen at build time

## License

MIT
