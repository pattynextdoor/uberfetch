use std::process::Command;

use super::SystemInfo;

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
    sysctl("kern.boottime")
        .and_then(|raw| {
            let sec_str = raw.split("sec = ").nth(1)?;
            let sec: u64 = sec_str.split(',').next()?.trim().parse().ok()?;
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .ok()?
                .as_secs();
            Some(super::format_uptime(now.saturating_sub(sec)))
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

    let page_size: u64 = sysctl("hw.pagesize")
        .and_then(|s| s.parse().ok())
        .unwrap_or(16384);

    let used_bytes = cmd_output("vm_stat", &[]).map_or(0, |raw| {
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
    });

    if total_bytes == 0 {
        "Unknown".into()
    } else {
        super::format_memory(used_bytes, total_bytes)
    }
}

fn get_shell() -> String {
    std::env::var("SHELL").ok().map_or_else(
        || "Unknown".into(),
        |s| {
            let name = s.rsplit('/').next().unwrap_or(&s).to_string();
            let version = cmd_output(&s, &["--version"])
                .and_then(|v| v.lines().next().map(std::string::ToString::to_string));
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
