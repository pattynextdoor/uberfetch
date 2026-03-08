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
    cmd_output("system_profiler", &["SPDisplaysDataType"])
        .and_then(|raw| {
            raw.lines()
                .find(|l| l.contains("Chipset Model:"))
                .and_then(|l| Some(l.split(':').nth(1)?.trim().to_string()))
        })
        .unwrap_or_else(|| "Unknown".into())
}

fn get_disk() -> String {
    super::parse_df_root(&cmd_output("df", &["-h", "/"]).unwrap_or_default())
        .unwrap_or_else(|| "Unknown".into())
}

fn get_packages() -> String {
    let formula = cmd_output("brew", &["list", "--formula", "-1"]).map_or(0, |s| s.lines().count());
    let cask = cmd_output("brew", &["list", "--cask", "-1"]).map_or(0, |s| s.lines().count());
    let total = formula + cask;
    if total == 0 {
        "Unknown".into()
    } else {
        format!("{total} (brew)")
    }
}

fn get_de_wm() -> String {
    "Aqua".into()
}

fn get_resolution() -> String {
    cmd_output("system_profiler", &["SPDisplaysDataType"])
        .and_then(|raw| {
            raw.lines()
                .find(|l| l.contains("Resolution:"))
                .and_then(|l| Some(l.split(':').nth(1)?.trim().to_string()))
        })
        .unwrap_or_else(|| "Unknown".into())
}

fn get_battery() -> Option<String> {
    let raw = cmd_output("pmset", &["-g", "batt"])?;
    parse_pmset_battery(&raw)
}

/// Parse `pmset -g batt` output into a display string like "75% (charging)".
fn parse_pmset_battery(raw: &str) -> Option<String> {
    let line = raw.lines().find(|l| l.contains("InternalBattery"))?;
    let pct = line.split('\t').nth(1)?.split(';').next()?.trim();
    let state = if line.contains("charging") && !line.contains("discharging") {
        "charging"
    } else if line.contains("discharging") {
        "discharging"
    } else {
        "full"
    };
    Some(format!("{pct} ({state})"))
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

#[cfg(test)]
mod tests {
    use super::*;

    mod parse_pmset_battery_tests {
        use super::*;

        #[test]
        fn parses_charging_battery() {
            let raw = "Now drawing from 'AC Power'\n \
                        -InternalBattery-0 (id=1234)\t75%; charging; 1:23 remaining present: true";
            assert_eq!(parse_pmset_battery(raw).unwrap(), "75% (charging)");
        }

        #[test]
        fn parses_discharging_battery() {
            let raw = "Now drawing from 'Battery Power'\n \
                        -InternalBattery-0 (id=1234)\t42%; discharging; 3:45 remaining present: true";
            assert_eq!(parse_pmset_battery(raw).unwrap(), "42% (discharging)");
        }

        #[test]
        fn parses_full_battery() {
            let raw = "Now drawing from 'AC Power'\n \
                        -InternalBattery-0 (id=1234)\t100%; charged; 0:00 remaining present: true";
            assert_eq!(parse_pmset_battery(raw).unwrap(), "100% (full)");
        }

        #[test]
        fn returns_none_when_no_battery() {
            let raw = "Now drawing from 'AC Power'\n No battery.";
            assert!(parse_pmset_battery(raw).is_none());
        }
    }
}
