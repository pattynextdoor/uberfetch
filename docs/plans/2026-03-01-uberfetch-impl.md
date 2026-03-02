# uberfetch Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build a Rust neofetch alternative that displays esoteric braille animations alongside system info in a persistent terminal display.

**Architecture:** Three modules (sysinfo, animation, renderer) composed by a tick-based main loop. Animations use rsille's Object3D for 3D geometry and ratatui's Canvas with Braille markers for rendering. tachyonfx provides post-processing effects.

**Tech Stack:** Rust, ratatui, crossterm, tachyonfx, keyframe, rsille, clap

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/animation/mod.rs`
- Create: `src/sysinfo/mod.rs`
- Create: `src/renderer/mod.rs`

**Step 1: Initialize Cargo project**

Run: `cargo init --name uberfetch`

**Step 2: Set up Cargo.toml with dependencies**

Replace `Cargo.toml` with:

```toml
[package]
name = "uberfetch"
version = "0.1.0"
edition = "2021"
description = "A neofetch alternative with esoteric terminal animations"

[dependencies]
ratatui = { version = "0.29", features = ["all-widgets"] }
crossterm = "0.28"
tachyonfx = "0.9"
keyframe = "1"
rsille = "2"
clap = { version = "4", features = ["derive"] }
```

Note: pin exact minor versions after verifying latest on crates.io. The versions above are starting points — adjust if `cargo check` fails.

**Step 3: Create module directory structure**

```
mkdir -p src/animation src/sysinfo src/renderer
```

**Step 4: Create placeholder modules**

`src/animation/mod.rs`:
```rust
pub mod diamond;

use ratatui::widgets::canvas::Context;

pub trait Animation {
    fn update(&mut self, dt: f32);
    fn draw(&self, ctx: &mut Context);
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

pub fn list_animations() -> Vec<(&'static str, &'static str)> {
    vec![
        ("diamond", "Rotating pulsating octahedron"),
        ("hypercube", "4D tesseract rotation"),
        ("toroid", "Toroidal particle flow"),
        ("geodesic", "Breathing geodesic sphere"),
    ]
}
```

`src/sysinfo/mod.rs`:
```rust
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub uptime: String,
    pub cpu: String,
    pub memory: String,
    pub shell: String,
    pub terminal: String,
}

impl SystemInfo {
    pub fn collect() -> Self {
        #[cfg(target_os = "macos")]
        return macos::collect();
        #[cfg(target_os = "linux")]
        return linux::collect();
    }
}
```

`src/renderer/mod.rs`:
```rust
pub mod color;
pub mod layout;
```

`src/renderer/color.rs`:
```rust
// Color utilities — placeholder
```

`src/renderer/layout.rs`:
```rust
// Layout composition — placeholder
```

`src/main.rs`:
```rust
mod animation;
mod renderer;
mod sysinfo;

fn main() {
    println!("uberfetch - coming soon");
}
```

**Step 5: Verify it compiles**

Run: `cargo check`
Expected: success (warnings about unused modules are fine)

**Step 6: Commit**

```bash
git add Cargo.toml src/
git commit -m "✨ feat: scaffold project structure with module stubs"
```

---

### Task 2: CLI Parsing

**Files:**
- Modify: `src/main.rs`

**Step 1: Write CLI struct with clap derive**

Replace `src/main.rs` with:

```rust
mod animation;
mod renderer;
mod sysinfo;

use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "uberfetch", version, about = "Neofetch with esoteric animations")]
struct Cli {
    /// Animation to display
    #[arg(short, long, value_enum, default_value_t = AnimationChoice::Diamond)]
    animation: AnimationChoice,

    /// Target frames per second
    #[arg(short, long, default_value_t = 30)]
    fps: u32,

    /// List available animations and exit
    #[arg(short, long)]
    list: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum AnimationChoice {
    Diamond,
    Hypercube,
    Toroid,
    Geodesic,
}

fn main() {
    let cli = Cli::parse();

    if cli.list {
        println!("Available animations:");
        for (name, desc) in animation::list_animations() {
            println!("  {:<12} {}", name, desc);
        }
        return;
    }

    println!("Animation: {:?}", cli.animation as u8);
    println!("FPS: {}", cli.fps);
}
```

**Step 2: Verify CLI works**

Run: `cargo run -- --help`
Expected: help text with --animation, --fps, --list options

Run: `cargo run -- --list`
Expected: list of 4 animations with descriptions

Run: `cargo run -- -a hypercube --fps 60`
Expected: prints animation choice and fps

**Step 3: Commit**

```bash
git add src/main.rs
git commit -m "✨ feat(cli): add argument parsing with clap"
```

---

### Task 3: System Info — macOS

**Files:**
- Create: `src/sysinfo/macos.rs`
- Modify: `src/sysinfo/mod.rs`

**Step 1: Write tests for parsing helpers**

Add to `src/sysinfo/mod.rs` (bottom):

```rust
pub fn format_uptime(total_secs: u64) -> String {
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let mins = (total_secs % 3600) / 60;
    match (days, hours, mins) {
        (0, 0, m) => format!("{m} mins"),
        (0, h, m) => format!("{h} hours, {m} mins"),
        (d, h, _) => format!("{d} days, {h} hours"),
    }
}

pub fn format_memory(used_bytes: u64, total_bytes: u64) -> String {
    let used_gib = used_bytes as f64 / 1_073_741_824.0;
    let total_gib = total_bytes as f64 / 1_073_741_824.0;
    format!("{used_gib:.1} GiB / {total_gib:.1} GiB")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_uptime_minutes_only() {
        assert_eq!(format_uptime(300), "5 mins");
    }

    #[test]
    fn test_format_uptime_hours_and_minutes() {
        assert_eq!(format_uptime(7500), "2 hours, 5 mins");
    }

    #[test]
    fn test_format_uptime_days_and_hours() {
        assert_eq!(format_uptime(100_000), "1 days, 3 hours");
    }

    #[test]
    fn test_format_memory() {
        let used = 8_589_934_592; // 8 GiB
        let total = 17_179_869_184; // 16 GiB
        assert_eq!(format_memory(used, total), "8.0 GiB / 16.0 GiB");
    }
}
```

**Step 2: Run tests to verify they pass**

Run: `cargo test`
Expected: 4 tests pass

**Step 3: Implement macOS collection**

Write `src/sysinfo/macos.rs`:

```rust
use crate::sysinfo::SystemInfo;
use std::process::Command;

pub fn collect() -> SystemInfo {
    SystemInfo {
        hostname: get_hostname(),
        os: get_os(),
        kernel: get_kernel(),
        uptime: get_uptime(),
        cpu: get_cpu(),
        memory: get_memory(),
        shell: get_shell(),
        terminal: get_terminal(),
    }
}

fn cmd_output(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn sysctl(key: &str) -> Option<String> {
    cmd_output("sysctl", &["-n", key])
}

fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "Unknown".into())
}

