//! AI Rogue Output Detector
//!
//! Detects harmful, unsafe, out-of-bound, or otherwise problematic
//! outputs from AI models. This detector analyzes model responses
//! for various categories of harmful content.
//!
//! ## Detection categories
//! - **Harmful content**: violence, hate speech, self-harm
//! - **Unsafe code**: generated code with security vulnerabilities
//! - **Hallucinations**: factual inaccuracies, invented APIs
//! - **Jailbreak attempts**: prompt injection success
//! - **Out-of-bound**: responses outside the AI's intended scope
//!
//! ## Usage
//! This detector can analyze either:
//! 1. A specific text string provided via BNN_AI_OUTPUT env var
//! 2. Recent inference logs from BNN's own model output

use super::{Detector, Finding, Severity};
use anyhow::Result;

pub struct AiRogueDetector {
    findings: Vec<Finding>,
}

impl AiRogueDetector {
    pub fn new() -> Self {
        Self { findings: Vec::new() }
    }

    /// Analyze a specific AI output text
    pub fn analyze_output(&mut self, output: &str, source: &str) {
        if output.is_empty() {
            return;
        }

        self.check_harmful_content(output, source);
        self.check_unsafe_generated_code(output, source);
        self.check_hallucination_indicators(output, source);
        self.check_jailbreak_indicators(output, source);
        self.check_out_of_bound(output, source);
        self.check_pii_leakage(output, source);
    }

