# MinaCalc language bindings

All bindings call the `minacalc-bindings` Rust crate through its stable C ABI;
they do not link to MinaCalc's C++ sources or call repository code directly.

## Build the native library

```sh
cargo build --release -p minacalc-bindings
```

The resulting library is named `libminacalc_bindings.so` on Linux,
`libminacalc_bindings.dylib` on macOS, and `minacalc_bindings.dll` on Windows.
Set `MINACALC_LIBRARY_PATH` to its absolute path before using a binding, or
install it through the platform's normal dynamic-library mechanism.

`include/minacalc.h` is the versioned ABI contract. It exposes two operations:
single-rate calculation and calculation for the fourteen rates from 0.7x to
2.0x. Inputs are deliberately plain note rows: `{ notes: bitmask, row_time: seconds }`.

## Layout

- `typescript/`: Node.js, Bun, and Deno TypeScript entry points.
- `python/`: a typed Python package using `ctypes` from the standard library.
- `csharp/`: a .NET library based on source-generated P/Invoke.
- `go/`: a Go package using cgo and the shared ABI header.

Every wrapper validates its own obvious inputs and turns non-zero native status
codes into a language-native error.
