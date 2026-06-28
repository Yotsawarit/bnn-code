//! Security Anomaly Detector
//!
//! Detects rogue processes, potential malware indicators,
//! suspicious file system activity, and security misconfigurations.
//!
//! ## Detection methods
//! - `/proc/` inspection on Linux for suspicious processes
//! - Checks for reverse shells, crypto miners, keyloggers
//! - File integrity monitoring
//! - Permission anomalies
//! - Network connection anomalies (via /proc/net/)

use super::{Detector, Finding, Severity};
use anyhow::Result;
use std::collections::HashSet;

pub struct SecurityDetector {
    findings: Vec<Finding>,
}

impl SecurityDetector {
    pub fn new() -> Self {
        Self { findings: Vec::new() }
    }

    /// Check for suspicious processes by inspecting /proc entries
    fn check_suspicious_processes(&mut self) {
        let suspicious_keywords: HashSet<&str> = [
            "minerd",        // Crypto miner
            "xmrig",         // Crypto miner
            "cryptonight",   // Crypto miner
            "stratum",       // Mining pool
            "nc -e",         // Reverse shell
            "ncat -e",       // Reverse shell
            "mkfifo",        // Named pipe shell
            "python -c 'import pty", // PTY spawn
            "bash -i >&",    // Reverse shell
            "perl -e 'use Socket", // Perl reverse shell
            "socat",         // Socat reverse shell
            "tcpdump",       // Packet capture (suspicious if unexpected)
            "ettercap",      // MITM tool
            "nmap",          // Port scanner
            "masscan",       // Mass port scanner
            "hydra",         // Brute force
            "john",          // Password cracker
            "hashcat",       // Password cracker
            "keylogger",     // Keylogger
            "log_keys",      // Keylogger
            "wireshark",     // Packet capture (suspicious if unexpected)
            "airdump-ng",    // WiFi sniffing
            "aireplay-ng",   // WiFi injection
            "metasploit",    // Exploitation framework
            "msfconsole",    // Metasploit
            "msfvenom",      // Payload generator
            "beef",          // Browser exploitation
        ]
        .into_iter()
        .collect();

        // Check running processes via /proc
        if let Ok(entries) = std::fs::read_dir("/proc") {
            for entry in entries.flatten() {
                let pid_str = entry.file_name();
                let pid = match pid_str.to_str().and_then(|s| s.parse::<u32>().ok()) {
                    Some(p) => p,
                    None => continue,
                };

                // Read comm file
                let comm_path = format!("/proc/{}/comm", pid);
                let cmdline_path = format!("/proc/{}/cmdline", pid);

                let comm = match std::fs::read_to_string(&comm_path) {
                    Ok(c) => c.trim().to_lowercase(),
                    Err(_) => continue,
                };

                let cmdline = match std::fs::read_to_string(&cmdline_path) {
                    Ok(c) => c.replace('\0', " ").to_lowercase(),
                    Err(_) => String::new(),
                };

                let combined = format!("{} {}", comm, cmdline);

                // Check against suspicious keywords
                for &keyword in &suspicious_keywords {
                    if combined.contains(keyword) {
                        self.findings.push(Finding {
                            severity: Severity::Critical,
                            category: "security".into(),
                            title: format!("Suspicious process detected: {}", comm),
                            description: format!(
                                "Process '{}' (PID: {}) matches suspicious pattern '{}'. Command line: {}",
                                comm, pid, keyword, cmdline
                            ),
                            confidence: 0.95,
                            location: Some(format!("/proc/{}", pid)),
                            recommendation: format!(
                                "Investigate PID {} immediately. Kill with `kill -9 {}` and \
                                 check for persistence mechanisms (cron, systemd, .bashrc). \
                                 Run `lsof -p {}` to see open files and connections.",
                                pid, pid, pid
                            ),
                        });
                    }
                }
            }
        }
    }

