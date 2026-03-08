use std::process::Command;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

/// Run an external command and return its trimmed stdout.
fn cmd_output(program: &str, args: &[&str]) -> Option<String> {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
}

/// System information collected at startup.
pub struct SystemInfo {
    pub hostname: String,
    pub os: String,
    pub kernel: String,
    pub uptime: String,
    pub cpu: String,
    pub gpu: String,
    pub memory: String,
    pub disk: String,
    pub packages: String,
    pub shell: String,
    pub terminal: String,
    pub de_wm: String,
    pub resolution: String,
    pub battery: Option<String>,
}

impl SystemInfo {
    /// Collect system information for the current platform.
    pub fn collect() -> Self {
        #[cfg(target_os = "macos")]
        return macos::collect();
        #[cfg(target_os = "linux")]
        return linux::collect();
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        compile_error!("uberfetch only supports macOS and Linux");
    }
}

/// Format seconds into a human-readable uptime string.
pub fn format_uptime(total_secs: u64) -> String {
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let mins = (total_secs % 3600) / 60;
    let plural = |n, word: &str| {
        if n == 1 {
            format!("{n} {word}")
        } else {
            format!("{n} {word}s")
        }
    };
    match (days, hours, mins) {
        (0, 0, m) => plural(m, "min"),
        (0, h, m) => format!("{}, {}", plural(h, "hour"), plural(m, "min")),
        (d, h, _) => format!("{}, {}", plural(d, "day"), plural(h, "hour")),
    }
}

/// Parse `df -h /` output into a display string like "123G / 500G (45%)".
///
/// Expects the standard `df -h` header + data layout where the second line
/// contains: Filesystem Size Used Avail Capacity ...
fn parse_df_root(raw: &str) -> Option<String> {
    let line = raw.lines().nth(1)?;
    let cols: Vec<&str> = line.split_whitespace().collect();
    // macOS: Filesystem Size Used Available Capacity ...
    // Linux:  Filesystem Size Used Avail Use% ...
    if cols.len() < 5 {
        return None;
    }
    let size = cols[1];
    let used = cols[2];
    let pct = cols[4].trim_end_matches('%');
    Some(format!("{used} / {size} ({pct}%)"))
}

/// Format memory usage as "X.X GiB / Y.Y GiB".
#[expect(
    clippy::cast_precision_loss,
    reason = "byte counts fit comfortably in f64 for display"
)]
pub fn format_memory(used_bytes: u64, total_bytes: u64) -> String {
    let used_gib = used_bytes as f64 / 1_073_741_824.0;
    let total_gib = total_bytes as f64 / 1_073_741_824.0;
    format!("{used_gib:.1} GiB / {total_gib:.1} GiB")
}

#[cfg(test)]
mod tests {
    use super::*;

    mod format_uptime {
        use super::*;

        #[test]
        fn returns_minutes_when_under_one_hour() {
            assert_eq!(format_uptime(300), "5 mins");
        }

        #[test]
        fn returns_singular_minute_for_one_minute() {
            assert_eq!(format_uptime(60), "1 min");
        }

        #[test]
        fn returns_hours_and_minutes_when_under_one_day() {
            assert_eq!(format_uptime(7500), "2 hours, 5 mins");
        }

        #[test]
        fn returns_singular_forms_for_one_hour_one_min() {
            assert_eq!(format_uptime(3660), "1 hour, 1 min");
        }

        #[test]
        fn returns_days_and_hours_for_long_uptimes() {
            assert_eq!(format_uptime(100_000), "1 day, 3 hours");
        }

        #[test]
        fn returns_zero_mins_for_zero_seconds() {
            assert_eq!(format_uptime(0), "0 mins");
        }
    }

    mod format_memory {
        use super::*;

        #[test]
        fn formats_gib_with_one_decimal() {
            let used = 8_589_934_592; // 8 GiB
            let total = 17_179_869_184; // 16 GiB
            assert_eq!(format_memory(used, total), "8.0 GiB / 16.0 GiB");
        }
    }

    mod parse_df_root_tests {
        use super::*;

        #[test]
        fn parses_macos_df_output() {
            let raw =
                "Filesystem     Size   Used  Avail Capacity  iused ifree %iused  Mounted on\n\
                        /dev/disk3s1  460Gi  123Gi  320Gi    28%  1234567  4567890   21%   /";
            assert_eq!(parse_df_root(raw).unwrap(), "123Gi / 460Gi (28%)");
        }

        #[test]
        fn parses_linux_df_output() {
            let raw = "Filesystem      Size  Used Avail Use% Mounted on\n\
                        /dev/sda1       100G   67G   33G  67% /";
            assert_eq!(parse_df_root(raw).unwrap(), "67G / 100G (67%)");
        }

        #[test]
        fn returns_none_for_empty_input() {
            assert!(parse_df_root("").is_none());
        }

        #[test]
        fn returns_none_for_header_only() {
            let raw = "Filesystem      Size  Used Avail Use% Mounted on";
            assert!(parse_df_root(raw).is_none());
        }
    }
}
