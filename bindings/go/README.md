# MinaCalc for Go

Go 1.22+ bindings using cgo and the stable `minacalc.h` ABI.

## Build and run

```sh
cargo build --release -p minacalc-bindings
cd bindings/go
export CGO_LDFLAGS="-L$PWD/../../target/release"
export LD_LIBRARY_PATH="$PWD/../../target/release:$LD_LIBRARY_PATH" # Linux
go test ./...
```

Use `DYLD_LIBRARY_PATH` on macOS, or put the DLL directory in `PATH` on Windows.

## Example

```go
scores, err := minacalc.CalcAtRate([]minacalc.Note{
    {Notes: 0b0001, RowTime: 0.00},
    {Notes: 0b0010, RowTime: 0.20},
    {Notes: 0b0101, RowTime: 0.40}, // left + up jump
}, 1.0, 0.93, 4, minacalc.SSR)
if err != nil {
    log.Fatal(err)
}
fmt.Println(scores.Overall)
```

`MSD` calculates raw difficulty; `SSR` calculates score-relative difficulty.
`CalcAllRates` returns a fixed `[14]SkillsetScores`, ordered from 0.7x to 2.0x.
