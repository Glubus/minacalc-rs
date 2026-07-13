# MinaCalc for TypeScript

TypeScript entry points are provided for Node.js, Bun, and Deno. Every entry
loads the native library built by `cargo build --release -p minacalc-bindings`.
Set `MINACALC_LIBRARY_PATH` to its absolute path before importing a binding.

## Node.js

```sh
cd bindings/typescript
npm install
export MINACALC_LIBRARY_PATH="$PWD/../../target/release/libminacalc_bindings.so"
```

```ts
import { calcAtRate, calcAllRates } from "@minacalc/bindings/node";

const notes = [
  { notes: 0b0001, rowTime: 0.00 },
  { notes: 0b0010, rowTime: 0.20 },
  { notes: 0b0101, rowTime: 0.40 }, // left + up jump
];

console.log(calcAtRate(notes, 1.0).overall);
console.log(calcAllRates(notes)[3].overall); // 1.0x MSD
```

The Node entry uses [`koffi`](https://koffi.dev/). Node 20+ is required.

## Deno and Bun

Import `./src/deno.ts` in Deno or `@minacalc/bindings/bun` in Bun. Deno needs
FFI and environment permissions:

```sh
deno run --allow-ffi --allow-env my-chart.ts
```

`calcAtRate(notes, rate, goal?, keys?, mode?)` defaults to SSR, goal `0.93`,
and 4K. `calcAllRates(notes, keys?, mode?)` defaults to MSD and returns 14
scores from 0.7x to 2.0x. Notes use a `u32` bitmask and absolute seconds.