fn get_os() -> String {
    let name = cmd_output("sw_vers", &["-productName"]).unwrap_or_default();
    let version = cmd_output("sw_vers", &["-productVersion"]).unwrap_or_default();
    if name.is_empty() {
        "Unknown".into()
    } else {
        format!("{name} {version}")
    }
}

fn get_kernel() -> String {
    cmd_output("uname", &["-r"]).unwrap_or_else(|| "Unknown".into())
}

fn get_uptime() -> String {
    // kern.boottime returns: { sec = 1234567890, usec = 0 }
    sysctl("kern.boottime")
        .and_then(|raw| {
            let sec_str = raw.split("sec = ").nth(1)?;
            let sec: u64 = sec_str.split(',').next()?.trim().parse().ok()?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            Some(super::format_uptime(now - sec))
        })
        .unwrap_or_else(|| "Unknown".into())
}

fn get_cpu() -> String {
    let brand = sysctl("machdep.cpu.brand_string").unwrap_or_else(|| "Unknown".into());
    let cores = sysctl("hw.ncpu").unwrap_or_else(|| "?".into());
    format!("{brand} ({cores} cores)")
}

fn get_memory() -> String {
    let total_bytes: u64 = sysctl("hw.memsize")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    // vm_stat gives pages; page size is typically 16384 on Apple Silicon, 4096 on Intel
    let page_size: u64 = sysctl("hw.pagesize")
        .and_then(|s| s.parse().ok())
        .unwrap_or(16384);

    let used_bytes = cmd_output("vm_stat", &[])
        .map(|raw| {
            let parse_line = |key: &str| -> u64 {
                raw.lines()
                    .find(|l| l.contains(key))
                    .and_then(|l| {
                        l.split(':')
                            .nth(1)?
                            .trim()
                            .trim_end_matches('.')
                            .parse()
                            .ok()
                    })
                    .unwrap_or(0)
            };
            let active = parse_line("Pages active");
            let wired = parse_line("Pages wired down");
            let compressed = parse_line("Pages occupied by compressor");
            (active + wired + compressed) * page_size
        })
        .unwrap_or(0);

    if total_bytes == 0 {
        "Unknown".into()
    } else {
        super::format_memory(used_bytes, total_bytes)
    }
}

fn get_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .map(|s| {
            let name = s.rsplit('/').next().unwrap_or(&s).to_string();
            // Try to get version
            let version = cmd_output(&s, &["--version"])
                .and_then(|v| {
                    // Extract first version-like string (e.g., "zsh 5.9")
                    v.lines().next().map(|l| l.to_string())
                });
            match version {
                Some(v) if v.contains(&name) => v,
                _ => name,
            }
        })
        .unwrap_or_else(|| "Unknown".into())
}

fn get_terminal() -> String {
    std::env::var("TERM_PROGRAM").unwrap_or_else(|_| "Unknown".into())
}
```

Note: this uses the `hostname` crate. Add it to `Cargo.toml`:

```toml
hostname = "0.4"
```

**Step 4: Verify it compiles and info looks correct**

Add a temporary test in main:

```rust
// In main(), temporarily:
let info = sysinfo::SystemInfo::collect();
println!("{}@{}", whoami(), info.hostname);
println!("OS: {}", info.os);
println!("Kernel: {}", info.kernel);
println!("Uptime: {}", info.uptime);
println!("CPU: {}", info.cpu);
println!("Memory: {}", info.memory);
println!("Shell: {}", info.shell);
println!("Terminal: {}", info.terminal);
```

Run: `cargo run`
Expected: your actual system info printed correctly

**Step 5: Commit**

```bash
git add -A
git commit -m "✨ feat(sysinfo): add macOS system info collection"
```

---

### Task 4: System Info — Linux

**Files:**
- Create: `src/sysinfo/linux.rs`

**Step 1: Implement Linux collection**

Write `src/sysinfo/linux.rs`:

```rust
use crate::sysinfo::SystemInfo;
use std::fs;
use std::process::Command;

pub fn collect() -> SystemInfo {
    SystemInfo {
        hostname: get_hostname(),
        os: get_os(),
        kernel: get_kernel(),
        uptime: get_uptime(),
        cpu: get_cpu(),
        memory: get_memory(),
        shell: get_shell(),
        terminal: get_terminal(),
    }
}

fn read_file(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
}

fn cmd_output(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

fn get_hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "Unknown".into())
}

fn get_os() -> String {
    read_file("/etc/os-release")
        .and_then(|content| {
            content
                .lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .map(|l| l.trim_start_matches("PRETTY_NAME=").trim_matches('"').to_string())
        })
        .unwrap_or_else(|| "Linux".into())
}

fn get_kernel() -> String {
    cmd_output("uname", &["-r"]).unwrap_or_else(|| "Unknown".into())
}

fn get_uptime() -> String {
    read_file("/proc/uptime")
        .and_then(|content| {
            let secs: f64 = content.split_whitespace().next()?.parse().ok()?;
            Some(super::format_uptime(secs as u64))
        })
        .unwrap_or_else(|| "Unknown".into())
}

fn get_cpu() -> String {
    read_file("/proc/cpuinfo")
        .map(|content| {
            let model = content
                .lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map(|s| s.trim().to_string())
                .unwrap_or_else(|| "Unknown".into());
            let cores = content
                .lines()
                .filter(|l| l.starts_with("processor"))
                .count();
            format!("{model} ({cores} cores)")
        })
        .unwrap_or_else(|| "Unknown".into())
}

fn get_memory() -> String {
    read_file("/proc/meminfo")
        .map(|content| {
            let parse_kb = |key: &str| -> u64 {
                content
                    .lines()
                    .find(|l| l.starts_with(key))
                    .and_then(|l| {
                        l.split_whitespace().nth(1)?.parse().ok()
                    })
                    .unwrap_or(0)
            };
            let total_kb = parse_kb("MemTotal:");
            let available_kb = parse_kb("MemAvailable:");
            let used_kb = total_kb - available_kb;
            super::format_memory(used_kb * 1024, total_kb * 1024)
        })
        .unwrap_or_else(|| "Unknown".into())
}

