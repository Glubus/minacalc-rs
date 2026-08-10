# ROX + MinaCalc Axum worker

This example exposes `minacalc-rs` as a small HTTP service and includes a
browser UI for trying it without writing a client. Maps are either uploaded or
downloaded from an `osu.ppy.sh` beatmap URL, then decoded by
[Rhythm Open Exchange](https://github.com/Glubus/rhythm-open-exchange).

## Run

From the repository root:

```sh
cargo run -p rox-minacalc-worker --release
```

Then open <http://127.0.0.1:3000/>. The browser UI accepts an osu! URL, exposes
the complete calculator configuration in its Filters dialog, and renders the
chart cover, an interactive skillset radar, a rate progression graph, deltas
against 1.00x, and the complete rating table. The server accepts uploads up to
16 MiB and at most 64 rates per request.

## API

`POST /api/rate` expects `multipart/form-data`:

| Field | Required | Description |
| --- | --- | --- |
| `chart` | one source required | Map file understood by ROX |
| `osu_url` | one source required | `https://osu.ppy.sh` beatmapset, beatmap, or direct `.osu` URL |
| `rates` | no | Comma-separated positive rates; defaults to `1.0` |
| `mode` | no | `msd` or `ssr`; defaults to `msd` |
| `score_goal` | no | SSR goal from `0.0` to `1.0`; defaults to `0.93` |
| `ssr_goal_cap` | no | Maximum SSR goal; defaults to `0.965` |
| `low_acc_cutoff` | no | Low-accuracy scaling threshold; defaults to `0.90` |
| `ssr_rating_cap` | no | Positive cap, or empty/`none` to disable it |
| `grind_scaling` | no | `true` or `false`; defaults to `true` |
| `scaler_*` | no | Positive Stream, Jumpstream, Handstream, Stamina, Jackspeed, Chordjack, and Technical multipliers |

```sh
curl http://127.0.0.1:3000/api/rate \
  -F 'chart=@chart.osu' \
  -F 'rates=0.85,1.0,1.25,1.5' \
  -F 'mode=ssr' \
  -F 'score_goal=0.95'
```

Do not send both `chart` and `osu_url`. For example, the URL shown on an osu!
beatmapset page can be sent directly:

```sh
curl http://127.0.0.1:3000/api/rate \
  -F 'osu_url=https://osu.ppy.sh/beatmapsets/1856758#mania/3816042' \
  -F 'rates=1.0,1.1,1.2' \
  -F 'mode=msd'
```

The worker extracts `3816042` and downloads
`https://osu.ppy.sh/osu/3816042`. Other hosts are rejected so this example
cannot be used as an arbitrary server-side proxy.

The response contains chart metadata, source note and merged row counts, and
all eight MinaCalc skillsets for every requested rate:

```json
{
  "file_name": "chart.osu",
  "title": "Example",
  "artist": "Artist",
  "difficulty": "Challenge",
  "key_count": 4,
  "source_note_count": 1200,
  "row_count": 1160,
  "mode": "ssr",
  "score_goal": 0.95,
  "results": [
    {
      "rate": 1.0,
      "scores": {
        "overall": 18.42,
        "stream": 17.95,
        "jumpstream": 18.12,
        "handstream": 16.88,
        "stamina": 17.31,
        "jackspeed": 14.52,
        "chordjack": 13.74,
        "technical": 16.91
      }
    }
  ]
}
```

`GET /api/health` returns `{"status":"ok"}`.

## Why `spawn_blocking`?

ROX parsing and MinaCalc are CPU-bound. More importantly, each `Calc` wraps a
mutable C++ calculator and is deliberately neither `Send` nor `Sync`. The
handler moves the uploaded bytes into Tokio's blocking pool, creates one
calculator inside that worker thread, computes every requested rate, and drops
it there. No calculator is shared across requests.

The conversion keeps native chart timestamps. MinaCalc applies each requested
music rate internally; changing timestamps before `calc_rates` would apply the
rate twice. Notes sharing a timestamp are merged with bitwise OR, while mines
are excluded from playable rows.

## Code layout

| File | Responsibility |
| --- | --- |
| `src/web.rs` | Axum router and static asset responses |
| `src/api.rs` | `/api/rate` handler and blocking-worker dispatch |
| `src/request.rs` | Multipart fields and validated `CalcConfig` construction |
| `src/osu.rs` | Restricted osu! URL parsing and `.osu` download |
| `src/calculator.rs` | ROX parsing and MinaCalc invocation |
| `src/conversion.rs` | `RoxChart` to MinaCalc row conversion |
| `src/models.rs` | Request and JSON response types |
| `src/error.rs` | HTTP and conversion errors |
| `static/index.html` | Semantic page structure and Filters dialog |
| `static/styles.css` | Responsive visual design |
| `static/app.js` | Request flow, results and interactions |
| `static/charts.js` | Dependency-free SVG radar and progression charts |
