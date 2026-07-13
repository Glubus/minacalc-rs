# minacalc

Set `MINACALC_LIBRARY_PATH` to the library produced by
`cargo build --release -p minacalc-bindings`, then use:

```python
from minacalc import calc_at_rate

scores = calc_at_rate([(1, 0.0), (2, 0.2), (4, 0.4)], rate=1.0)
print(scores.overall)
```
