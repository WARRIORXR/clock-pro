# Clock Pro CLI — Rust Edition

Terminal port of the single-file **Clock Pro** web page (`index.html` in the
parent folder): digital clock, stopwatch, countdown timer and world clock.

## Run

```bash
cargo run
```

- `1` digital clock (updates live, `q` + Enter to return)
- `2` stopwatch — `s` start/stop, `r` reset, `q` done
- `3` timer — enter `MM:SS`, `q` cancels, beeps (`\a`) at zero
- `4` world clock — Delhi, London, New York, Tokyo, Dubai, Sydney, UTC