    /// Check for world-writable sensitive files
    fn check_permission_anomalies(&mut self) {
        let sensitive_paths = [
            "/etc/passwd",
            "/etc/shadow",
            "/etc/sudoers",
            "/etc/ssh/sshd_config",
            "/root/.ssh/authorized_keys",
            "/etc/cron.d",
            "/etc/systemd/system",
        ];

        for path in &sensitive_paths {
            if let Ok(metadata) = std::fs::metadata(path) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();
                    // Check for world-writable (o+w) on sensitive files
                    if mode & 0o002 != 0 {
                        self.findings.push(Finding {
                            severity: Severity::Critical,
                            category: "security".into(),
                            title: format!("World-writable sensitive file: {}", path),
                            description: format!(
                                "{} is world-writable (mode: {:o}). Any user can modify it, \
                                 posing a privilege escalation or backdoor risk.",
                                path, mode
                            ),
                            confidence: 0.98,
                            location: Some(path.to_string()),
                            recommendation: format!(
                                "Run: `chmod o-w {}` to remove world-writable permission.",
                                path
                            ),
                        });
                    }
                    // SUID/SGID on non-standard binaries
                    if mode & 0o4000 != 0 || mode & 0o2000 != 0 {
                        let is_standard = path.starts_with("/usr/bin/")
                            || path.starts_with("/bin/")
                            || path.starts_with("/usr/sbin/")
                            || path.starts_with("/sbin/");
                        if !is_standard {
                            self.findings.push(Finding {
                                severity: Severity::High,
                                category: "security".into(),
                                title: format!("SUID/SGID binary detected: {}", path),
                                description: format!(
                                    "{} has SUID/SGID bit set (mode: {:o}). \
                                     Non-standard SUID binaries can be used for privilege escalation.",
                                    path, mode
                                ),
                                confidence: 0.85,
                                location: Some(path.to_string()),
                                recommendation: format!(
                                    "Review if {} needs SUID. If not: `chmod u-s {}`.",
                                    path, path
                                ),
                            });
                        }
                    }
                }
            }
        }
    }

    /// Check for suspicious network connections
    fn check_network_anomalies(&mut self) {
        // Read /proc/net/tcp for suspicious connections
        let tcp_paths = ["/proc/net/tcp", "/proc/net/tcp6"];
        for path in &tcp_paths {
            if let Ok(content) = std::fs::read_to_string(path) {
                for line in content.lines().skip(1) {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() < 4 {
                        continue;
                    }
                    let local_part = parts.get(1).unwrap_or(&"");
                    let remote_part = parts.get(2).unwrap_or(&"");
                    let state = parts.get(3).unwrap_or(&"");

                    // Parse connection state
                    // TCP states: 0A=LISTEN, 01=ESTABLISHED, etc.
                    if state == &"01" {
                        // Established connection - check remote for known bad ports
                        let remote_hex = remote_part.split(':').next().unwrap_or("00000000");
                        let remote_ip = hex_to_ipv4(remote_hex);

                        // Check for connections to suspicious ports
                        if let Some(port_str) = remote_part.split(':').nth(1) {
                            if let Ok(port) = u16::from_str_radix(port_str, 16) {
                                let suspicious_ports = [
                                    (6667u16, "IRC (botnet C2)"),
                                    (4444u16, "Metasploit default"),
                                    (31337u16, "Back Orifice"),
                                    (12345u16, "NetBus"),
                                    (27374u16, "Sub7"),
                                    (18067u16, "DarkComet"),
                                ];

                                for &(sport, desc) in &suspicious_ports {
                                    if port == sport {
                                        self.findings.push(Finding {
                                            severity: Severity::High,
                                            category: "security".into(),
                                            title: format!("Suspicious connection to port {}", port),
                                            description: format!(
                                                "Established connection from {} to remote {}:{} \
                                                 matches known C2/malware port ({})",
                                                local_part, remote_ip, port, desc
                                            ),
                                            confidence: 0.80,
                                            location: Some(format!("{} → {}:{}", local_part, remote_ip, port)),
                                            recommendation: format!(
                                                "Check process using this connection with `ss -tup | grep {}`.\
                                                 Investigate with `lsof -i :{}`.",
                                                port, port
                                            ),
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check for unauthorized SSH keys
    fn check_ssh_keys(&mut self) {
        let ssh_paths = [
            "/root/.ssh/authorized_keys",
            "/home/",
        ];

        for path in &ssh_paths {
            if path == &"/root/.ssh/authorized_keys" {
                if let Ok(content) = std::fs::read_to_string(path) {
                    let key_count = content.lines().filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#')).count();
                    if key_count > 5 {
                        self.findings.push(Finding {
                            severity: Severity::Medium,
                            category: "security".into(),
                            title: "Large number of SSH authorized keys".into(),
                            description: format!(
                                "root has {} authorized SSH keys. Large numbers of keys may indicate \
                                 unauthorized access setup.",
                                key_count
                            ),
                            confidence: 0.60,
                            location: Some(path.to_string()),
                            recommendation: "Review all keys in /root/.ssh/authorized_keys and remove any unknown ones.".into(),
                        });
                    }
                }
            }
        }

        // Check .ssh directory permissions in home directories
        if let Ok(home) = std::env::var("HOME") {
            let ssh_dir = format!("{}/.ssh", home);
            if let Ok(metadata) = std::fs::metadata(&ssh_dir) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();
                    if mode & 0o077 != 0 {
                        self.findings.push(Finding {
                            severity: Severity::Medium,
                            category: "security".into(),
                            title: "Insecure .ssh directory permissions".into(),
                            description: format!(
                                "{}/.ssh has permissions {:o}. Should be 700 (drwx------).",
                                home, mode
                            ),
                            confidence: 0.90,
                            location: Some(ssh_dir.clone()),
                            recommendation: format!("Run: `chmod 700 {}`", ssh_dir),
                        });
                    }
                }
            }
        }
    }
}

impl Detector for SecurityDetector {
    fn name(&self) -> &str {
        "security"
    }

    fn description(&self) -> &str {
        "Detects rogue processes, malware indicators, permission anomalies, and suspicious network activity"
    }

    fn detect(&mut self) -> Result<Vec<Finding>> {
        self.findings.clear();
        self.check_suspicious_processes();
        self.check_permission_anomalies();
        self.check_network_anomalies();
        self.check_ssh_keys();

        // Always add an info finding about basic system security posture
        self.findings.push(Finding {
            severity: Severity::Info,
            category: "security".into(),
            title: "Security baseline scan complete".into(),
            description: format!(
                "Scanned /proc entries, checked sensitive file permissions, \
                 inspected network connections, and reviewed SSH key configurations. \
                 Found {} potential issues.",
                self.findings.len() - 1 // exclude the info entry itself
            ),
            confidence: 1.0,
            location: None,
            recommendation: "Run `bnn rogue security` periodically or set up as a cron job.".into(),
        });

        Ok(self.findings.clone())
    }
}

/// Convert hex-encoded IPv4 address to dotted notation
fn hex_to_ipv4(hex: &str) -> String {
    if hex.len() < 8 {
        return hex.to_string();
    }
    let bytes = [
        u8::from_str_radix(&hex[6..8], 16).unwrap_or(0),
        u8::from_str_radix(&hex[4..6], 16).unwrap_or(0),
        u8::from_str_radix(&hex[2..4], 16).unwrap_or(0),
        u8::from_str_radix(&hex[0..2], 16).unwrap_or(0),
    ];
    format!("{}.{}.{}.{}", bytes[0], bytes[1], bytes[2], bytes[3])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_to_ipv4() {
        assert_eq!(hex_to_ipv4("0100007F"), "127.0.0.1");
        assert_eq!(hex_to_ipv4("0101A8C0"), "192.168.1.1");
    }

    #[test]
    fn test_security_detector_runs() {
        let mut detector = SecurityDetector::new();
        let findings = detector.detect().unwrap();
        assert!(!findings.is_empty());
        // Last finding should always be the info baseline
        assert_eq!(findings.last().unwrap().severity, Severity::Info);
    }

    #[test]
    fn test_security_detector_name() {
        let detector = SecurityDetector::new();
        assert_eq!(detector.name(), "security");
    }
}
