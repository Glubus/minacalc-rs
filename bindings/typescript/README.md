# MinaCalc for TypeScript

TypeScript entry points are provided for Node.js, Bun, and Deno. Every entry
loads the native library built by `cargo build --release -p minacalc-bindings`.
Set `MINACALC_LIBRARY_PATH` to its absolute path before importing a binding.

## Node.js

```sh
npm install @glubus/minacalc
export MINACALC_LIBRARY_PATH="$PWD/libminacalc_bindings.so"
```

```ts
import { calcAtRate, calcAllRates } from "@glubus/minacalc/node";

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

Import `npm:@glubus/minacalc/deno` in Deno or `@glubus/minacalc/bun` in Bun. Deno needs
FFI and environment permissions:

```sh
deno run --allow-ffi --allow-env my-chart.ts
```

`calcAtRate(notes, rate, goal?, keys?, mode?)` defaults to SSR, goal `0.93`,
and 4K. `calcAllRates(notes, keys?, mode?)` defaults to MSD and returns 14
scores from 0.7x to 2.0x. Notes use a `u32` bitmask and absolute seconds.

## Configuration and custom rates

Every runtime also exports `DEFAULT_CONFIG`, `calcAtRateDetailed`, and
`calcRates`. Copy the default configuration before tweaking it; setting
`ssrRatingCap` to `null` disables that cap. Detailed results include the
effective `grindScaler`.

```ts
import { DEFAULT_CONFIG, calcAtRateDetailed, calcRates } from "@glubus/minacalc/node";

const config = {
  ...DEFAULT_CONFIG,
  ssrGoalCap: 1.0,
  ssrRatingCap: null,
  skillsetScalers: { ...DEFAULT_CONFIG.skillsetScalers, stream: 1.05 },
};

const detailed = calcAtRateDetailed(notes, 1.0, 0.98, 4, "ssr", config);
console.log(detailed.scores.overall, detailed.grindScaler);
console.log(calcRates(notes, [0.85, 1.0, 1.25], 4, "msd", config));
```
