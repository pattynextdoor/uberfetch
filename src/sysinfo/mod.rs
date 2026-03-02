#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "linux")]
mod linux;

/// System information collected at startup.
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
    /// Collect system information for the current platform.
    pub fn collect() -> Self {
        #[cfg(target_os = "macos")]
        return macos::collect();
        #[cfg(target_os = "linux")]
        return linux::collect();
    }
}