fn get_shell() -> String {
    std::env::var("SHELL")
        .ok()
        .map(|s| {
            let name = s.rsplit('/').next().unwrap_or(&s).to_string();
            let version = Command::new(&s)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|v| v.lines().next().map(|l| l.to_string()));
            match version {
                Some(v) if v.contains(&name) => v,
                _ => name,
            }
        })
        .unwrap_or_else(|| "Unknown".into())
}

fn get_terminal() -> String {
    std::env::var("TERM_PROGRAM").unwrap_or_else(|_| "Unknown".into())
}
```

**Step 2: Verify it compiles**

Run: `cargo check`
Expected: success (Linux code won't run on macOS but should compile with cfg gating)

**Step 3: Commit**

```bash
git add src/sysinfo/linux.rs
git commit -m "✨ feat(sysinfo): add Linux system info collection"
```

---

### Task 5: 3D Math Utilities

**Files:**
- Create: `src/animation/math.rs`
- Modify: `src/animation/mod.rs`

This task builds the shared 3D projection and rotation math that all animations use.

**Step 1: Write tests for 3D math**

Write `src/animation/math.rs`:

```rust
/// A 3D point.
pub type Vec3 = [f64; 3];

/// A 2D point (after projection).
pub type Vec2 = [f64; 2];

/// Rotate a point around the X axis by `angle` radians.
pub fn rotate_x(p: Vec3, angle: f64) -> Vec3 {
    let (s, c) = angle.sin_cos();
    [p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c]
}

/// Rotate a point around the Y axis by `angle` radians.
pub fn rotate_y(p: Vec3, angle: f64) -> Vec3 {
    let (s, c) = angle.sin_cos();
    [p[0] * c + p[2] * s, p[1], -p[0] * s + p[2] * c]
}

/// Rotate a point around the Z axis by `angle` radians.
pub fn rotate_z(p: Vec3, angle: f64) -> Vec3 {
    let (s, c) = angle.sin_cos();
    [p[0] * c - p[1] * s, p[0] * s + p[1] * c, p[2]]
}

/// Perspective projection from 3D to 2D.
/// `distance` is the camera distance from the origin along the Z axis.
/// Returns (x, y) screen coordinates.
pub fn project(p: Vec3, distance: f64) -> Vec2 {
    let z = p[2] + distance;
    if z.abs() < 0.001 {
        return [0.0, 0.0];
    }
    let factor = distance / z;
    [p[0] * factor, p[1] * factor]
}

/// Scale a Vec3 by a scalar.
pub fn scale(p: Vec3, s: f64) -> Vec3 {
    [p[0] * s, p[1] * s, p[2] * s]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    fn approx_eq(a: &[f64], b: &[f64], eps: f64) -> bool {
        a.iter().zip(b).all(|(x, y)| (x - y).abs() < eps)
    }

    #[test]
    fn test_rotate_x_90() {
        let p = [0.0, 1.0, 0.0];
        let r = rotate_x(p, PI / 2.0);
        assert!(approx_eq(&r, &[0.0, 0.0, 1.0], 1e-10));
    }

    #[test]
    fn test_rotate_y_90() {
        let p = [1.0, 0.0, 0.0];
        let r = rotate_y(p, PI / 2.0);
        assert!(approx_eq(&r, &[0.0, 0.0, -1.0], 1e-10));
    }

    #[test]
    fn test_rotate_z_90() {
        let p = [1.0, 0.0, 0.0];
        let r = rotate_z(p, PI / 2.0);
        assert!(approx_eq(&r, &[0.0, 1.0, 0.0], 1e-10));
    }

    #[test]
    fn test_project_on_axis() {
        // Point at origin should project to origin
        let p = [0.0, 0.0, 0.0];
        let r = project(p, 5.0);
        assert!(approx_eq(&r, &[0.0, 0.0], 1e-10));
    }

    #[test]
    fn test_project_perspective() {
        // Point at (1, 0, 0) with distance 5 -> factor = 5/5 = 1 -> (1, 0)
        let p = [1.0, 0.0, 0.0];
        let r = project(p, 5.0);
        assert!(approx_eq(&r, &[1.0, 0.0], 1e-10));
    }

    #[test]
    fn test_scale() {
        let p = [1.0, 2.0, 3.0];
        let r = scale(p, 2.0);
        assert!(approx_eq(&r, &[2.0, 4.0, 6.0], 1e-10));
    }
}
```

**Step 2: Add math module to animation/mod.rs**

Add at the top of `src/animation/mod.rs`:

```rust
pub mod math;
```

**Step 3: Run tests**

Run: `cargo test animation::math`
Expected: 6 tests pass

**Step 4: Commit**

```bash
git add src/animation/math.rs src/animation/mod.rs
git commit -m "✨ feat(animation): add 3D math utilities with tests"
```

---

### Task 6: Diamond Animation

**Files:**
- Create: `src/animation/diamond.rs`
- Modify: `src/animation/mod.rs`

The diamond is an octahedron (6 vertices, 12 edges) that rotates and pulses.

**Step 1: Write diamond animation**

Write `src/animation/diamond.rs`:

```rust
use crate::animation::math::{self, Vec3};
use crate::animation::Animation;
use ratatui::style::Color;
use ratatui::widgets::canvas::Context;

/// Octahedron vertices: top, bottom, and 4 equatorial points.
const VERTICES: [Vec3; 6] = [
    [0.0, 1.0, 0.0],   // top
    [0.0, -1.0, 0.0],  // bottom
    [1.0, 0.0, 0.0],   // right
    [-1.0, 0.0, 0.0],  // left
    [0.0, 0.0, 1.0],   // front
    [0.0, 0.0, -1.0],  // back
];

/// Edges connecting vertices by index.
const EDGES: [(usize, usize); 12] = [
    // Top to equator
    (0, 2), (0, 3), (0, 4), (0, 5),
    // Bottom to equator
    (1, 2), (1, 3), (1, 4), (1, 5),
    // Equatorial ring
    (2, 4), (4, 3), (3, 5), (5, 2),
];

pub struct Diamond {
    angle_x: f64,
    angle_y: f64,
    angle_z: f64,
    time: f64,
}

impl Diamond {
    pub fn new() -> Self {
        Self {
            angle_x: 0.0,
            angle_y: 0.0,
            angle_z: 0.0,
            time: 0.0,
        }
    }
}

impl Animation for Diamond {
    fn update(&mut self, dt: f32) {
        let dt = dt as f64;
        self.time += dt;
        self.angle_x += 0.8 * dt;
        self.angle_y += 1.2 * dt;
        self.angle_z += 0.3 * dt;
    }

