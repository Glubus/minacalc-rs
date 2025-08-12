# MinaCalc Rust Bindings

Rust bindings for the MinaCalc C++ library for rhythm game difficulty calculation.

## Description

This project provides safe and idiomatic Rust bindings for the MinaCalc C++ API, allowing you to calculate difficulty scores (MSD and SSR) for rhythm games like Stepmania.

## Features

- **MSD Calculation** : Difficulty scores for all music rates (0.7x to 2.0x)
- **SSR Calculation** : Difficulty scores for a specific rate and score goal
- **Idiomatic Rust Interface** : Automatic memory management and error handling
- **Memory Safety** : RAII usage to prevent memory leaks

## Installation

### Prerequisites

- Rust (version 1.70+)
- A compatible C++ compiler (GCC, Clang, or MSVC)
- MinaCalc C++ source files (`API.h`, `API.cpp`, etc.)

### Compilation

```bash
# Clone the project
git clone <repository-url>
cd minacalc-rs

# Build the project
cargo build

# Run tests
cargo test

# Run example
cargo run --example basic_usage
```

## Usage

### Basic Example

```rust
use minacalc_rs::{Calc, Note, SkillsetScores};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a calculator instance
    let calc = Calc::new()?;
    
    // Create note data
    let notes = vec![
        Note { notes: 4, row_time: 0.0 },
        Note { notes: 0, row_time: 0.5 },
        Note { notes: 4, row_time: 1.0 },
    ];
    
    // Calculate MSD scores
    let msd_results = calc.calc_msd(&notes)?;
    
    // Calculate SSR scores for 1.0x at 95%
    let ssr_scores = calc.calc_ssr(&notes, 1.0, 95.0)?;
    
    println!("Overall MSD: {:.2}", msd_results.msds[3].overall);
    println!("Overall SSR: {:.2}", ssr_scores.overall);
    
    Ok(())
}
```

### Main API

#### `Calc`

The main structure for performing calculations.

```rust
impl Calc {
    /// Creates a new calculator instance
    pub fn new() -> Result<Self, &'static str>
    
    /// Gets the calculator version
    pub fn version() -> i32
    
    /// Calculates MSD scores for all music rates
    pub fn calc_msd(&self, notes: &[Note]) -> Result<MsdForAllRates, &'static str>
    
    /// Calculates SSR scores for a specific rate and goal
    pub fn calc_ssr(&self, notes: &[Note], music_rate: f32, score_goal: f32) -> Result<SkillsetScores, &'static str>
}
```

#### `Note`

Represents a note in the rhythm game.

```rust
pub struct Note {
    pub notes: u32,      // Number of notes at this position
    pub row_time: f32,   // Row time (in seconds)
}
```

#### `SkillsetScores`

Contains difficulty scores for different skillsets.

```rust
pub struct SkillsetScores {
    pub overall: f32,
    pub stream: f32,
    pub jumpstream: f32,
    pub handstream: f32,
    pub stamina: f32,
    pub jackspeed: f32,
    pub chordjack: f32,
    pub technical: f32,
}
```

#### `MsdForAllRates`

Contains MSD scores for all music rates (0.7x to 2.0x).

```rust
pub struct MsdForAllRates {
    pub msds: [SkillsetScores; 14], // 14 rates from 0.7x to 2.0x
}
```

## Project Structure

```
minacalc-rs/
├── Cargo.toml          # Rust project configuration
├── build.rs            # C++ compilation script
├── API.h               # Original C++ header
├── API.cpp             # Original C++ implementation
├── NoteDataStructures.h # C++ data structures
├── src/
│   ├── lib.rs          # Library entry point
│   ├── bindings.rs     # Auto-generated FFI bindings
│   └── wrapper.rs      # Idiomatic Rust interface
├── examples/
│   └── basic_usage.rs  # Usage example
└── README.md           # This file
```

## Error Handling

Functions return `Result` with descriptive error messages:

- `"Failed to create calculator"` : Calculator creation failed
- `"No notes provided"` : No notes provided
- `"Music rate must be positive"` : Invalid music rate
- `"Score goal must be between 0 and 100"` : Invalid score goal

## Tests

Run tests with:

```bash
cargo test
```

Tests include:
- Calculator version verification
- Type conversion tests
- Instance creation tests

## License

MIT License - see LICENSE file for details.

## Contributing

Contributions are welcome! Feel free to:
- Report bugs
- Suggest improvements
- Submit pull requests

## Acknowledgments

This project is based on the original MinaCalc library developed for Stepmania.
