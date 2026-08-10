# MinaCalc for Go

Go 1.22+ bindings using cgo and the stable `minacalc.h` ABI.

## Build and run

```sh
go get github.com/Glubus/minacalc-rs/bindings/go@v0.2.0
export CGO_LDFLAGS="-L$PWD"
export LD_LIBRARY_PATH="$PWD:$LD_LIBRARY_PATH" # Linux; native library downloaded here
go test ./...
```

Use `DYLD_LIBRARY_PATH` on macOS, or put the DLL directory in `PATH` on Windows.

## Example

```go
import (
    "fmt"
    "log"

    "github.com/Glubus/minacalc-rs/bindings/go/minacalc"
)

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
The native ABI transparently keeps one reusable MinaCalc instance per calling
OS thread, so callers do not manage handles and threads never share a calculator.

Start from `DefaultConfig()` and call `CalcAtRateDetailed` or `CalcRates` for
tuning and custom rate lists. Set `SsrRatingCap` to `nil` to disable the cap.

```go
config := minacalc.DefaultConfig()
config.SsrGoalCap = 1.0
config.SsrRatingCap = nil
config.SkillsetScalers.Stream = 1.05

detailed, err := minacalc.CalcAtRateDetailed(notes, 1.0, 0.98, 4, minacalc.SSR, config)
if err != nil { log.Fatal(err) }
customRates, err := minacalc.CalcRates(notes, []float32{0.85, 1.0, 1.25}, 4, minacalc.MSD, config)
if err != nil { log.Fatal(err) }
fmt.Println(detailed.Scores.Overall, detailed.GrindScaler, customRates)
```