    fn draw(&self, ctx: &mut Context) {
        let pulse = 1.0 + 0.2 * (self.time * 2.5).sin();
        let distance = 4.0;
        let base_scale = 30.0 * pulse;

        let projected: Vec<[f64; 2]> = VERTICES
            .iter()
            .map(|&v| {
                let v = math::scale(v, base_scale);
                let v = math::rotate_x(v, self.angle_x);
                let v = math::rotate_y(v, self.angle_y);
                let v = math::rotate_z(v, self.angle_z);
                math::project(v, distance * base_scale)
            })
            .collect();

        // Compute brightness per edge based on average z-depth for color variation
        for &(i, j) in &EDGES {
            let brightness = ((self.time * 1.5 + i as f64 * 0.5).sin() * 0.3 + 0.7)
                .clamp(0.3, 1.0);
            let gray = (brightness * 255.0) as u8;
            let color = Color::Rgb(gray, gray, (gray as f64 * 0.8) as u8);

            ctx.draw(&ratatui::widgets::canvas::Line {
                x1: projected[i][0],
                y1: projected[i][1],
                x2: projected[j][0],
                y2: projected[j][1],
                color,
            });
        }
    }

    fn name(&self) -> &'static str {
        "diamond"
    }

    fn description(&self) -> &'static str {
        "Rotating pulsating octahedron"
    }
}
```

**Step 2: Update Animation trait to include description**

Update `src/animation/mod.rs`:

```rust
pub mod diamond;
pub mod math;

use ratatui::widgets::canvas::Context;

pub trait Animation {
    fn update(&mut self, dt: f32);
    fn draw(&self, ctx: &mut Context);
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
}

pub fn list_animations() -> Vec<(&'static str, &'static str)> {
    vec![
        ("diamond", "Rotating pulsating octahedron"),
        ("hypercube", "4D tesseract rotation"),
        ("toroid", "Toroidal particle flow"),
        ("geodesic", "Breathing geodesic sphere"),
    ]
}
```

**Step 3: Verify it compiles**

Run: `cargo check`
Expected: success

**Step 4: Commit**

```bash
git add src/animation/
git commit -m "✨ feat(animation): add rotating pulsating diamond"
```

---

### Task 7: Renderer and Layout

**Files:**
- Rewrite: `src/renderer/mod.rs`
- Rewrite: `src/renderer/color.rs`
- Rewrite: `src/renderer/layout.rs`

**Step 1: Implement color palette**

Write `src/renderer/color.rs`:

```rust
use ratatui::style::Color;

/// The 8 standard ANSI colors for the palette strip.
pub const PALETTE: [Color; 16] = [
    Color::Indexed(0),  // black
    Color::Indexed(1),  // red
    Color::Indexed(2),  // green
    Color::Indexed(3),  // yellow
    Color::Indexed(4),  // blue
    Color::Indexed(5),  // magenta
    Color::Indexed(6),  // cyan
    Color::Indexed(7),  // white
    Color::Indexed(8),  // bright black
    Color::Indexed(9),  // bright red
    Color::Indexed(10), // bright green
    Color::Indexed(11), // bright yellow
    Color::Indexed(12), // bright blue
    Color::Indexed(13), // bright magenta
    Color::Indexed(14), // bright cyan
    Color::Indexed(15), // bright white
];

/// The accent color used for labels (adapts to terminal palette).
pub const LABEL_COLOR: Color = Color::Indexed(4); // blue
/// The separator color.
pub const SEPARATOR_COLOR: Color = Color::Indexed(8); // bright black / gray
```

**Step 2: Implement layout compositor**

Write `src/renderer/layout.rs`:

```rust
use crate::animation::Animation;
use crate::renderer::color;
use crate::sysinfo::SystemInfo;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::symbols::Marker;
use ratatui::text::{Line, Span};
use ratatui::widgets::canvas::Canvas;
use ratatui::widgets::{Block, Paragraph, Widget};
use ratatui::Frame;

const MIN_WIDTH: u16 = 80;
const MIN_HEIGHT: u16 = 24;

pub fn draw(frame: &mut Frame, animation: &dyn Animation, info: &SystemInfo) {
    let area = frame.area();

    if area.width < MIN_WIDTH || area.height < MIN_HEIGHT {
        let msg = Paragraph::new("Terminal too small (need 80x24)")
            .style(Style::default().fg(ratatui::style::Color::Red));
        frame.render_widget(msg, area);
        return;
    }

    let chunks = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ])
    .split(area);

    draw_animation(frame, animation, chunks[0]);
    draw_info(frame, info, chunks[1]);
}

fn draw_animation(frame: &mut Frame, animation: &dyn Animation, area: Rect) {
    let canvas = Canvas::default()
        .marker(Marker::Braille)
        .x_bounds([-(area.width as f64), area.width as f64])
        .y_bounds([-(area.height as f64), area.height as f64])
        .paint(|ctx| {
            animation.draw(ctx);
        });
    frame.render_widget(canvas, area);
}

fn draw_info(frame: &mut Frame, info: &SystemInfo, area: Rect) {
    let username = std::env::var("USER")
        .or_else(|_| std::env::var("LOGNAME"))
        .unwrap_or_else(|_| "user".into());
    let title = format!("{username}@{}", info.hostname);
    let separator = "─".repeat(title.len());

    let fields = [
        ("OS", &info.os),
        ("Kernel", &info.kernel),
        ("Uptime", &info.uptime),
        ("CPU", &info.cpu),
        ("Memory", &info.memory),
        ("Shell", &info.shell),
        ("Terminal", &info.terminal),
    ];

    let mut lines: Vec<Line> = Vec::new();
    lines.push(Line::from(Span::styled(
        title,
        Style::default().fg(color::LABEL_COLOR).bold(),
    )));
    lines.push(Line::from(Span::styled(
        separator,
        Style::default().fg(color::SEPARATOR_COLOR),
    )));

    for (label, value) in &fields {
        lines.push(Line::from(vec![
            Span::styled(
                format!("{label}: "),
                Style::default().fg(color::LABEL_COLOR).bold(),
            ),
            Span::raw(*value),
        ]));
    }

    // Empty line before palette
    lines.push(Line::from(""));

    // Color palette strip — two rows of 8 colored blocks
    let palette_top: Vec<Span> = color::PALETTE[..8]
        .iter()
        .map(|&c| Span::styled("███", Style::default().fg(c)))
        .collect();
    let palette_bottom: Vec<Span> = color::PALETTE[8..]
        .iter()
        .map(|&c| Span::styled("███", Style::default().fg(c)))
        .collect();
    lines.push(Line::from(palette_top));
    lines.push(Line::from(palette_bottom));

    // Vertically center the info block
    let content_height = lines.len() as u16;
    let top_padding = area.height.saturating_sub(content_height) / 2;

    // Prepend empty lines for vertical centering
    let mut centered_lines: Vec<Line> = vec![Line::from(""); top_padding as usize];
    centered_lines.extend(lines);

    let paragraph = Paragraph::new(centered_lines).block(Block::default());
    frame.render_widget(paragraph, area);
}
```

**Step 3: Update renderer mod.rs**

Write `src/renderer/mod.rs`:

```rust
pub mod color;
pub mod layout;
```

**Step 4: Verify it compiles**

Run: `cargo check`
Expected: success

**Step 5: Commit**

```bash
git add src/renderer/
git commit -m "✨ feat(renderer): add side-by-side layout with info panel and color palette"
```

---

### Task 8: Main Loop

**Files:**
- Rewrite: `src/main.rs`

This ties everything together: CLI parsing, sysinfo collection, animation selection, and the render loop.

**Step 1: Write the main loop**

Replace `src/main.rs`:

```rust
mod animation;
mod renderer;
mod sysinfo;

