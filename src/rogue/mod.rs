//! Rogue Detection Suite
//!
//! A full-spectrum anomaly detection system for BNN Code.
//! Detects security threats, code quality issues, AI safety concerns,
//! and user behavior anomalies.
//!
//! ## Usage
//! ```
//! use bnn_code::rogue::{RogueEngine, Detector};
//! let mut engine = RogueEngine::new()?;
//! engine.run_all()?;          // run all detectors
//! engine.run_category("security")?; // run one category
//! # Ok::<_, anyhow::Error>(())
//! ```

mod security;
mod code_smell;
mod ai_output;
mod user_behavior;

use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

/// Severity level of a finding
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// A single anomaly finding
#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    /// Severity level
    pub severity: Severity,
    /// Category: "security", "code_smell", "ai_rogue", "user_behavior"
    pub category: String,
    /// Short title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Confidence score 0.0–1.0
    pub confidence: f64,
    /// Location (file path, process name, command, etc.)
    pub location: Option<String>,
    /// Remediation advice
    pub recommendation: String,
}

/// Detector trait — all detectors implement this
pub trait Detector {
    /// Human-readable name
    fn name(&self) -> &str;
    /// Description of what this detector checks
    fn description(&self) -> &str;
    /// Run detection and return findings
    fn detect(&mut self) -> Result<Vec<Finding>>;
}

/// Summary report from all detectors
#[derive(Debug, Serialize)]
pub struct RogueReport {
    pub total_findings: usize,
    pub by_severity: HashMap<String, usize>,
    pub by_category: HashMap<String, usize>,
    pub findings: Vec<Finding>,
}

/// Main engine that orchestrates all detectors
pub struct RogueEngine {
    detectors: Vec<Box<dyn Detector>>,
}

impl RogueEngine {
    /// Create a new engine with all detectors registered
    pub fn new() -> Self {
        let detectors: Vec<Box<dyn Detector>> = vec![
            Box::new(security::SecurityDetector::new()),
            Box::new(code_smell::CodeSmellDetector::new()),
            Box::new(ai_output::AiRogueDetector::new()),
            Box::new(user_behavior::UserBehaviorDetector::new()),
        ];
        Self { detectors }
    }

    /// Run all detectors and return combined report
    pub fn run_all(&mut self) -> Result<RogueReport> {
        let mut all_findings = Vec::new();
        for detector in &mut self.detectors {
            let findings = detector.detect()?;
            all_findings.extend(findings);
        }
        Ok(Self::build_report(all_findings))
    }

    /// Run only detectors matching the given category (supports aliases)
    pub fn run_category(&mut self, category: &str) -> Result<RogueReport> {
        let normalized = category.to_lowercase().replace('-', "_");
        let mut all_findings = Vec::new();
        for detector in &mut self.detectors {
            let cat = detector.name();
            let matches = match (cat, normalized.as_str()) {
                ("security", "security") | (_, "all") => true,
                ("code_smell", "code_smell" | "code" | "smell") => true,
                ("ai_rogue", "ai_rogue" | "ai" | "rogue") => true,
                ("user_behavior", "user_behavior" | "user" | "behavior") => true,
                _ => false,
            };
            if !matches {
                continue;
            }
            let findings = detector.detect()?;
            all_findings.extend(findings);
        }
        Ok(Self::build_report(all_findings))
    }

    /// Build aggregated report from findings
    fn build_report(findings: Vec<Finding>) -> RogueReport {
        let total_findings = findings.len();
        let mut by_severity: HashMap<String, usize> = HashMap::new();
        let mut by_category: HashMap<String, usize> = HashMap::new();

        for f in &findings {
            *by_severity.entry(f.severity.to_string()).or_insert(0) += 1;
            *by_category.entry(f.category.clone()).or_insert(0) += 1;
        }

        RogueReport {
            total_findings,
            by_severity,
            by_category,
            findings,
        }
    }
}

impl Default for RogueEngine {
    fn default() -> Self {
        Self::new()
    }
}

