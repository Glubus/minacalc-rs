# minacalc for Python

Typed Python 3.9+ bindings for MinaCalc. The package uses the standard
library's `ctypes`; it needs the native library built from this repository.

## Installation

```sh
cargo build --release -p minacalc-bindings
cd bindings/python
python -m pip install .
```

Tell Python where the resulting library lives:

```sh
export MINACALC_LIBRARY_PATH="$PWD/../../target/release/libminacalc_bindings.so"
```

On macOS use `libminacalc_bindings.dylib`; on Windows use
`minacalc_bindings.dll` and PowerShell's `$env:MINACALC_LIBRARY_PATH`.

## Calculate one rate

`calc_at_rate` accepts `Note` instances or `(bitmask, time_seconds)` tuples.
This 4K chart has left, down, up, then a left+up jump (`0b0101`).

```python
from minacalc import calc_at_rate

notes = [
    (0b0001, 0.00),
    (0b0010, 0.20),
    (0b0100, 0.40),
    (0b0101, 0.60),
]

scores = calc_at_rate(notes, rate=1.0, goal=0.93, keys=4, mode="ssr")
print(scores.overall, scores.technical)
```

`mode="ssr"` is score-relative difficulty; use `mode="msd"` for raw
difficulty. `goal` applies to SSR and usually is `0.93`.

## All standard rates

```python
from minacalc import calc_all_rates

all_scores = calc_all_rates(notes, keys=4, mode="msd")
one_x = all_scores[3]  # 0.7 + 3 * 0.1 = 1.0x
print(one_x.overall)
```

The return value has 14 `SkillsetScores`, for 0.7x through 2.0x.

## Errors

Invalid calculator input raises `ValueError`; a failure reported by the native
library raises `MinaCalcError`, whose `status` attribute contains the ABI code.
The input must be non-empty; `keys` must be 4, 6, or 7.
