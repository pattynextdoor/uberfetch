use std::fs;
use std::process::Command;

use super::{cmd_output, SystemInfo};

pub fn collect() -> SystemInfo {
    SystemInfo {
        hostname: get_hostname(),
        os: get_os(),
        kernel: get_kernel(),
        uptime: get_uptime(),
        cpu: get_cpu(),
        gpu: get_gpu(),
        memory: get_memory(),
        disk: get_disk(),
        packages: get_packages(),
        shell: get_shell(),
        terminal: get_terminal(),
        de_wm: get_de_wm(),
        resolution: get_resolution(),
        battery: get_battery(),
    }
}

fn get_gpu() -> String {
    cmd_output("lspci", &[])
        .and_then(|raw| {
            raw.lines()
                .find(|l| l.contains("VGA") || l.contains("3D controller"))
                .and_then(|l| {
                    // Format: "00:02.0 VGA compatible controller: Vendor Device"
                    let after_type = l.splitn(3, ':').nth(2)?;
                    Some(after_type.trim().to_string())
                })
        })
        .unwrap_or_else(|| "Unknown".into())
}

fn get_disk() -> String {
    super::parse_df_root(&cmd_output("df", &["-h", "/"]).unwrap_or_default())
        .unwrap_or_else(|| "Unknown".into())
}

fn get_packages() -> String {
    // Try package managers in order of popularity
    if let Some(out) = cmd_output("dpkg-query", &["-f", ".\n", "-W"]) {
        let count = out.lines().count();
        if count > 0 {
            return format!("{count} (dpkg)");
        }
    }
    if let Some(out) = cmd_output("rpm", &["-qa"]) {
        let count = out.lines().count();
        if count > 0 {
            return format!("{count} (rpm)");
        }
    }
    if let Some(out) = cmd_output("pacman", &["-Q"]) {
        let count = out.lines().count();
        if count > 0 {
            return format!("{count} (pacman)");
        }
    }
    "Unknown".into()
}

fn get_de_wm() -> String {
    // Try environment variables first
    if let Ok(desktop) = std::env::var("XDG_CURRENT_DESKTOP") {
        if !desktop.is_empty() {
            return desktop;
        }
    }
    if let Ok(session) = std::env::var("DESKTOP_SESSION") {
        if !session.is_empty() {
            return session;
        }
    }
    // Fallback: scan for common WM processes
    let wms = [
        "sway",
        "hyprland",
        "i3",
        "bspwm",
        "dwm",
        "openbox",
        "fluxbox",
        "awesome",
        "xmonad",
        "herbstluftwm",
        "qtile",
    ];
    if let Some(procs) = cmd_output("ps", &["-eo", "comm"]) {
        for wm in &wms {
            if procs.lines().any(|l| l.trim() == *wm) {
                return (*wm).to_string();
            }
        }
    }
    "Unknown".into()
}

fn get_resolution() -> String {
    // Try xrandr first (X11)
    if let Some(raw) = cmd_output("xrandr", &["--current"]) {
        if let Some(res) = raw
            .lines()
            .find(|l| l.contains(" connected"))
            .and_then(|l| {
                l.split_whitespace().find(|w| {
                    w.contains('x') && w.chars().next().map_or(false, |c| c.is_ascii_digit())
                })
            })
        {
            // Strip off any +offset suffix (e.g. "1920x1080+0+0" → "1920x1080")
            return res.split('+').next().unwrap_or(res).to_string();
        }
    }
    // Fallback: xdpyinfo
    if let Some(raw) = cmd_output("xdpyinfo", &[]) {
        if let Some(line) = raw.lines().find(|l| l.contains("dimensions:")) {
            if let Some(dim) = line.split_whitespace().nth(1) {
                return dim.to_string();
            }
        }
    }
    "Unknown".into()
}

fn get_battery() -> Option<String> {
    // Try BAT0 then BAT1
    for bat in &["BAT0", "BAT1"] {
        let base = format!("/sys/class/power_supply/{bat}");
        if let Some(capacity) = read_file(&format!("{base}/capacity")) {
            let pct = capacity.trim();
            let status = read_file(&format!("{base}/status"))
                .map(|s| s.trim().to_lowercase())
                .unwrap_or_else(|| "unknown".into());
            let state = match status.as_str() {
                "charging" => "charging",
                "discharging" => "discharging",
                "full" => "full",
                _ => "unknown",
            };
            return Some(format!("{pct}% ({state})"));
        }
    }
    None
}

fn read_file(path: &str) -> Option<String> {
    fs::read_to_string(path).ok()
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
                .map(|l| {
                    l.trim_start_matches("PRETTY_NAME=")
                        .trim_matches('"')
                        .to_string()
                })
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
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "uptime seconds always fit in u64; negative values clamped to 0"
            )]
            let secs_u64 = secs.max(0.0) as u64;
            Some(super::format_uptime(secs_u64))
        })
        .unwrap_or_else(|| "Unknown".into())
}

fn get_cpu() -> String {
    read_file("/proc/cpuinfo").map_or_else(
        || "Unknown".into(),
        |content| {
            let model = content
                .lines()
                .find(|l| l.starts_with("model name"))
                .and_then(|l| l.split(':').nth(1))
                .map_or_else(|| "Unknown".into(), |s| s.trim().to_string());
            let cores = content
                .lines()
                .filter(|l| l.starts_with("processor"))
                .count();
            format!("{model} ({cores} cores)")
        },
    )
}

fn get_memory() -> String {
    read_file("/proc/meminfo").map_or_else(
        || "Unknown".into(),
        |content| {
            let parse_kb = |key: &str| -> u64 {
                content
                    .lines()
                    .find(|l| l.starts_with(key))
                    .and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
                    .unwrap_or(0)
            };
            let total_kb = parse_kb("MemTotal:");
            let available_kb = parse_kb("MemAvailable:");
            let used_kb = total_kb.saturating_sub(available_kb);
            super::format_memory(used_kb * 1024, total_kb * 1024)
        },
    )
}

fn get_shell() -> String {
    std::env::var("SHELL").ok().map_or_else(
        || "Unknown".into(),
        |s| {
            let name = s.rsplit('/').next().unwrap_or(&s).to_string();
            let version = Command::new(&s)
                .arg("--version")
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .and_then(|v| v.lines().next().map(ToString::to_string));
            match version {
                Some(v) if v.contains(&name) => v,
                _ => name,
            }
        },
    )
}

fn get_terminal() -> String {
    std::env::var("TERM_PROGRAM").unwrap_or_else(|_| "Unknown".into())
}
