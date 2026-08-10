# MinaCalc for .NET

.NET 8+ bindings built with source-generated P/Invoke over the stable
`minacalc_bindings` native library.

## Setup

```sh
cargo build --release -p minacalc-bindings
dotnet add reference bindings/csharp/MinaCalc/MinaCalc.csproj
```

Ensure the dynamic loader can find the output: add `target/release` to
`LD_LIBRARY_PATH` on Linux, `DYLD_LIBRARY_PATH` on macOS, or `PATH` on Windows.
Alternatively, copy the platform library beside the application executable.

## Example

```csharp
using MinaCalc;

var notes = new[]
{
    new Note(0b0001, 0.00f),
    new Note(0b0010, 0.20f),
    new Note(0b0101, 0.40f), // left + up jump
};

var score = Calculator.CalcAtRate(notes, rate: 1.0f);
Console.WriteLine(score.Overall);

var allRates = Calculator.CalcAllRates(notes, keys: 4, mode: CalcMode.Msd);
Console.WriteLine(allRates[3].Overall); // 1.0x
```

`CalcAtRate` defaults to SSR with a 0.93 score goal. Use `CalcMode.Msd` for
raw difficulty. Every `Note` contains a bitmask of active zero-based columns
and an absolute timestamp in seconds.

Use `CalcConfig`, `Calculator.CalcAtRateDetailed`, and `Calculator.CalcRates`
for tuning, grind-scaler metadata, and custom rate lists. A null
`SsrRatingCap` disables the per-skillset cap.
