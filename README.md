# smoothlife-rs

A Rust implementation of **SmoothLife** — a continuous version of Conway's Game of Life.

## What is SmoothLife?

Unlike the classic Game of Life where cells are either dead (`0`) or alive (`1`), SmoothLife operates on **continuous floating-point states** (`0.0` ~ `1.0`). Cells evolve based on the average state of neighbors within two concentric circular regions, producing organic, flowing patterns that pulse, split, and merge like living tissue.

## Features

- **Continuous cell states** — no hard binary life/death rules
- **Dual-radius neighborhood** — inner disk (`RI`) + outer annulus (`RA`)
- **Sigmoid transitions** — smooth birth/survival/death thresholds
- **ASCII visualization** — watch the simulation evolve directly in your terminal

## Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| `WIDTH` / `HEIGHT` | 100×100 | Grid size |
| `RA` | 21 | Outer radius (neighborhood) |
| `RI` | 7 | Inner radius (self + immediate neighbors) |
| `ALPHA_N` | 0.028 | Smoothing factor for neighbor density transition |
| `ALPHA_M` | 0.147 | Smoothing factor for self-density threshold interpolation |
| `B1` / `B2` | 0.278 / 0.365 | Birth interval boundaries |
| `D1` / `D2` | 0.267 / 0.445 | Death interval boundaries |
| `DT` | 0.01 | Time step |

## Running

```bash
cargo run
```

The simulation starts from a random grid and evolves indefinitely, printing each frame as ASCII art using the charset ` .-=co*&@#`.

## References

- Stephan Rafler, "Generalization of Conway's 'Game of Life' to a continuous domain" — the original SmoothLife paper
- Tutorial series that inspired this implementation

## License

[MIT](LICENSE)
