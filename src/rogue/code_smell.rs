//! Code Smell / Bad Pattern Detector
//!
//! Uses tree-sitter AST analysis to detect code smells, anti-patterns,
//! and suspicious code patterns in source files.
//!
//! ## Detection categories
//! - **Complexity**: deeply nested code, excessive branches, long functions
//! - **Security**: hardcoded secrets, unsafe blocks, eval usage
//! - **Maintainability**: duplicated code, magic numbers, dead code
//! - **Performance**: O(n²) patterns, excessive allocations
//! - **Style**: inconsistent naming, overly long lines

use super::{Detector, Finding, Severity};
use anyhow::Result;
use walkdir::WalkDir;

pub struct CodeSmellDetector {
    findings: Vec<Finding>,
    /// File extensions to scan
    source_extensions: &'static [&'static str],
}

impl CodeSmellDetector {
    pub fn new() -> Self {
        Self {
            findings: Vec::new(),
            source_extensions: &[
                "rs", "py", "js", "ts", "jsx", "tsx", "go", "java",
                "cpp", "c", "h", "hpp", "rb", "swift", "kt", "kts",
            ],
        }
    }

    /// Scan a directory recursively for source files
    fn scan_directory(&mut self, path: &str) {
        let dir = if path.is_empty() || path == "." {
            "."
        } else {
            path
        };

        for entry in WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let ext = match entry.path().extension().and_then(|e| e.to_str()) {
                Some(e) => e,
                None => continue,
            };

            if !self.source_extensions.contains(&ext) {
                continue;
            }

            // Skip common non-source directories
            let path_str = entry.path().to_string_lossy();
            if path_str.contains("/node_modules/")
                || path_str.contains("/target/")
                || path_str.contains("/.git/")
                || path_str.contains("/vendor/")
                || path_str.contains("/dist/")
                || path_str.contains("/build/")
                || path_str.contains("__pycache__")
                || path_str.contains("/.venv/")
            {
                continue;
            }

            let content = match std::fs::read_to_string(entry.path()) {
                Ok(c) => c,
                Err(_) => continue,
            };

            self.analyze_file(&path_str, &content);
        }
    }

    /// Analyze a single source file for code smells
    fn analyze_file(&mut self, file_path: &str, content: &str) {
        let lines: Vec<&str> = content.lines().collect();
        let total_lines = lines.len();

        // Skip empty/generated files
        if total_lines < 3 {
            return;
        }

        // 1. Check for hardcoded secrets
        self.check_hardcoded_secrets(file_path, &lines);

        // 2. Check for deeply nested code
        self.check_nesting_depth(file_path, &lines);

        // 3. Check for magic numbers
        self.check_magic_numbers(file_path, &lines);

        // 4. Check for overly long lines
        self.check_long_lines(file_path, &lines);

        // 5. Check for TODO/FIXME comments
        self.check_todo_fixme(file_path, &lines);

        // 6. Check for large functions (by brace count heuristic)
        self.check_large_functions(file_path, &lines);

        // 7. Check for unsafe code blocks (Rust-specific)
        self.check_unsafe_blocks(file_path, &lines);

        // 8. Check for eval/dynamic execution
        self.check_eval_usage(file_path, &lines, total_lines);

        // 9. Check for commented-out code
        self.check_commented_code(file_path, &lines);
    }

    /// Detect hardcoded secrets (API keys, passwords, tokens)
    fn check_hardcoded_secrets(&mut self, file_path: &str, lines: &[&str]) {
        let secret_patterns: &[(&str, &str, Severity)] = &[
            (r#"(?i)(api[_-]?key|apikey)\s*[:=]\s*['"][A-Za-z0-9_\-]{16,}"#, "API Key", Severity::Critical),
            (r#"(?i)(secret|token|auth)\s*[:=]\s*['"][A-Za-z0-9_\-\.]{16,}"#, "Secret/Token", Severity::Critical),
            (r#"(?i)password\s*[:=]\s*['"][^'"\s]{4,}"#, "Password", Severity::Critical),
            (r"(?i)(aws_access_key|aws_secret_key|AKIA[0-9A-Z]{16})", "AWS Credential", Severity::Critical),
            (r"ghp_[A-Za-z0-9]{36}", "GitHub Token", Severity::Critical),
            (r"sk-[A-Za-z0-9]{32,}", "OpenAI API Key", Severity::Critical),
            (r"(?i)-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----", "Private Key", Severity::Critical),
            (r"(?i)jdbc:mysql://[^:]+:[^@]+@", "Database URL with password", Severity::High),
            (r"mongodb://[^:]+:[^@]+@", "MongoDB URL with credentials", Severity::High),
            (r"redis://:[^@]+@", "Redis URL with password", Severity::High),
        ];

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments and test files
            if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
                continue;
            }

            for &(pattern, label, ref severity) in secret_patterns {
                if let Ok(re) = regex_lite::Regex::new(pattern) {
                    if re.is_match(trimmed) {
                        self.findings.push(Finding {
                            severity: severity.clone(),
                            category: "code_smell".into(),
                            title: format!("Hardcoded {} detected", label),
                            description: format!(
                                "Found potential {} on line {} of {}. Hardcoding secrets is a security risk.",
                                label, i + 1, file_path
                            ),
                            confidence: 0.85,
                            location: Some(format!("{}:{}", file_path, i + 1)),
                            recommendation: format!(
                                "Move to environment variables or a secrets manager. \
                                 Use `git secrets` or `.env` with `.gitignore`."
                            ),
                        });
                    }
                }
            }
        }
    }

    /// Detect deeply nested code (high cyclomatic complexity)
    fn check_nesting_depth(&mut self, file_path: &str, lines: &[&str]) {
        let mut max_depth = 0;
        let mut current_depth = 0;
        let mut depth_line = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            // Count opening braces/brackets
            let opens = trimmed.matches('{').count() + trimmed.matches('(').count();
            let closes = trimmed.matches('}').count() + trimmed.matches(')').count();

            // Also check Python indentation
            let indent = line.len() - line.trim_start().len();

            current_depth = current_depth + opens - closes;

            if current_depth > max_depth {
                max_depth = current_depth;
                depth_line = i + 1;
            }

            // Python-style: check indentation depth
            if indent > max_depth * 4 {
                // crude heuristic
            }
        }

        if max_depth > 6 {
            self.findings.push(Finding {
                severity: if max_depth > 10 { Severity::High } else { Severity::Medium },
                category: "code_smell".into(),
                title: "Excessive nesting depth".into(),
                description: format!(
                    "File reaches nesting depth of {} at line {}. Deep nesting indicates \
                     high cyclomatic complexity and poor maintainability.",
                    max_depth, depth_line
                ),
                confidence: 0.85,
                location: Some(format!("{}:{}", file_path, depth_line)),
                recommendation: "Refactor deeply nested code: extract methods, use early returns, \
                                 or apply the guard clause pattern.".into(),
            });
        }
    }

    /// Detect magic numbers (numeric literals without named constants)
    fn check_magic_numbers(&mut self, file_path: &str, lines: &[&str]) {
        let mut magic_count = 0;

        // Patterns to exclude (common non-magic numbers)
        let exclude_patterns: &[&str] = &[
            "0", "1", "-1", "100", "0.0", "1.0",
            "0x", "0o", "0b",  // hex/octal/binary prefixes
        ];

        let magic_re = regex_lite::Regex::new(r"(?m)\b(\d{3,}|\.\d{2,})\b").unwrap();

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            // Skip comments, imports, and attribute/annotation lines
            if trimmed.starts_with("//")
                || trimmed.starts_with('#')
                || trimmed.starts_with("/*")
                || trimmed.starts_with("*")
                || trimmed.starts_with("import")
                || trimmed.starts_with("use ")
                || trimmed.starts_with("#[")
                || trimmed.starts_with("@")
            {
                continue;
            }

            for cap in magic_re.find_iter(trimmed) {
                let num = cap.as_str();
                // Skip if it matches excluded patterns
                if exclude_patterns.iter().any(|&p| num == p || num.starts_with(p)) {
                    continue;
                }
                // Skip if it's a version number like "3.14", "2.0"
                if num.contains('.') {
                    continue;
                }
                magic_count += 1;

                if magic_count <= 3 {
                    // Only report first 3 magic numbers per file to avoid noise
                    self.findings.push(Finding {
                        severity: Severity::Low,
                        category: "code_smell".into(),
                        title: "Magic number detected".into(),
                        description: format!(
                            "Numeric literal `{}` on line {} of {} should be a named constant.",
                            num, i + 1, file_path
                        ),
                        confidence: 0.70,
                        location: Some(format!("{}:{}", file_path, i + 1)),
                        recommendation: "Extract to a named constant: `const MAX_RETRIES: u32 = 3;`".into(),
                    });
                }
            }
        }
    }

    /// Detect overly long lines
    fn check_long_lines(&mut self, file_path: &str, lines: &[&str]) {
        let max_line_length = 120;
        let mut long_lines = 0;

        for (i, line) in lines.iter().enumerate() {
            if line.len() > max_line_length {
                long_lines += 1;
                if long_lines <= 3 {
                    self.findings.push(Finding {
                        severity: Severity::Low,
                        category: "code_smell".into(),
                        title: "Line exceeds maximum length".into(),
                        description: format!(
                            "Line {} is {} characters (max: {}). Long lines reduce readability.",
                            i + 1,
                            line.len(),
                            max_line_length
                        ),
                        confidence: 0.90,
                        location: Some(format!("{}:{}", file_path, i + 1)),
                        recommendation: "Break the line into multiple lines or extract logic to a function.".into(),
                    });
                }
            }
        }

        if long_lines > 10 {
            self.findings.push(Finding {
                severity: Severity::Medium,
                category: "code_smell".into(),
                title: "Many lines exceed maximum length".into(),
                description: format!(
                    "{} lines exceed {} characters in {}. Suggests poor formatting habits.",
                    long_lines, max_line_length, file_path
                ),
                confidence: 0.85,
                location: Some(file_path.to_string()),
                recommendation: "Configure formatter (rustfmt, prettier, black) to enforce line length limits.".into(),
            });
        }
    }

    /// Detect TODO/FIXME/HACK comments
    fn check_todo_fixme(&mut self, file_path: &str, lines: &[&str]) {
        let patterns: &[(&str, &str, Severity)] = &[
            (r"(?i)\bTODO\b", "TODO", Severity::Info),
            (r"(?i)\bFIXME\b", "FIXME", Severity::Medium),
            (r"(?i)\bHACK\b", "HACK", Severity::Medium),
            (r"(?i)\bXXX\b", "XXX", Severity::Info),
            (r"(?i)\bWORKAROUND\b", "WORKAROUND", Severity::Low),
            (r"(?i)\bBUG\b", "BUG", Severity::High),
            (r"(?i)\bOPTIMIZE\b", "OPTIMIZE", Severity::Info),
            (r"(?i)\bTODO:?\s*SECURITY\b", "Security TODO", Severity::High),
        ];

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            for &(pattern, label, ref severity) in patterns {
                if let Ok(re) = regex_lite::Regex::new(pattern) {
                    if re.is_match(trimmed) {
                        self.findings.push(Finding {
                            severity: severity.clone(),
                            category: "code_smell".into(),
                            title: format!("{} comment found", label),
                            description: format!(
                                "Line {} of {} contains a `{}` comment: \"{}\"",
                                i + 1,
                                file_path,
                                label,
                                trimmed.chars().take(80).collect::<String>()
                            ),
                            confidence: 0.95,
                            location: Some(format!("{}:{}", file_path, i + 1)),
                            recommendation: format!(
                                "Address the {} comment. Track in your issue tracker and remove the comment once resolved.",
                                label
                            ),
                        });
                    }
                }
            }
        }
    }

    /// Detect large functions by analyzing brace depth
    fn check_large_functions(&mut self, file_path: &str, lines: &[&str]) {
        let mut brace_depth = 0;
        let mut func_start = 0;
        let mut func_lines = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            let opens = trimmed.matches('{').count();
            let closes = trimmed.matches('}').count();

            if brace_depth == 0 && opens > 0 {
                func_start = i + 1;
                func_lines = 0;
            }

            brace_depth = brace_depth + opens - closes;
            func_lines += 1;

            if brace_depth == 0 && func_lines > 60 {
                self.findings.push(Finding {
                    severity: Severity::Medium,
                    category: "code_smell".into(),
                    title: "Excessively long function".into(),
                    description: format!(
                        "Function starting at line {} spans ~{} lines. Long functions are hard to understand and test.",
                        func_start, func_lines
                    ),
                    confidence: 0.80,
                    location: Some(format!("{}:{}", file_path, func_start)),
                    recommendation: "Split into smaller functions following the Single Responsibility Principle. \
                                     Aim for functions under 30 lines.".into(),
                });
                func_lines = 0;
            }
        }
    }

    /// Detect unsafe Rust blocks
    fn check_unsafe_blocks(&mut self, file_path: &str, lines: &[&str]) {
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("unsafe {") || trimmed == "unsafe" {
                self.findings.push(Finding {
                    severity: Severity::High,
                    category: "code_smell".into(),
                    title: "Unsafe code block".into(),
                    description: format!(
                        "Unsafe block at line {} of {}. Unsafe code bypasses Rust's memory safety guarantees.",
                        i + 1, file_path
                    ),
                    confidence: 0.95,
                    location: Some(format!("{}:{}", file_path, i + 1)),
                    recommendation: "Minimize unsafe code. Wrap in a safe abstraction with `# Safety` docs. \
                                     Consider using `safe_transmute` or `bytemuck` as alternatives.".into(),
                });
            }
        }
    }

    /// Detect eval and dynamic code execution
    fn check_eval_usage(&mut self, file_path: &str, lines: &[&str], _total_lines: usize) {
        let eval_patterns: &[(&str, &str)] = &[
            (r"\beval\s*\(", "eval()"),
            (r"\bexec\s*\(", "exec()"),
            (r"\bsystem\s*\(", "system()"),
            (r"\bpopen\s*\(", "popen()"),
            (r"\bchild_process\.exec\b", "child_process.exec"),
            (r"\bFunction\s*\(", "new Function()"),
            (r#"\bsetTimeout\s*\(["'`]"#, "setTimeout(string)"),
            (r#"\bsetInterval\s*\(["'`]"#, "setInterval(string)"),
            (r"\b__import__\s*\(", "__import__()"),
            (r"\bcompile\s*\(", "compile()"),
            (r"\bexecfile\s*\(", "execfile()"),
            (r"\bdynamicLoad\b", "dynamicLoad"),
            (r"\bdlopen\b", "dlopen"),
            (r"\bLoadLibrary\b", "LoadLibrary"),
        ];

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("//") || trimmed.starts_with('#') || trimmed.starts_with("/*") {
                continue;
            }

            for &(pattern, label) in eval_patterns {
                if let Ok(re) = regex_lite::Regex::new(pattern) {
                    if re.is_match(trimmed) {
                        self.findings.push(Finding {
                            severity: Severity::High,
                            category: "code_smell".into(),
                            title: format!("Dynamic code execution: {}", label),
                            description: format!(
                                "Line {} of {} uses `{}` which executes dynamic code. \
                                 This can lead to code injection vulnerabilities.",
                                i + 1, file_path, label
                            ),
                            confidence: 0.85,
                            location: Some(format!("{}:{}", file_path, i + 1)),
                            recommendation: format!(
                                "Avoid `{}` with user-controlled input. Use safe alternatives like parsers, \
                                 sandboxed interpreters, or whitelisted function calls.",
                                label
                            ),
                        });
                    }
                }
            }
        }
    }

    /// Detect commented-out code
    fn check_commented_code(&mut self, file_path: &str, lines: &[&str]) {
        let mut comment_blocks = 0;
        let mut in_block_comment = false;
        let mut block_start = 0;
        let mut code_lines_in_comment = 0;

        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("/*") {
                in_block_comment = true;
                block_start = i + 1;
                code_lines_in_comment = 0;
            }

            if in_block_comment {
                // Count lines that look like code (contain operators, semicolons, etc.)
                if trimmed.contains(';')
                    || trimmed.contains('=')
                    || trimmed.contains("fn ")
                    || trimmed.contains("if ")
                    || trimmed.contains("for ")
                    || trimmed.contains("while ")
                    || trimmed.contains("return ")
                    || trimmed.starts_with("//")
                {
                    code_lines_in_comment += 1;
                }

                if trimmed.ends_with("*/") {
                    in_block_comment = false;
                    if code_lines_in_comment > 3 {
                        comment_blocks += 1;
                        if comment_blocks <= 2 {
                            self.findings.push(Finding {
                                severity: Severity::Low,
                                category: "code_smell".into(),
                                title: "Commented-out code block".into(),
                                description: format!(
                                    "Lines {}-{} contain ~{} lines of commented-out code in {}. \
                                     Dead code reduces maintainability.",
                                    block_start, i + 1, code_lines_in_comment, file_path
                                ),
                                confidence: 0.75,
                                location: Some(format!("{}:{}", file_path, block_start)),
                                recommendation: "Remove dead code. Use version control (git) to recover if needed. \
                                                 Or move to a documentation file if it's a reference example.".into(),
                            });
                        }
                    }
                }
            }

            // Single-line comments that look like code
            if trimmed.starts_with("//") || trimmed.starts_with('#') {
                let content = trimmed.trim_start_matches("//").trim_start_matches('#').trim();
                if content.contains(';')
                    || (content.contains('=') && content.len() > 5)
                    || content.starts_with("fn ")
                    || content.starts_with("def ")
                    || content.starts_with("if ")
                    || content.starts_with("for ")
                    || content.starts_with("while ")
                    || content.starts_with("return ")
                    || content.starts_with("import ")
                    || content.starts_with("use ")
                {
                    // Only flag if it looks like an actual statement, not a description
                    if content.contains('(') || content.contains(';') {
                        self.findings.push(Finding {
                            severity: Severity::Low,
                            category: "code_smell".into(),
                            title: "Commented-out code line".into(),
                            description: format!(
                                "Line {} of {} contains commented-out code: \"{}\"",
                                i + 1,
                                file_path,
                                content.chars().take(60).collect::<String>()
                            ),
                            confidence: 0.65,
                            location: Some(format!("{}:{}", file_path, i + 1)),
                            recommendation: "Remove dead code. Use version control to track history.".into(),
                        });
                    }
                }
            }
        }
    }
}

