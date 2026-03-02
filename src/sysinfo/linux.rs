// Linux system information collection

use super::SystemInfo;

pub fn collect() -> SystemInfo {
    SystemInfo {
        hostname: String::new(),
        os: String::new(),
        kernel: String::new(),
        uptime: String::new(),
        cpu: String::new(),
        memory: String::new(),
        shell: String::new(),
        terminal: String::new(),
    }
}
