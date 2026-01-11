# Python Bindings for minacalc-rs

Python bindings for MinaCalc, the Etterna difficulty calculator.

## Installation

### From PyPI (when published)
```bash
pip install minacalc-rs
```

### From source
```bash
# Install maturin
pip install maturin

# Build and install
cd bindings/python
maturin develop --release
```

## Usage

```python
import minacalc_rs

# Create a calculator instance
calc = minacalc_rs.Calculator()

# Calculate SSR from a file
scores = calc.calculate_ssr_from_file("chart.osu", music_rate=1.0, score_goal=93.0)
print(f"Overall: {scores['Overall']}")
print(f"Stream: {scores['Stream']}")

# Calculate SSR from string content
with open("chart.sm", "r") as f:
    content = f.read()
scores = calc.calculate_ssr_from_string(content, "sm", 1.0, 93.0)

# Calculate MSD for all rates
all_rates = calc.calculate_all_rates_from_file("chart.osu")
print(f"MSD at 1.0x: {all_rates['1.0']['Overall']}")
print(f"MSD at 1.5x: {all_rates['1.5']['Overall']}")
```

## Supported Formats

- osu!mania (.osu)
- StepMania (.sm)
- Rhythm Open Exchange (.rox)
- Quaver (.qua)
- Friday Night Funkin' (.json)

Format detection is automatic based on file content.

## Development

```bash
# Run tests
maturin develop
python -m pytest

# Build wheel
maturin build --release

# Publish to PyPI
maturin publish
```

## License

MIT