/// Format report as a human-readable terminal string
pub fn format_report(report: &RogueReport, verbose: bool) -> String {
    use std::fmt::Write;

    let mut out = String::new();

    writeln!(out, "{}", "═".repeat(60)).ok();
    writeln!(out, "  🔍 ROGUE DETECTION REPORT").ok();
    writeln!(out, "{}", "═".repeat(60)).ok();
    writeln!(
        out,
        "  Total findings: {}",
        report.total_findings
    )
    .ok();
    writeln!(out).ok();

    // Summary by severity
    if !report.by_severity.is_empty() {
        writeln!(out, "  ── By Severity ──").ok();
        let mut sevs: Vec<_> = report.by_severity.iter().collect();
        sevs.sort_by(|a, b| {
            let order = |s: &str| match s {
                "CRITICAL" => 0,
                "HIGH" => 1,
                "MEDIUM" => 2,
                "LOW" => 3,
                "INFO" => 4,
                _ => 5,
            };
            order(a.0).cmp(&order(b.0))
        });
        for (sev, count) in &sevs {
            let icon = match sev.as_str() {
                "CRITICAL" => "🔴",
                "HIGH" => "🟠",
                "MEDIUM" => "🟡",
                "LOW" => "🔵",
                _ => "⚪",
            };
            writeln!(out, "    {} {}: {}", icon, sev, count).ok();
        }
        writeln!(out).ok();
    }

    // Summary by category
    if !report.by_category.is_empty() {
        writeln!(out, "  ── By Category ──").ok();
        for (cat, count) in &report.by_category {
            let label = match cat.as_str() {
                "security" => "Security",
                "code_smell" => "Code Smell",
                "ai_rogue" => "AI Rogue Output",
                "user_behavior" => "User Behavior",
                _ => cat,
            };
            writeln!(out, "    • {}: {}", label, count).ok();
        }
        writeln!(out).ok();
    }

    // Detailed findings
    if verbose && !report.findings.is_empty() {
        writeln!(out, "  ── Details ──").ok();
        for (i, f) in report.findings.iter().enumerate() {
            let sev_icon = match f.severity {
                Severity::Critical => "🔴",
                Severity::High => "🟠",
                Severity::Medium => "🟡",
                Severity::Low => "🔵",
                Severity::Info => "⚪",
            };
            writeln!(out).ok();
            writeln!(
                out,
                "  {}. {} [{}] {}",
                i + 1,
                sev_icon,
                f.severity,
                f.title
            )
            .ok();
            writeln!(out, "     {}", f.description).ok();
            if let Some(ref loc) = f.location {
                writeln!(out, "     Location: {}", loc).ok();
            }
            writeln!(
                out,
                "     Confidence: {:.0}%",
                f.confidence * 100.0
            )
            .ok();
            writeln!(out, "     💡 {}", f.recommendation).ok();
        }
    }

    writeln!(out, "{}", "═".repeat(60)).ok();

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_runs_all() {
        let mut engine = RogueEngine::new();
        let report = engine.run_all().unwrap();
        assert!(report.total_findings > 0);
    }

    #[test]
    fn test_engine_runs_category() {
        let mut engine = RogueEngine::new();
        let report = engine.run_category("security").unwrap();
        for f in &report.findings {
            assert_eq!(f.category, "security");
        }
    }

    #[test]
    fn test_format_report() {
        let mut engine = RogueEngine::new();
        let report = engine.run_all().unwrap();
        let formatted = format_report(&report, true);
        assert!(!formatted.is_empty());
        assert!(formatted.contains("ROGUE DETECTION REPORT"));
    }

    #[test]
    fn test_finding_serialization() {
        let f = Finding {
            severity: Severity::High,
            category: "security".into(),
            title: "Test finding".into(),
            description: "A test".into(),
            confidence: 0.95,
            location: Some("/tmp/test".into()),
            recommendation: "Fix it".into(),
        };
        let json = serde_json::to_string(&f).unwrap();
        assert!(json.contains(r#""High""#), "JSON should contain 'High' severity, got: {}", json);
        assert!(json.contains("security"), "JSON should contain 'security', got: {}", json);
    }
}
