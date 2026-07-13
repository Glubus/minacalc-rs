# MinaCalc bindings

Official multi-language bindings for the MinaCalc v515 difficulty calculator.
Every package calls the `minacalc-bindings` Rust crate through the stable C ABI
in [`include/minacalc.h`](include/minacalc.h). They never use the C++ source
tree directly.

## Build and locate the native library

```sh
cargo build --release -p minacalc-bindings
```

| Platform | Output |
| --- | --- |
| Linux | `target/release/libminacalc_bindings.so` |
| macOS | `target/release/libminacalc_bindings.dylib` |
| Windows | `target/release/minacalc_bindings.dll` |

Set `MINACALC_LIBRARY_PATH` to this file's absolute path. Go and .NET can also
use their platform's normal linker and dynamic-loader search paths.

## Chart input: bitmasks

Each row has an absolute time in seconds and a `u32` bitmask of active columns.
Columns are zero-based: in 4K, `1` is left, `2` is down, `4` is up, and `8` is
right. Combine bits for chords: `5` (`0b0101`) is a left+up jump.

```text
time  bitmask  meaning
0.00  1        left
0.20  2        down
0.40  5        left + up jump
```

Rows must be non-empty and ordered by time. MinaCalc supports 4K, 6K, and 7K.

## Available bindings

- [`typescript/`](typescript/README.md): Node.js, Bun, and Deno.
- [`python/`](python/README.md): Python 3.9+ with no third-party runtime dependency.
- [`csharp/`](csharp/README.md): .NET 8+.
- [`go/`](go/README.md): Go 1.22+ with cgo.

All wrappers expose `calc_at_rate`/`CalcAtRate` and
`calc_all_rates`/`CalcAllRates`. The latter returns fourteen scores for rates
0.7x through 2.0x, in 0.1x steps.
