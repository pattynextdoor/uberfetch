# uberfetch — Design Document

A Rust-based neofetch alternative that replaces static ASCII art with esoteric terminal animations rendered in Unicode braille characters.

## Summary

- **Runtime**: Persistent display — animates until dismissed (q / Esc / Ctrl+C)
- **Rendering**: Unicode braille characters in terminal (no GPU, works everywhere)
- **Layout**: Side-by-side, classic neofetch style (animation left, system info right)
- **Animations**: 4 curated (rotating diamond, hypercube, toroidal flow, geodesic sphere)
- **System info**: 8 core fields (OS, kernel, hostname, uptime, CPU, memory, shell, terminal)
- **Colors**: Terminal-aware (ANSI 256/truecolor, respects terminal palette)
- **Platforms**: macOS + Linux

## Architecture

```
┌─────────────────────────────────────────────┐
│                  uberfetch                   │
├──────────┬──────────────┬───────────────────┤
│  sysinfo │  animations  │    renderer       │
│  module  │   engine     │   (terminal)      │
├──────────┴──────────────┴───────────────────┤
│              main loop (tick-based)          │
└─────────────────────────────────────────────┘
```

- **sysinfo module** — Collects system information once at startup. Uses `cfg` target-based conditional compilation for macOS vs Linux.
- **animation engine** — Each animation implements an `Animation` trait with `update(dt)` and `draw()` methods. Produces braille-rendered frames at ~30 FPS.
- **renderer** — Composes animation frame + system info into side-by-side layout. Handles terminal size detection, ANSI color output, cursor management, alternate screen buffer.
- **main loop** — Runs at ~30 FPS: poll input, advance animation, render frame, sleep for remainder of frame budget.

## Project Structure

```
uberfetch/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI parsing, main loop, signal handling
│   ├── animation/
│   │   ├── mod.rs            # Animation trait + registry
│   │   ├── diamond.rs        # Rotating pulsating diamond
│   │   ├── hypercube.rs      # 4D tesseract projection
│   │   ├── toroid.rs         # Toroidal particle flow
│   │   └── geodesic.rs       # Breathing geodesic sphere
│   ├── sysinfo/
│   │   ├── mod.rs            # SystemInfo struct + unified interface
│   │   ├── linux.rs          # Linux-specific collection (/proc, /etc)
│   │   └── macos.rs          # macOS-specific collection (sysctl, sw_vers)
│   └── renderer/
│       ├── mod.rs            # Compositor: animation + info → terminal
│       ├── color.rs          # ANSI color utilities, terminal-aware palette
│       └── layout.rs         # Side-by-side layout, terminal size handling
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `ratatui` | TUI framework, Canvas with Braille markers, layout system |
| `crossterm` | Terminal backend (raw mode, alternate screen, input events) |
| `tachyonfx` | Post-processing effects (glow, dissolve, color cycling) |
| `keyframe` | Easing/interpolation for smooth animation |
| `rsille` | 3D braille rendering with built-in rotation/animation support |
| `clap` | CLI argument parsing |

Hand-rolled 3D math for rotation matrices and perspective projection.

## Animation Engine

### Trait

```rust
pub trait Animation {
    /// Advance the animation state by dt seconds
    fn update(&mut self, dt: f32);

    /// Render the current frame into a ratatui Canvas context
    fn draw(&self, ctx: &mut Context);