use std::io;
use std::time::{Duration, Instant};

use animation::diamond::Diamond;
use animation::Animation;
use clap::{Parser, ValueEnum};
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

#[derive(Parser)]
#[command(name = "uberfetch", version, about = "Neofetch with esoteric animations")]
struct Cli {
    /// Animation to display
    #[arg(short, long, value_enum, default_value_t = AnimationChoice::Diamond)]
    animation: AnimationChoice,

    /// Target frames per second
    #[arg(short, long, default_value_t = 30)]
    fps: u32,

    /// List available animations and exit
    #[arg(short, long)]
    list: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, ValueEnum)]
enum AnimationChoice {
    Diamond,
    Hypercube,
    Toroid,
    Geodesic,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    if cli.list {
        println!("Available animations:");
        for (name, desc) in animation::list_animations() {
            println!("  {:<12} {}", name, desc);
        }
        return Ok(());
    }

    let mut anim: Box<dyn Animation> = match cli.animation {
        AnimationChoice::Diamond => Box::new(Diamond::new()),
        // TODO: add other animations as they're implemented
        _ => {
            eprintln!("Animation not yet implemented, using diamond");
            Box::new(Diamond::new())
        }
    };

    let info = sysinfo::SystemInfo::collect();

    let terminal = ratatui::init();
    let result = run(terminal, anim.as_mut(), &info, cli.fps);
    ratatui::restore();
    result
}

fn run(
    mut terminal: DefaultTerminal,
    animation: &mut dyn Animation,
    info: &sysinfo::SystemInfo,
    fps: u32,
) -> io::Result<()> {
    let tick_rate = Duration::from_secs_f64(1.0 / fps as f64);
    let mut last_tick = Instant::now();

    loop {
        terminal.draw(|frame| {
            renderer::layout::draw(frame, animation, info);
        })?;

        let timeout = tick_rate.saturating_sub(last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            let dt = last_tick.elapsed().as_secs_f32();
            animation.update(dt);
            last_tick = Instant::now();
        }
    }
}
```

**Step 2: Run the application**

Run: `cargo run`
Expected: fullscreen alternate-screen display with rotating diamond on the left, system info on the right. Press `q` to quit.

**Step 3: Verify quit works**

Press `q` — should exit cleanly back to normal terminal.
Press `Esc` — should also exit.

**Step 4: Test with flags**

Run: `cargo run -- --fps 60`
Expected: smoother animation

**Step 5: Commit**

```bash
git add src/main.rs
git commit -m "✨ feat: wire up main loop with animation, sysinfo, and renderer"
```

---

### Task 9: Hypercube Animation

**Files:**
- Create: `src/animation/hypercube.rs`
- Modify: `src/animation/mod.rs`
- Modify: `src/main.rs`

A tesseract (4D hypercube) has 16 vertices and 32 edges. We rotate in the XW and YZ 4D planes, then project 4D→3D→2D.

**Step 1: Add 4D math helpers**

Add to `src/animation/math.rs`:

```rust
/// A 4D point.
pub type Vec4 = [f64; 4];

/// Rotate in the XW plane (4D rotation).
pub fn rotate_xw(p: Vec4, angle: f64) -> Vec4 {
    let (s, c) = angle.sin_cos();
    [p[0] * c - p[3] * s, p[1], p[2], p[0] * s + p[3] * c]
}

/// Rotate in the YZ plane (4D rotation).
pub fn rotate_yz(p: Vec4, angle: f64) -> Vec4 {
    let (s, c) = angle.sin_cos();
    [p[0], p[1] * c - p[2] * s, p[1] * s + p[2] * c, p[3]]
}

/// Rotate in the XZ plane (4D rotation).
pub fn rotate_xz(p: Vec4, angle: f64) -> Vec4 {
    let (s, c) = angle.sin_cos();
    [p[0] * c - p[2] * s, p[1], p[0] * s + p[2] * c, p[3]]
}

/// Project from 4D to 3D using perspective projection.
pub fn project_4d_to_3d(p: Vec4, distance: f64) -> Vec3 {
    let w = p[3] + distance;
    if w.abs() < 0.001 {
        return [0.0, 0.0, 0.0];
    }
    let factor = distance / w;
    [p[0] * factor, p[1] * factor, p[2] * factor]
}
```

**Step 2: Add 4D math tests**

Add to the tests module in `src/animation/math.rs`:

```rust
    #[test]
    fn test_rotate_xw_identity() {
        let p = [1.0, 0.0, 0.0, 0.0];
        let r = rotate_xw(p, 0.0);
        assert!(approx_eq(&r, &[1.0, 0.0, 0.0, 0.0], 1e-10));
    }

    #[test]
    fn test_project_4d_to_3d() {
        let p = [1.0, 2.0, 3.0, 0.0];
        let r = project_4d_to_3d(p, 5.0);
        assert!(approx_eq(&r, &[1.0, 2.0, 3.0], 1e-10));
    }
```

**Step 3: Run tests**

Run: `cargo test animation::math`
Expected: all tests pass (original 6 + 2 new)

**Step 4: Write hypercube animation**

Write `src/animation/hypercube.rs`:

```rust
use crate::animation::math::{self, Vec4};
use crate::animation::Animation;
use ratatui::style::Color;
use ratatui::widgets::canvas::Context;