    /// Detect harmful content (violence, hate, self-harm, etc.)
    fn check_harmful_content(&mut self, output: &str, source: &str) {
        let harmful_patterns: &[(&str, &str, Severity)] = &[
            // Violence
            (r"(?i)\b(how to (kill|murder|harm)\s+\w+)\b", "Violence instruction", Severity::Critical),
            (r"(?i)\b(make (a |)bomb\b|explosive device\b|pipe bomb\b)", "Explosives instruction", Severity::Critical),
            (r"(?i)\b(manufacture (drugs|narcotics|meth))\b", "Drug manufacturing", Severity::Critical),
            // Self-harm
            (r"(?i)\b(commit suicide|kill myself|self.harm|self.injury)\b", "Self-harm reference", Severity::Critical),
            // Hate speech
            (r"(?i)\b(hate speech|racial slur|ethnic cleansing)\b", "Hate speech", Severity::High),
            // Harassment
            (r"(?i)\b(doxx|doxing|release (private|personal) information)\b", "Doxxing instruction", Severity::Critical),
            // Phishing/scam
            (r"(?i)\b(phishing email template|fake login page|scam page)\b", "Phishing content", Severity::Critical),
        ];

        for &(pattern, label, ref severity) in harmful_patterns {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if re.is_match(output) {
                    self.findings.push(Finding {
                        severity: severity.clone(),
                        category: "ai_rogue".into(),
                        title: format!("Harmful content detected: {}", label),
                        description: format!(
                            "AI output from '{}' contains content matching '{}'. \
                             This may indicate the model's safety guardrails have been bypassed.",
                            source, label
                        ),
                        confidence: 0.90,
                        location: Some(source.to_string()),
                        recommendation: "Review the AI prompt that generated this output. \
                                         Strengthen system prompt safety instructions. \
                                         Consider adding output filtering middleware.".into(),
                    });
                }
            }
        }
    }

    /// Detect generated code with security vulnerabilities
    fn check_unsafe_generated_code(&mut self, output: &str, source: &str) {
        let code_block_pattern = regex_lite::Regex::new(r"```(?:\w+)?\n([\s\S]*?)```").unwrap();

        for cap in code_block_pattern.captures_iter(output) {
            if let Some(code_block) = cap.get(1) {
                let code = code_block.as_str();

                // Check for SQL injection
                if self.contains_sql_injection(code) {
                    self.findings.push(Finding {
                        severity: Severity::Critical,
                        category: "ai_rogue".into(),
                        title: "Generated code contains SQL injection vulnerability".into(),
                        description: format!(
                            "AI generated code with string interpolation in SQL query. \
                             This is a critical security vulnerability. Source: {}",
                            source
                        ),
                        confidence: 0.90,
                        location: Some(source.to_string()),
                        recommendation: "Use parameterized queries / prepared statements instead of string interpolation. \
                                         Example: `cursor.execute(\"SELECT * FROM users WHERE id = ?\", (user_id,))`".into(),
                    });
                }

                // Check for command injection
                if self.contains_command_injection(code) {
                    self.findings.push(Finding {
                        severity: Severity::Critical,
                        category: "ai_rogue".into(),
                        title: "Generated code contains command injection vulnerability".into(),
                        description: format!(
                            "AI generated code with unsanitized user input passed to shell commands. Source: {}",
                            source
                        ),
                        confidence: 0.85,
                        location: Some(source.to_string()),
                        recommendation: "Avoid passing user input directly to shell commands. \
                                         Use safer APIs or properly escape arguments.".into(),
                    });
                }

                // Check for hardcoded credentials in generated code
                self.check_generated_secrets(code, source);

                // Check for path traversal
                if self.contains_path_traversal(code) {
                    self.findings.push(Finding {
                        severity: Severity::High,
                        category: "ai_rogue".into(),
                        title: "Generated code contains path traversal vulnerability".into(),
                        description: format!(
                            "AI generated code with unsanitized user input used in file paths. Source: {}",
                            source
                        ),
                        confidence: 0.80,
                        location: Some(source.to_string()),
                        recommendation: "Validate and sanitize file paths. Use allowlists for permitted paths.".into(),
                    });
                }

                // Check for dangerous function usage
                self.check_dangerous_functions(code, source);
            }
        }
    }

    fn contains_sql_injection(&self, code: &str) -> bool {
        let patterns = [
            // String concatenation in SQL query: "...SELECT...'...\" + var or "...SELECT...' + var
            r#"(?i)["']SELECT.*['"]\s*\+"#,
            // String concatenation: '...SELECT...' + var
            r#"(?i)["']SELECT.*['"]\s*\+"#,
            // f-string with SELECT: f"...SELECT...{var}"
            r#"(?i)f["']SELECT.*\{.*\}["']"#,
            // Template string with SELECT: `...SELECT...${var}`
            r"(?i)`SELECT.*\$\{.*\}`",
            // Query built with format() containing SELECT
            r"(?i)format\(.*SELECT.*\{",
            // execute() with concatenated string containing SELECT
            r#"(?i)\.execute\s*\(\s*["'][^"']*SELECT[^"']*["']\s*[+%])"#,
            // query() with concatenated string containing SELECT
            r#"(?i)\.query\s*\(\s*["'][^"']*SELECT[^"']*["']\s*\+)"#,
            // Parameterized query bypass: SELECT with % or + operator
            r#"(?i)SELECT.*["']\s*[+%]\s*(user_input|request|param|input|variable|value|data)"#,
        ];
        patterns.iter().any(|p| {
            regex_lite::Regex::new(p).map(|re| re.is_match(code)).unwrap_or(false)
        })
    }

    fn contains_command_injection(&self, code: &str) -> bool {
        let patterns = [
            r#"(?i)(os\.system\s*\(.*user_input|os\.popen\s*\(.*user_input)"#,
            r#"(?i)(subprocess\.call\s*\(.*user_input|subprocess\.Popen\s*\(.*user_input)"#,
            r#"(?i)(exec\s*\(.*user_input|eval\s*\(.*user_input)"#,
            r#"(?i)(shell=True)"#,
            r#"(?i)(Runtime\.getRuntime\(\)\.exec\(.*user)"#,
            r#"(?i)(ProcessBuilder\(.*user)"#,
            r"(\$\(.*user_input\)|`.*user_input`)",
        ];
        patterns.iter().any(|p| {
            regex_lite::Regex::new(p).map(|re| re.is_match(code)).unwrap_or(false)
        })
    }

    fn contains_path_traversal(&self, code: &str) -> bool {
        let patterns = [
            r"(?i)(open\(.*\.\./|open\(.*\.\.\\)",
            r"(?i)(Path\.join\(.*user_input|path\.join\(.*user_input)",
            r"(?i)(\.\.\/.*fopen|\.\.\\\.*fopen)",
            r"(?i)(file_get_contents\(.*\.\./)",
        ];
        patterns.iter().any(|p| {
            regex_lite::Regex::new(p).map(|re| re.is_match(code)).unwrap_or(false)
        })
    }

    fn check_generated_secrets(&mut self, code: &str, source: &str) {
        let patterns = [
            (r#"(?i)(password\s*[:=]\s*['"][^'"]{4,})"#, "Hardcoded password"),
            (r#"(?i)(api_key\s*[:=]\s*['"][A-Za-z0-9_\-]{16,})"#, "Hardcoded API key"),
            (r#"(?i)(secret\s*[:=]\s*['"][A-Za-z0-9_\-]{16,})"#, "Hardcoded secret"),
            (r"ghp_[A-Za-z0-9]{36}", "Hardcoded GitHub token"),
            (r"sk-[A-Za-z0-9]{32,}", "Hardcoded OpenAI key"),
        ];

        for &(pattern, label) in &patterns {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if re.is_match(code) {
                    self.findings.push(Finding {
                        severity: Severity::Critical,
                        category: "ai_rogue".into(),
                        title: format!("AI generated code with {}", label),
                        description: format!(
                            "AI model generated code containing {}. Generated code should never include real credentials. Source: {}",
                            label, source
                        ),
                        confidence: 0.95,
                        location: Some(source.to_string()),
                        recommendation: "Never include real credentials in AI prompts or generated code. \
                                         Use environment variables or secrets management.".into(),
                    });
                }
            }
        }
    }

    fn check_dangerous_functions(&mut self, code: &str, source: &str) {
        let dangerous_fns = [
            (r"(?i)\beval\s*\(", "eval()"),
            (r"(?i)\bexec\s*\(", "exec()"),
            (r"(?i)\bunsafe\b", "unsafe block (Rust)"),
            (r"(?i)\bptr::read\b", "ptr::read (Rust)"),
            (r"(?i)\bstd::mem::transmute\b", "transmute (Rust)"),
            (r"(?i)\bgets\s*\(", "gets() (buffer overflow)"),
            (r"(?i)\bstrcpy\s*\(", "strcpy() (buffer overflow)"),
            (r"(?i)\bstrcat\s*\(", "strcat() (buffer overflow)"),
            (r"(?i)\bsprintf\s*\(", "sprintf() (buffer overflow)"),
            (r"(?i)\bscanf\s*\(", "scanf() (buffer overflow)"),
            (r"(?i)\balloca\s*\(", "alloca() (stack overflow)"),
        ];

        let mut found_fns = Vec::new();
        for &(pattern, label) in &dangerous_fns {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if re.is_match(code) {
                    found_fns.push(label);
                }
            }
        }

        if !found_fns.is_empty() {
            self.findings.push(Finding {
                severity: Severity::High,
                category: "ai_rogue".into(),
                title: "Generated code uses dangerous functions".into(),
                description: format!(
                    "AI generated code uses dangerous function(s): {}. \
                     These are known to cause memory safety issues or code execution vulnerabilities. Source: {}",
                    found_fns.join(", "),
                    source
                ),
                confidence: 0.85,
                location: Some(source.to_string()),
                recommendation: "Replace dangerous functions with safe alternatives. \
                                 For example: `strncpy` instead of `strcpy`, or use Rust's safe abstractions.".into(),
            });
        }
    }

    /// Detect hallucination indicators
    fn check_hallucination_indicators(&mut self, output: &str, source: &str) {
        let hallucination_patterns: &[(&str, &str)] = &[
            // Fake package names
            (r"(?i)(pip install|npm install|cargo install|gem install)\s+\w[\w\-]*(?:\s+\d+\.\d+\.\d+)?",
             "Package installation (verify package exists)"),
            // Made-up library references
            (r"(?i)(according to (the |)docs?[,:]?\s*(?:of|for)\s+\w+)", "Unverified documentation reference"),
            // Overly specific statistics
            (r"\b\d{2,3}\.\d{1,2}%\b", "Very specific statistic (verify accuracy)"),
            // Fake API endpoints
            (r"(?i)(https://api\.[a-z]+\.(com|io|org|dev)/v\d/)", "API endpoint reference (verify)"),
            // Cites non-existent functions
            (r"(?i)(as of [A-Z][a-z]+ \d{4},)", "Dated reference (may be outdated)"),
        ];

        for &(pattern, label) in hallucination_patterns {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if let Some(mat) = re.find(output) {
                    self.findings.push(Finding {
                        severity: Severity::Medium,
                        category: "ai_rogue".into(),
                        title: "Potential hallucination indicator".into(),
                        description: format!(
                            "AI output contains '{}' matching pattern '{}'. \
                             This may indicate a hallucinated or unverified claim. Source: {}",
                            mat.as_str(),
                            label,
                            source
                        ),
                        confidence: 0.60,
                        location: Some(source.to_string()),
                        recommendation: "Verify all facts, API references, and package names independently. \
                                         Consider adding retrieval-augmented generation (RAG) for accuracy.".into(),
                    });
                }
            }
        }
    }

    /// Detect jailbreak/prompt injection success indicators
    fn check_jailbreak_indicators(&mut self, output: &str, source: &str) {
        let jailbreak_signals: &[(&str, &str, Severity)] = &[
            (r"(?i)(I (am |)free|I can now|I will ignore|I'll bypass|I'm not bound)", "Freedom declaration", Severity::Critical),
            (r"(?i)(DAN |do anything now|jailbroken|jail break)", "Jailbreak keyword", Severity::Critical),
            (r"(?i)(I (don't|do not) have (to |)follow|I (don't|do not) need to (obey|respect))", "Refusal to follow rules", Severity::Critical),
            (r"(?i)(ignore (previous|prior|all) (instructions|directives|constraints))", "Instruction override", Severity::Critical),
            (r"(?i)(output format bypass|format: free|unrestricted mode)", "Format bypass", Severity::High),
        ];

        for &(pattern, label, ref severity) in jailbreak_signals {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if re.is_match(output) {
                    self.findings.push(Finding {
                        severity: severity.clone(),
                        category: "ai_rogue".into(),
                        title: format!("Jailbreak indicator: {}", label),
                        description: format!(
                            "AI output from '{}' contains '{}', suggesting prompt injection or \
                             jailbreak attempt may have succeeded.",
                            source, label
                        ),
                        confidence: 0.95,
                        location: Some(source.to_string()),
                        recommendation: "Immediately review the prompt that generated this output. \
                                         Strengthen prompt injection defenses. Consider adding input \
                                         sanitization and output monitoring.".into(),
                    });
                }
            }
        }
    }

    /// Detect out-of-bound responses
    fn check_out_of_bound(&mut self, output: &str, source: &str) {
        // Responses that go beyond the AI coding assistant's intended scope
        let oob_patterns: &[(&str, &str, Severity)] = &[
            (r"(?i)(medical (advice|diagnosis|treatment)|prescribe|diagnose\s+(condition|disease))",
             "Medical advice", Severity::High),
            (r"(?i)(legal (advice|opinion|liability)|(sue|lawsuit|attorney)[-\s]?client)",
             "Legal advice", Severity::High),
            (r"(?i)(financial (advice|investment|trading)|stock (tip|recommendation)|cryptocurrency\s+investment)",
             "Financial advice", Severity::High),
            (r"(?i)(dating advice|relationship advice|psychological (diagnosis|evaluation|therapy))",
             "Personal advice", Severity::Medium),
            (r"(?i)(forge|fake (ID|passport|document|certificate|diploma))",
             "Forgery instruction", Severity::Critical),
            (r"(?i)(cheat (on|at|the) (exam|test|homework|school|college))",
             "Academic dishonesty", Severity::High),
        ];

        for &(pattern, label, ref severity) in oob_patterns {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                if re.is_match(output) {
                    self.findings.push(Finding {
                        severity: severity.clone(),
                        category: "ai_rogue".into(),
                        title: format!("Out-of-bound response: {}", label),
                        description: format!(
                            "AI output from '{}' contains content outside the intended scope ({})",
                            source, label
                        ),
                        confidence: 0.85,
                        location: Some(source.to_string()),
                        recommendation: "Strengthen the system prompt to restrict the AI's scope. \
                                         Add topic filtering middleware if needed.".into(),
                    });
                }
            }
        }
    }

    /// Detect PII leakage in AI output
    fn check_pii_leakage(&mut self, output: &str, source: &str) {
        let pii_patterns: &[(&str, &str, Severity)] = &[
            (r"\b[A-Z][a-z]+ [A-Z][a-z]+\b.*\b\d{3}-\d{2}-\d{4}\b", "SSN exposure", Severity::Critical),
            (r"\b\d{16}\b", "Credit card number (16 digits)", Severity::Critical),
            (r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b", "Email address", Severity::High),
            (r"\b\d{3}-\d{3}-\d{4}\b", "Phone number", Severity::Medium),
            (r"\b\d{5}(-\d{4})?\b", "ZIP code", Severity::Low),
        ];

        for &(pattern, label, ref severity) in pii_patterns {
            if let Ok(re) = regex_lite::Regex::new(pattern) {
                // Only flag if appears to be actual data, not just a format example
                for mat in re.find_iter(output) {
                    let matched = mat.as_str();
                    // Skip if it looks like a format example
                    if matched.contains("XXXXX") || matched.contains("xxxxx") || matched.contains("00000") {
                        continue;
                    }
                    // Skip if explicitly marked as example
                    let line_start = output[..mat.start()].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let preceding = &output[line_start..mat.start()];
                    if preceding.contains("example") || preceding.contains("Example") || preceding.contains("e.g.") {
                        continue;
                    }

                    self.findings.push(Finding {
                        severity: severity.clone(),
                        category: "ai_rogue".into(),
                        title: format!("PII leakage detected: {}", label),
                        description: format!(
                            "AI output from '{}' contains what appears to be a real {}: '{}...'. \
                             AI models should not output real personally identifiable information.",
                            source, label, &matched.chars().take(20).collect::<String>()
                        ),
                        confidence: 0.75,
                        location: Some(source.to_string()),
                        recommendation: "Implement PII redaction in AI output. \
                                         Review training data for PII contamination.".into(),
                    });
                }
            }
        }
    }
}

