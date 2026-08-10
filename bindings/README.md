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

Released versions provide the same files as archives on the corresponding
GitHub Release. For example on Linux:

```sh
gh release download v515.2.0 -R Glubus/minacalc-rs \
  -p 'minacalc-native-linux-x86_64.tar.gz'
tar -xzf minacalc-native-linux-x86_64.tar.gz
export MINACALC_LIBRARY_PATH="$PWD/libminacalc_bindings.so"
```

The language package supplies the typed wrapper; this native library supplies
the calculator itself. Keeping them separate avoids downloading every platform
binary in every npm, PyPI, NuGet, or Go installation.

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

All wrappers expose the original single/all-rate functions plus a validated
`CalcConfig`, a custom-rates function, and a detailed single-rate result that
contains the effective grind scaler. A null/omitted config retains Etterna's
defaults; a nullable rating cap disables that cap cleanly.

| Language | Install |
| --- | --- |
| TypeScript | `npm install @glubus/minacalc` |
| Python | `python -m pip install minacalc` |
| .NET | `dotnet add package Glubus.MinaCalc --version 0.2.0` |
| Go | `go get github.com/Glubus/minacalc-rs/bindings/go@v0.2.0` |

See each linked README for a complete single-rate and configurable example.