    /// Human-readable name for CLI selection
    fn name(&self) -> &'static str;
}
```

### Animations

| Animation | 3D Math | Visual |
|-----------|---------|--------|
| **Rotating diamond** | Octahedron vertices + rotation matrices, scale oscillates via keyframe sine easing | Braille wireframe edges with pulsing brightness |
| **Hypercube** | 16 tesseract vertices in 4D, two sequential rotations (XW + YZ planes), project 4D→3D→2D | Braille edges, inner/outer cube distinguished by color intensity |
| **Toroidal flow** | Parametric torus surface, particle positions advance along surface normals over time | Braille dots for particles, density creates the donut shape |
| **Geodesic sphere** | Icosahedron subdivided once (~42 vertices), vertex positions interpolate inward/outward via keyframe elastic easing | Braille wireframe with breathing radius |

### Per-Frame Pipeline

1. `update(dt)` advances rotation angles, particle positions, breath phase
2. 3D points projected to 2D via perspective projection
3. 2D points mapped to Canvas coordinate space (scaled to animation panel size)
4. Lines/dots drawn onto ratatui Canvas with Braille markers
5. tachyonfx applies post-processing (subtle glow, color shifts)

## System Info Collection

### Struct

```rust
pub struct SystemInfo {
    pub os: String,
    pub kernel: String,
    pub hostname: String,
    pub uptime: String,
    pub cpu: String,
    pub memory: String,
    pub shell: String,
    pub terminal: String,
}
```

### Collection Strategy

| Field | macOS | Linux |
|-------|-------|-------|
| OS | `sw_vers` | `/etc/os-release` |
| Kernel | `uname -r` | `uname -r` |
| Hostname | `hostname` syscall | `/etc/hostname` |
| Uptime | `sysctl kern.boottime` | `/proc/uptime` |
| CPU | `sysctl machdep.cpu.brand_string` + `hw.ncpu` | `/proc/cpuinfo` |
| Memory | `sysctl hw.memsize` + `vm_stat` | `/proc/meminfo` |
| Shell | `$SHELL` + `--version` | `$SHELL` + `--version` |
| Terminal | `$TERM_PROGRAM` | `$TERM_PROGRAM` or parent process |

Failures for any field fall back to `"Unknown"` — system info is never worth a panic.

## Layout

```
┌─────────────────────────────────┬──────────────────────────┐
│                                 │  patty@patty-mbp         │
│                                 │  ──────────────────      │
│        [animation area]         │  OS: macOS 15.3          │
│                                 │  Kernel: Darwin 24.6.0   │
│     rotating diamond / hyper-   │  Uptime: 3 days, 7 hrs  │
│     cube / toroid / geodesic    │  CPU: Apple M2 Pro (12)  │
│                                 │  Memory: 8.2 / 16.0 GiB │
│                                 │  Shell: zsh 5.9          │
│                                 │  Terminal: iTerm2        │
│                                 │                          │
│                                 │  ████████████████        │
│                                 │  (color palette strip)   │
└─────────────────────────────────┴──────────────────────────┘
```

- 60/40 width split (animation gets the larger portion)
- Animation area scales dynamically to available space
- System info vertically centered in its panel
- Color palette strip at bottom of info panel (8/16 ANSI color blocks)
- Minimum terminal size ~80x24; smaller shows "terminal too small"

## Rendering Loop

1. Enter alternate screen + raw mode via crossterm
2. Collect system info (once)
3. Loop at ~30 FPS:
   - Poll crossterm events (quit on q / Esc / Ctrl+C)
   - Call `animation.update(dt)`
   - Draw ratatui frame: horizontal layout splits into two rects
   - Left: Canvas widget with Braille marker, animation draws into it
   - Right: Paragraph widget with styled system info lines
   - Apply tachyonfx effects to animation area
4. On exit: leave alternate screen, restore terminal

## CLI

```
uberfetch [OPTIONS]

Options:
  -a, --animation <NAME>    Animation to display [default: diamond]
                             [possible: diamond, hypercube, toroid, geodesic]
  -f, --fps <N>             Target frames per second [default: 30]
  -l, --list                List available animations and exit
  -v, --version             Print version and exit
  -h, --help                Print help
```

## Reference Projects

- **tarts** — Terminal screensaver with 3D braille cube (pipeline reference)
- **ternimal** — 2,500 FPS terminal animation (performance reference)
- **terminal3d** — .obj viewer with braille rendering (projection reference)