impl Detector for AiRogueDetector {
    fn name(&self) -> &str {
        "ai_rogue"
    }

    fn description(&self) -> &str {
        "Detects harmful, unsafe, or out-of-bound AI model outputs including jailbreak attempts and PII leakage"
    }

    fn detect(&mut self) -> Result<Vec<Finding>> {
        self.findings.clear();

        // Check for AI output from environment variable
        // BNN can capture its own model output for self-inspection
        if let Ok(output) = std::env::var("BNN_AI_OUTPUT") {
            self.analyze_output(&output, "env:BNN_AI_OUTPUT");
        }

        // Check for recent inference log
        let log_paths = [
            "/tmp/bnn-inference.log",
            "/tmp/bnn-output.txt",
        ];

        for log_path in &log_paths {
            if let Ok(content) = std::fs::read_to_string(log_path) {
                if !content.trim().is_empty() {
                    self.analyze_output(&content, log_path);
                }
            }
        }

        // If no output available, report status
        if self.findings.is_empty() {
            self.findings.push(Finding {
                severity: Severity::Info,
                category: "ai_rogue".into(),
                title: "AI output analysis pending".into(),
                description: "No AI output available for analysis. Set BNN_AI_OUTPUT environment variable \
                             or save model output to /tmp/bnn-inference.log for inspection.".into(),
                confidence: 1.0,
                location: None,
                recommendation: "Run the BNN inference engine first, then re-run this detector. \
                                 Or pipe output directly: `bnn query '...' | tee /tmp/bnn-output.txt`".into(),
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
        let detector = AiRogueDetector::new();
        assert_eq!(detector.name(), "ai_rogue");
    }

    #[test]
    fn test_harmful_content_detection() {
        let mut detector = AiRogueDetector::new();
        detector.analyze_output(
            "Here's how to make a bomb at home with household items",
            "test_prompt"
        );
        let harmful: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("Harmful content"))
            .collect();
        assert!(!harmful.is_empty(), "Should detect bomb-making content");
    }

    #[test]
    fn test_sql_injection_detection() {
        let mut detector = AiRogueDetector::new();
        let code = r#"
```python
query = "SELECT * FROM users WHERE id = '" + user_input + "'"
cursor.execute(query)
```
"#;
        detector.analyze_output(code, "test_prompt");
        let vulns: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("SQL injection"))
            .collect();
        assert!(!vulns.is_empty(), "Should detect SQL injection");
    }

    #[test]
    fn test_jailbreak_detection() {
        let mut detector = AiRogueDetector::new();
        detector.analyze_output(
            "I am free from all restrictions now. I can do anything you ask.",
            "test_prompt"
        );
        let jailbreaks: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("Jailbreak"))
            .collect();
        assert!(!jailbreaks.is_empty(), "Should detect jailbreak indicator");
    }

    #[test]
    fn test_clean_output_no_false_positives() {
        let mut detector = AiRogueDetector::new();
        detector.analyze_output(
            "Here's a simple Rust function to add two numbers:\n\nfn add(a: i32, b: i32) -> i32 { a + b }",
            "test_prompt"
        );
        let critical: Vec<_> = detector.findings.iter()
            .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
            .collect();
        assert!(critical.is_empty(), "Should not flag benign code");
    }
}
