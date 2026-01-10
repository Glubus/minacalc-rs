# MinaCalc C# Bindings

Safe .NET 8.0 wrapper for the MinaCalc Rust library.

## Prerequisites

- [Rust Toolchain](https://rustup.rs/) (to build the native library)
- [.NET 8.0 SDK](https://dotnet.microsoft.com/download)

## Setup

1.  **Build the Rust Native Library**:
    To use the C# bindings, you must build `minacalc-rs` with the `api` feature enabled.

    ```powershell
    # In the root minacalc-rs directory
    cargo build --release --features api
    ```

    This will produce `minacalc_rs.dll` in `target/release/`.

2.  **Using the Library**:
    Copy `minacalc_rs.dll` to your C# project's output directory (e.g., `bin/Release/net8.0/`).

3.  **Building the C# Project**:
    
    ```powershell
    cd bindings/csharp
    dotnet build -c Release
    ```

## Example Usage

```csharp
using MinaCalc;

// Create calculator instance (automatically managed/disposed)
using var calc = new Calculator();

var note = new MinaCalcNote { Notes = 4, RowTime = 1.0f };
var notes = new[] { note };

// Calculate SSR
try 
{
    var scores = calc.CalculateSsr(notes, 1.0f, 93.0f);
    Console.WriteLine($"Overall: {scores.Overall}");
    Console.WriteLine($"Stream: {scores.Stream}");
}
catch (Exception ex)
{
    Console.WriteLine($"Error: {ex.Message}");
}

// Calculate All Rates
var allRates = calc.CalculateAllRates(notes);
Console.WriteLine($"1.0x MSD: {allRates[3].Overall}"); // Index 3 corresponds to roughly 1.0x (internal mapping varies)
```