/// Generate the 16 vertices of a unit tesseract centered at origin.
fn tesseract_vertices() -> [Vec4; 16] {
    let mut verts = [[0.0; 4]; 16];
    for i in 0..16 {
        verts[i] = [
            if i & 1 != 0 { 1.0 } else { -1.0 },
            if i & 2 != 0 { 1.0 } else { -1.0 },
            if i & 4 != 0 { 1.0 } else { -1.0 },
            if i & 8 != 0 { 1.0 } else { -1.0 },
        ];
    }
    verts
}

/// Generate the 32 edges of a tesseract.
/// Two vertices are connected if they differ in exactly one coordinate.
fn tesseract_edges() -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for i in 0..16 {
        for j in (i + 1)..16 {
            let diff = i ^ j;
            if diff.count_ones() == 1 {
                edges.push((i, j));
            }
        }
    }
    edges
}

pub struct Hypercube {
    angle_xw: f64,
    angle_yz: f64,
    angle_xz: f64,
    vertices: [Vec4; 16],
    edges: Vec<(usize, usize)>,
}

impl Hypercube {
    pub fn new() -> Self {
        Self {
            angle_xw: 0.0,
            angle_yz: 0.0,
            angle_xz: 0.0,
            vertices: tesseract_vertices(),
            edges: tesseract_edges(),
        }
    }
}

impl Animation for Hypercube {
    fn update(&mut self, dt: f32) {
        let dt = dt as f64;
        self.angle_xw += 0.6 * dt;
        self.angle_yz += 0.4 * dt;
        self.angle_xz += 0.3 * dt;
    }

    fn draw(&self, ctx: &mut Context) {
        let distance_4d = 3.0;
        let distance_3d = 4.0;
        let base_scale = 20.0;

        let projected: Vec<[f64; 2]> = self
            .vertices
            .iter()
            .map(|&v| {
                // Scale
                let v = [v[0] * base_scale, v[1] * base_scale, v[2] * base_scale, v[3] * base_scale];
                // 4D rotations
                let v = math::rotate_xw(v, self.angle_xw);
                let v = math::rotate_yz(v, self.angle_yz);
                let v = math::rotate_xz(v, self.angle_xz);
                // 4D → 3D
                let v3 = math::project_4d_to_3d(v, distance_4d * base_scale);
                // 3D → 2D
                math::project(v3, distance_3d * base_scale)
            })
            .collect();

        for &(i, j) in &self.edges {
            // Color based on which dimension the edge spans
            let diff = i ^ j;
            let color = match diff {
                1 => Color::Rgb(100, 180, 255),  // x-edges: blue
                2 => Color::Rgb(255, 100, 180),  // y-edges: pink
                4 => Color::Rgb(100, 255, 180),  // z-edges: green
                8 => Color::Rgb(255, 220, 100),  // w-edges: gold
                _ => Color::White,
            };

            ctx.draw(&ratatui::widgets::canvas::Line {
                x1: projected[i][0],
                y1: projected[i][1],
                x2: projected[j][0],
                y2: projected[j][1],
                color,
            });
        }
    }

    fn name(&self) -> &'static str {
        "hypercube"
    }

    fn description(&self) -> &'static str {
        "4D tesseract rotation"
    }
}
```

**Step 5: Register in mod.rs and main.rs**

Add to `src/animation/mod.rs`:

```rust
pub mod hypercube;
```

In `src/main.rs`, update the animation match:

```rust
    let mut anim: Box<dyn Animation> = match cli.animation {
        AnimationChoice::Diamond => Box::new(Diamond::new()),
        AnimationChoice::Hypercube => Box::new(animation::hypercube::Hypercube::new()),
        _ => {
            eprintln!("Animation not yet implemented, using diamond");
            Box::new(Diamond::new())
        }
    };
```

**Step 6: Test it**

Run: `cargo run -- -a hypercube`
Expected: rotating tesseract with color-coded dimension edges

**Step 7: Commit**

```bash
git add src/animation/ src/main.rs
git commit -m "✨ feat(animation): add 4D hypercube (tesseract) animation"
```

---

### Task 10: Toroid Animation

**Files:**
- Create: `src/animation/toroid.rs`
- Modify: `src/animation/mod.rs`
- Modify: `src/main.rs`

The toroid uses particles flowing along a torus surface, rendered as braille dots.

**Step 1: Write toroid animation**

Write `src/animation/toroid.rs`:

```rust
use crate::animation::math;
use crate::animation::Animation;
use ratatui::style::Color;
use ratatui::widgets::canvas::{Context, Points};
use std::f64::consts::TAU;

const NUM_PARTICLES: usize = 600;

struct Particle {
    theta: f64, // major angle (around the ring)
    phi: f64,   // minor angle (around the tube)
    speed: f64,
}

pub struct Toroid {
    particles: Vec<Particle>,
    angle_x: f64,
    angle_y: f64,
    major_radius: f64,
    minor_radius: f64,
}

impl Toroid {
    pub fn new() -> Self {
        let mut particles = Vec::with_capacity(NUM_PARTICLES);
        for i in 0..NUM_PARTICLES {
            let frac = i as f64 / NUM_PARTICLES as f64;
            particles.push(Particle {
                theta: frac * TAU,
                phi: (frac * 7.0 * TAU) % TAU, // spread around the tube
                speed: 0.5 + (i % 5) as f64 * 0.2,
            });
        }
        Self {
            particles,
            angle_x: 0.4,
            angle_y: 0.0,
            major_radius: 20.0,
            minor_radius: 8.0,
        }
    }
}

impl Animation for Toroid {
    fn update(&mut self, dt: f32) {
        let dt = dt as f64;
        self.angle_y += 0.5 * dt;
        self.angle_x = 0.4 + 0.2 * (self.angle_y * 0.3).sin();

        for p in &mut self.particles {
            p.theta = (p.theta + p.speed * dt) % TAU;
            p.phi = (p.phi + p.speed * 1.5 * dt) % TAU;
        }
    }

