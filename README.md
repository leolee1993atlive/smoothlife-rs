# smoothlife-rs

A Rust implementation of **SmoothLife** — a continuous version of Conway's Game of Life.

## What is SmoothLife?

Unlike the classic Game of Life where cells are either dead (`0`) or alive (`1`), SmoothLife operates on **continuous floating-point states** (`0.0` ~ `1.0`). Cells evolve based on the average state of neighbors within two concentric circular regions, producing organic, flowing patterns that pulse, split, and merge like living tissue.

## Features

- **Continuous cell states** — no hard binary life/death rules
- **Dual-radius neighborhood** — inner disk (`RI`) + outer annulus (`RA`)
- **Sigmoid transitions** — smooth birth/survival/death thresholds
- **Two rendering modes**:
  - **Terminal mode** (`term`) — ASCII art visualization in your terminal with 20 FPS cap
  - **GPU mode** (`bevy`) — real-time windowed rendering powered by Bevy's compute shaders with colorful HSL heat-map visualization
- **CPU parallelization** — terminal simulation uses `rayon` for parallel grid updates
- **GPU acceleration** — Bevy mode offloads the entire simulation to a WGSL compute shader
- **Interactive brush** — hold **Left Click** to plant life or **Right Click** to erase life (Bevy mode only)

## Architecture

```
src/
├── main.rs              # Entry point: selects mode (term | bevy)
├── smoothlife_core.rs   # CPU simulation kernel (rayon-parallelized)
├── smoothlife_term.rs   # Terminal ASCII renderer
└── smoothlife_bevy.rs   # Bevy GPU compute renderer

assets/shaders/
└── smooth_life.wgsl     # WGSL compute shader (init + update)
```

## Parameters

| Parameter | Value | Description |
|-----------|-------|-------------|
| `RA` | 21 | Outer radius (neighborhood) |
| `RI` | 7 | Inner radius (self + immediate neighbors) |
| `ALPHA_N` | 0.028 | Smoothing factor for neighbor density transition |
| `ALPHA_M` | 0.147 | Smoothing factor for self-density threshold interpolation |
| `B1` / `B2` | 0.257 / 0.336 | Birth interval boundaries |
| `D1` / `D2` | 0.365 / 0.549 | Death interval boundaries |
| `DT` | 0.02 | Time step for both modes |

## Running

### Terminal Mode (default)

```bash
cargo run
# or explicitly
cargo run -- term
```

- Grid size: `100 × 100`
- Renders with ASCII charset ` .-=co*&@#`
- Frame rate capped at **20 FPS**
- Uses the alternate screen buffer (restores terminal on exit)

### GPU Mode (Bevy)

```bash
cargo run -- bevy
```

- Window: `1280 × 720`
- Internal simulation resolution: `320 × 180`
- Rendering: **Compute shader** on GPU via WGSL
- Color mapping: alive cells are rendered with a **rainbow HSL heat-map** based on state value
- **Interactive brush**: hold `Left Click` to plant life, `Right Click` to erase life
- Requires a GPU with Vulkan / DirectX 12 / Metal support

## Dependencies

- [Bevy 0.18](https://bevyengine.org/) — GPU rendering & compute pipeline
- [rayon](https://github.com/rayon-rs/rayon) — data-parallel CPU grid updates
- [rand](https://github.com/rust-random/rand) — random grid initialization
- [chrono](https://github.com/chronotope/chrono) & [log](https://github.com/rust-lang/log) — logging utilities

## License

[MIT](LICENSE)
