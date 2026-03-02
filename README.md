<p align="center">

<img src="https://media1.tenor.com/m/0Fp0aFT9LHIAAAAC/wireframe-neon.gif" width="400" alt="uberfetch" />

<br/>

<h3>☄️ system info with esoteric default visuals</h3>

[![Rust](https://img.shields.io/badge/Built_with-Rust-dea584?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue?style=flat-square)](LICENSE)
[![Alpha](https://img.shields.io/badge/Status-Alpha-orange?style=flat-square)](#status)

</p>

---

I wanted arcane shapes to thrive in the corner of my terminal.

## Install

```sh
cargo install --path .
```

Run `uberfetch`. Stare.

## Usage

**Pick your shape:**

```sh
uberfetch                     # default → rotating diamond
uberfetch --animation toroid  # toroidal particle flow
uberfetch --animation hypercube  # 4D tesseract projection
uberfetch --animation geodesic   # breathing geodesic sphere
```

**Control the speed:**

```sh
uberfetch --fps 60            # smoother on fast terminals
uberfetch --fps 15            # gentler on battery
```

**See what's available:**

```sh
uberfetch --list              # list all animations with descriptions
```

**Leave (if you can):**

Press `q`, `Esc`, or `Ctrl+C`.

## Animations

| Animation | What you see |
|-----------|-------------|
| **diamond** | Rotating pulsating octahedron with glow trails |
| **hypercube** | 4D tesseract projected into 3D, dual-axis rotation |
| **toroid** | Particles flowing along a torus surface |
| **geodesic** | Icosphere with breathing vertex oscillation |

All rendered as braille-dot wireframes with post-processing effects (glow, dissolve, color cycling) via tachyonfx.

## Why uberfetch?

| What you get | How it works |
|-------------|-------------|
| **Braille 3D rendering** | Wireframes drawn with Unicode braille — no GPU, no sixel, just text |
| **Real system info** | OS, kernel, hostname, uptime, CPU, memory, shell, terminal |
| **Color-aware** | Respects your terminal palette — ANSI 256 and truecolor |
| **60/40 layout** | Animation on the left, system info on the right, scales to terminal size |
| **Smooth at 30 FPS** | Tick-based main loop with frame pacing via crossterm |
| **macOS + Linux** | Platform-aware system info collection |

## Status

uberfetch is in **alpha**. Architecture is defined, module stubs are in place, implementation is underway. See [the implementation plan](docs/plans/2026-03-01-uberfetch-impl.md) for what's next.

## Deeper

- [Design document](docs/plans/2026-03-01-uberfetch-design.md) — architecture, animation math, rendering pipeline
- [Implementation plan](docs/plans/2026-03-01-uberfetch-impl.md) — 13-task roadmap

## License

[MIT](LICENSE)