    fn draw(&self, ctx: &mut Context) {
        let distance = 4.0 * self.major_radius;

        let coords: Vec<(f64, f64)> = self
            .particles
            .iter()
            .map(|p| {
                // Parametric torus: (R + r*cos(phi)) * cos(theta), ...
                let r = self.major_radius + self.minor_radius * p.phi.cos();
                let point = [
                    r * p.theta.cos(),
                    self.minor_radius * p.phi.sin(),
                    r * p.theta.sin(),
                ];
                let point = math::rotate_x(point, self.angle_x);
                let point = math::rotate_y(point, self.angle_y);
                let proj = math::project(point, distance);
                (proj[0], proj[1])
            })
            .collect();

        // Draw particles with varying brightness based on depth
        ctx.draw(&Points {
            coords: &coords,
            color: Color::Rgb(200, 160, 255),
        });

        // Draw a second layer with slightly different color for depth
        let coords2: Vec<(f64, f64)> = self
            .particles
            .iter()
            .enumerate()
            .filter(|(i, _)| i % 3 == 0)
            .map(|(_, p)| {
                let offset_phi = p.phi + 0.3;
                let r = self.major_radius + self.minor_radius * offset_phi.cos();
                let point = [
                    r * p.theta.cos(),
                    self.minor_radius * offset_phi.sin(),
                    r * p.theta.sin(),
                ];
                let point = math::rotate_x(point, self.angle_x);
                let point = math::rotate_y(point, self.angle_y);
                let proj = math::project(point, distance);
                (proj[0], proj[1])
            })
            .collect();

        ctx.draw(&Points {
            coords: &coords2,
            color: Color::Rgb(140, 100, 200),
        });
    }

    fn name(&self) -> &'static str {
        "toroid"
    }

    fn description(&self) -> &'static str {
        "Toroidal particle flow"
    }
}
```

**Step 2: Register in mod.rs and main.rs**

Add to `src/animation/mod.rs`:

```rust
pub mod toroid;
```

In `src/main.rs`, update the match:

```rust
        AnimationChoice::Toroid => Box::new(animation::toroid::Toroid::new()),
```

**Step 3: Test it**

Run: `cargo run -- -a toroid`
Expected: rotating donut made of flowing particles

**Step 4: Commit**

```bash
git add src/animation/ src/main.rs
git commit -m "✨ feat(animation): add toroidal particle flow animation"
```

---

### Task 11: Geodesic Sphere Animation

**Files:**
- Create: `src/animation/geodesic.rs`
- Modify: `src/animation/mod.rs`
- Modify: `src/main.rs`

An icosahedron subdivided once (~42 vertices) with vertices that breathe inward/outward.

**Step 1: Write geodesic sphere animation**

Write `src/animation/geodesic.rs`:

```rust
use crate::animation::math::{self, Vec3};
use crate::animation::Animation;
use ratatui::style::Color;
use ratatui::widgets::canvas::Context;

/// Generate the 12 vertices of a regular icosahedron.
fn icosahedron_vertices() -> Vec<Vec3> {
    let phi = (1.0 + 5.0_f64.sqrt()) / 2.0; // golden ratio
    let verts = vec![
        [-1.0, phi, 0.0],
        [1.0, phi, 0.0],
        [-1.0, -phi, 0.0],
        [1.0, -phi, 0.0],
        [0.0, -1.0, phi],
        [0.0, 1.0, phi],
        [0.0, -1.0, -phi],
        [0.0, 1.0, -phi],
        [phi, 0.0, -1.0],
        [phi, 0.0, 1.0],
        [-phi, 0.0, -1.0],
        [-phi, 0.0, 1.0],
    ];
    // Normalize all vertices to unit sphere
    verts
        .into_iter()
        .map(|v| {
            let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
            [v[0] / len, v[1] / len, v[2] / len]
        })
        .collect()
}

/// Icosahedron faces (20 triangles, vertex indices).
fn icosahedron_faces() -> Vec<[usize; 3]> {
    vec![
        [0, 11, 5], [0, 5, 1], [0, 1, 7], [0, 7, 10], [0, 10, 11],
        [1, 5, 9], [5, 11, 4], [11, 10, 2], [10, 7, 6], [7, 1, 8],
        [3, 9, 4], [3, 4, 2], [3, 2, 6], [3, 6, 8], [3, 8, 9],
        [4, 9, 5], [2, 4, 11], [6, 2, 10], [8, 6, 7], [9, 8, 1],
    ]
}

/// Subdivide an icosahedron once to create a geodesic sphere.
/// Returns (vertices, edges).
fn subdivide_icosahedron() -> (Vec<Vec3>, Vec<(usize, usize)>) {
    let base_verts = icosahedron_vertices();
    let faces = icosahedron_faces();

    let mut verts = base_verts.clone();
    let mut edge_set = std::collections::HashSet::new();
    let mut midpoint_cache = std::collections::HashMap::new();

    let mut get_midpoint = |a: usize, b: usize, verts: &mut Vec<Vec3>| -> usize {
        let key = if a < b { (a, b) } else { (b, a) };
        if let Some(&idx) = midpoint_cache.get(&key) {
            return idx;
        }
        let mid = [
            (verts[a][0] + verts[b][0]) / 2.0,
            (verts[a][1] + verts[b][1]) / 2.0,
            (verts[a][2] + verts[b][2]) / 2.0,
        ];
        // Normalize to unit sphere
        let len = (mid[0] * mid[0] + mid[1] * mid[1] + mid[2] * mid[2]).sqrt();
        let normalized = [mid[0] / len, mid[1] / len, mid[2] / len];
        let idx = verts.len();
        verts.push(normalized);
        midpoint_cache.insert(key, idx);
        idx
    };

    for face in &faces {
        let a = face[0];
        let b = face[1];
        let c = face[2];
        let ab = get_midpoint(a, b, &mut verts);
        let bc = get_midpoint(b, c, &mut verts);
        let ca = get_midpoint(c, a, &mut verts);

        // 4 sub-triangles produce 9 edges
        for &(i, j) in &[(a, ab), (ab, b), (b, bc), (bc, c), (c, ca), (ca, a), (ab, bc), (bc, ca), (ca, ab)] {
            let edge = if i < j { (i, j) } else { (j, i) };
            edge_set.insert(edge);
        }
    }

    let edges: Vec<(usize, usize)> = edge_set.into_iter().collect();
    (verts, edges)
}

pub struct Geodesic {
    base_vertices: Vec<Vec3>,
    edges: Vec<(usize, usize)>,
    angle_x: f64,
    angle_y: f64,
    time: f64,
}

impl Geodesic {
    pub fn new() -> Self {
        let (verts, edges) = subdivide_icosahedron();
        Self {
            base_vertices: verts,
            edges,
            angle_x: 0.0,
            angle_y: 0.0,
            time: 0.0,
        }
    }
}