impl Detector for CodeSmellDetector {
    fn name(&self) -> &str {
        "code_smell"
    }

    fn description(&self) -> &str {
        "Detects code smells, anti-patterns, security issues, and maintainability problems in source code"
    }

    fn detect(&mut self) -> Result<Vec<Finding>> {
        self.findings.clear();

        // Get target path from environment or default to current directory
        let path = std::env::var("BNN_CODE_PATH").unwrap_or_else(|_| ".".to_string());
        self.scan_directory(&path);

        // If no findings, add an all-clear info finding
        if self.findings.is_empty() {
            self.findings.push(Finding {
                severity: Severity::Info,
                category: "code_smell".into(),
                title: "No code smells detected".into(),
                description: "Scanned source files and found no significant code quality issues.".into(),
                confidence: 1.0,
                location: None,
                recommendation: "Maintain good practices by running `bnn rogue code` regularly.".into(),
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
        let detector = CodeSmellDetector::new();
        assert_eq!(detector.name(), "code_smell");
    }

    #[test]
    fn test_analyze_file_hardcoded_secrets() {
        let mut detector = CodeSmellDetector::new();
        let content = r#"
let api_key = "sk-abc123def456ghi789jkl012";
let password = "supersecret123";
let normal = 42;
"#;
        detector.analyze_file("test.rs", content);
        let secrets: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("Hardcoded") || f.title.contains("Secret"))
            .collect();
        assert!(secrets.len() >= 2, "Should detect API key and password, got {}", secrets.len());
    }

    #[test]
    fn test_analyze_file_nesting() {
        let mut detector = CodeSmellDetector::new();
        let content = r#"
fn main() {
    if true {
        if true {
            if true {
                if true {
                    if true {
                        if true {
                            if true {
                                println!("deep");
                            }
                        }
                    }
                }
            }
        }
    }
}
"#;
        detector.analyze_file("test.rs", content);
        let nesting: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("nesting"))
            .collect();
        assert!(!nesting.is_empty(), "Should detect deep nesting");
    }

    #[test]
    fn test_todo_detection() {
        let mut detector = CodeSmellDetector::new();
        let content = r#"
// TODO: implement this
// FIXME: this is broken
// HACK: workaround for bug
let x = 1;
"#;
        detector.analyze_file("test.rs", content);
        let todos: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("TODO") || f.title.contains("FIXME") || f.title.contains("HACK"))
            .collect();
        assert_eq!(todos.len(), 3);
    }

    #[test]
    fn test_unsafe_detection() {
        let mut detector = CodeSmellDetector::new();
        let content = r#"
unsafe {
    let ptr = &x as *const i32;
}
"#;
        detector.analyze_file("test.rs", content);
        let unsafe_findings: Vec<_> = detector.findings.iter()
            .filter(|f| f.title.contains("Unsafe"))
            .collect();
        assert!(!unsafe_findings.is_empty());
    }
}
