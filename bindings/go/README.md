# MinaCalc Go bindings

Build `minacalc-bindings` first, then make its library discoverable to the Go
linker (`CGO_LDFLAGS=-L/path/to/target/release`) and dynamic loader
(`LD_LIBRARY_PATH`, `DYLD_LIBRARY_PATH`, or `PATH` on Windows).

```go
scores, err := minacalc.CalcAtRate([]minacalc.Note{
  {Notes: 1, RowTime: 0}, {Notes: 2, RowTime: .2},
}, 1, .93, 4, minacalc.SSR)
```