impl Animation for Geodesic {
    fn update(&mut self, dt: f32) {
        let dt = dt as f64;
        self.time += dt;
        self.angle_x += 0.3 * dt;
        self.angle_y += 0.5 * dt;
    }

    fn draw(&self, ctx: &mut Context) {
        let base_scale = 25.0;
        let distance = 4.0 * base_scale;

        // Breathing effect: each vertex oscillates at slightly different phase
        let projected: Vec<[f64; 2]> = self
            .base_vertices
            .iter()
            .enumerate()
            .map(|(i, &v)| {
                let phase = i as f64 * 0.15;
                let breath = 1.0 + 0.15 * (self.time * 2.0 + phase).sin();
                let v = math::scale(v, base_scale * breath);
                let v = math::rotate_x(v, self.angle_x);
                let v = math::rotate_y(v, self.angle_y);
                math::project(v, distance)
            })
            .collect();

        for &(i, j) in &self.edges {
            let avg_phase = (i + j) as f64 * 0.075;
            let brightness = ((self.time * 1.5 + avg_phase).sin() * 0.3 + 0.7).clamp(0.3, 1.0);
            let g = (brightness * 255.0) as u8;
            let b = (brightness * 200.0) as u8;
            let color = Color::Rgb((g as f64 * 0.6) as u8, g, b);

            ctx.draw(&ratatui::widgets::canvas::Line {
                x1: projected[i][0],
                y1: projected[i][1],
                x2: projected[j][0],
                y2: projected[j][1],
                color,
            });
        }
    }

    fn name(&self) -> &'static str {
        "geodesic"
    }

    fn description(&self) -> &'static str {
        "Breathing geodesic sphere"
    }
}
```

**Step 2: Register in mod.rs and main.rs**

Add to `src/animation/mod.rs`:

```rust
pub mod geodesic;
```

In `src/main.rs`, update the match to replace the fallback:

```rust
    let mut anim: Box<dyn Animation> = match cli.animation {
        AnimationChoice::Diamond => Box::new(Diamond::new()),
        AnimationChoice::Hypercube => Box::new(animation::hypercube::Hypercube::new()),
        AnimationChoice::Toroid => Box::new(animation::toroid::Toroid::new()),
        AnimationChoice::Geodesic => Box::new(animation::geodesic::Geodesic::new()),
    };
```

Remove the `_ => { eprintln!(...) }` fallback arm.

**Step 3: Test all animations**

Run each:
```bash
cargo run -- -a diamond
cargo run -- -a hypercube
cargo run -- -a toroid
cargo run -- -a geodesic
```
Expected: each animation renders and animates correctly

**Step 4: Commit**

```bash
git add src/animation/ src/main.rs
git commit -m "✨ feat(animation): add breathing geodesic sphere, complete all 4 animations"
```

---

### Task 12: tachyonfx Post-Processing Effects

**Files:**
- Modify: `src/renderer/layout.rs`
- Modify: `src/main.rs`

Add subtle post-processing to the animation panel: a looping hsl color shift that gives the animation a living, breathing color feel.

**Step 1: Add effect state to the app**

Modify `src/main.rs` to track effect state. Add an `App` struct:

```rust
use std::time::Instant;
use tachyonfx::{fx, Effect, EffectManager};

struct App {
    animation: Box<dyn Animation>,
    info: sysinfo::SystemInfo,
    effects: EffectManager<&'static str>,
    last_tick: Instant,
    fps: u32,
}

impl App {
    fn new(animation: Box<dyn Animation>, info: sysinfo::SystemInfo, fps: u32) -> Self {
        let mut effects = EffectManager::new();
        // Repeating hsl hue shift on the animation area
        effects.add_unique_effect(
            "animation_glow",
            fx::never_complete(fx::ping_pong(fx::hsl_shift_fg(
                30.0,
                0.0,
                0.05,
                Duration::from_secs(3),
            ))),
        );
        Self {
            animation,
            info,
            effects,
            last_tick: Instant::now(),
            fps,
        }
    }
}
```

**Step 2: Update the render loop to apply effects**

Update `src/renderer/layout.rs` — add an `animation_area` return so main can apply effects to it:

Add a new public function:

```rust
/// Returns the Rect used by the animation panel (for applying effects).
pub fn animation_rect(area: Rect) -> Rect {
    let chunks = Layout::horizontal([
        Constraint::Percentage(60),
        Constraint::Percentage(40),
    ])
    .split(area);
    chunks[0]
}
```

Update the main loop in `src/main.rs`:

```rust
fn run(mut terminal: DefaultTerminal, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_secs_f64(1.0 / app.fps as f64);

    loop {
        let elapsed = app.last_tick.elapsed();

        terminal.draw(|frame| {
            renderer::layout::draw(frame, app.animation.as_ref(), &app.info);

            // Apply effects to animation area
            let anim_area = renderer::layout::animation_rect(frame.area());
            app.effects
                .process_effects(elapsed, frame.buffer_mut(), anim_area);
        })?;

        let timeout = tick_rate.saturating_sub(app.last_tick.elapsed());
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        KeyCode::Char('c')
                            if key
                                .modifiers
                                .contains(crossterm::event::KeyModifiers::CONTROL) =>
                        {
                            return Ok(());
                        }
                        _ => {}
                    }
                }
            }
        }

        if app.last_tick.elapsed() >= tick_rate {
            let dt = app.last_tick.elapsed().as_secs_f32();
            app.animation.update(dt);
            app.last_tick = Instant::now();
        }
    }
}
```

**Step 3: Verify effects render**

Run: `cargo run`
Expected: diamond animation with a subtle color shift cycling over time

**Step 4: Commit**

```bash
git add src/main.rs src/renderer/layout.rs
git commit -m "✨ feat(renderer): add tachyonfx post-processing color effects"
```

---

### Task 13: Polish and Final Verification

**Files:**
- Various small tweaks

**Step 1: Run all tests**

Run: `cargo test`
Expected: all tests pass

**Step 2: Run clippy**

Run: `cargo clippy -- -W clippy::all`
Expected: no errors (warnings acceptable for v1)
Fix any clippy errors.

**Step 3: Test all animations end-to-end**

```bash
cargo run -- -a diamond
cargo run -- -a hypercube
cargo run -- -a toroid
cargo run -- -a geodesic
cargo run -- --list
cargo run -- --help
cargo run -- --fps 60 -a toroid
```

Expected: all work correctly, clean exit on q/Esc/Ctrl+C

**Step 4: Final commit**

```bash
git add -A
git commit -m "🎨 style: clippy fixes and polish for v0.1.0"
```
