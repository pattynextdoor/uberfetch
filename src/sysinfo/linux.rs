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
        memory: get_memory(),
        shell: get_shell(),
        terminal: get_terminal(),
    }
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
