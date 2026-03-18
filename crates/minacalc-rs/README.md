# minacalc-rs

Safe Rust wrapper around [MinaCalc](https://github.com/etternagame/etterna) — the difficulty rating calculator used by [Etterna](https://etternaonline.com).

Built on top of [`minacalc-sys`](https://crates.io/crates/minacalc-sys). All unsafe FFI is handled internally.

## Versioning

Major version = MinaCalc algorithm version. `515.x.x` wraps calc v515.

## Usage

```toml
[dependencies]
minacalc-rs = "515.1.0"
```

```rust
use minacalc_rs::{Calc, CalcMode, Note};

fn main() {
    let calc = Calc::new().expect("failed to create calculator");

    let notes = vec![
        Note { notes: 0b0001, row_time: 0.0 },
        Note { notes: 0b0010, row_time: 0.15 },
        Note { notes: 0b0100, row_time: 0.30 },
        Note { notes: 0b1000, row_time: 0.45 },
    ];

    // Difficulty at 1.0x, SSR mode (capped, score goal 93%)
    let scores = calc
        .calc_at_rate(&notes, 1.0, 0.93, 4, CalcMode::Ssr)
        .unwrap();

    println!("Overall: {:.2}", scores.overall);

    // All rates from 0.7x to 2.0x
    let all = calc.calc_all_rates(&notes, 4, CalcMode::Msd).unwrap();
    for (i, s) in all.rates.iter().enumerate() {
        let rate = 0.7 + i as f32 * 0.1;
        println!("{:.1}x → {:.2}", rate, s.overall);
    }
}
```

## API

### `Note`

```rust
pub struct Note {
    pub notes: u32,    // column bitmask (bit 0 = left, bit 3 = right for 4k)
    pub row_time: f32, // timestamp in seconds
}
```

### `CalcMode`

| Variant | Description |
|---------|-------------|
| `CalcMode::Ssr` | Score-relative difficulty, capped. Requires a score goal (e.g. `0.93` for 93%). |
| `CalcMode::Msd` | Raw difficulty, uncapped. Score goal is ignored. |

### `SkillsetScores`

8 fields: `overall`, `stream`, `jumpstream`, `handstream`, `stamina`, `jackspeed`, `chordjack`, `technical`.

### `AllRates`

`rates: [SkillsetScores; 14]` — indices 0..13 map to 0.7×..2.0× in 0.1 steps.

### `Calc`

`Calc` is **not `Send`** — the underlying C++ instance is not thread-safe. Instantiate one per thread.

```rust
let calc = Calc::new()?;                                          // RAII, freed on drop
let s    = calc.calc_at_rate(&notes, rate, goal, keys, mode)?;
let all  = calc.calc_all_rates(&notes, keys, mode)?;
let ver  = Calc::version();                                       // algorithm version int
```

## Build requirements

- C++ compiler (g++ or clang++)
- `libclang` for bindgen

## License

MIT
