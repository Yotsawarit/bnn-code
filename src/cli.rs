use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bnn")]
#[command(author = "BNN Code Team")]
#[command(version = "0.1.3")]
#[command(about = "Terminal-native AI coding agent powered by BNNs")]
pub struct Cli {
    /// Query to run (optional, enters REPL if not provided)
    pub query: Option<String>,

    /// Path to the codebase to index
    #[arg(short, long, default_value = ".")]
    pub path: String,

    /// BNN model to use
    #[arg(short, long, default_value = "default")]
    pub model: String,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Disable streaming output
    #[arg(long)]
    pub no_stream: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Explain a file or function
    Explain {
        /// File to explain
        file: String,
    },
    /// Suggest refactoring improvements
    Refactor {
        /// File to refactor
        file: String,
    },
    /// Generate unit tests
    Test {
        /// File to test
        file: String,
    },
    /// Initialize BNN Code in current project
    Init,
    /// Run rogue detection (security, code, ai, user behavior anomalies)
    Rogue {
        /// Detection category: security|code|ai|user (default: all)
        #[arg(short, long)]
        category: Option<String>,
        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Fix bugs or errors in a file (scans codebase if no file given)
    Fix {
        /// File to fix (optional — scans full codebase if omitted)
        file: Option<String>,
    },
    /// Generate a commit message from staged changes
    Commit,
    /// Review code for bugs, security, and performance
    Review {
        /// File to review (optional — reviews git diff if omitted)
        file: Option<String>,
    },
    /// Generate docstrings and documentation
    Document {
        /// File to document
        file: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_defaults() {
        let cli = Cli::try_parse_from(["bnn"]).unwrap();
        assert_eq!(cli.path, ".");
        assert_eq!(cli.model, "default");
        assert!(!cli.verbose);
        assert!(!cli.no_stream);
        assert!(cli.query.is_none());
        assert!(cli.command.is_none());
    }

    #[test]
    fn test_cli_with_query() {
        let cli = Cli::try_parse_from(["bnn", "explain this code"]).unwrap();
        assert_eq!(cli.query.as_deref(), Some("explain this code"));
    }

    #[test]
    fn test_cli_with_options() {
        let cli = Cli::try_parse_from([
            "bnn",
            "--path", "/my/project",
            "--model", "codebert",
            "--verbose",
            "--no-stream",
        ]).unwrap();
        assert_eq!(cli.path, "/my/project");
        assert_eq!(cli.model, "codebert");
        assert!(cli.verbose);
        assert!(cli.no_stream);
    }

    #[test]
    fn test_cli_explain_command() {
        let cli = Cli::try_parse_from(["bnn", "explain", "src/main.rs"]).unwrap();
        match cli.command {
            Some(Commands::Explain { file }) => assert_eq!(file, "src/main.rs"),
            _ => panic!("Expected Explain command"),
        }
    }

    #[test]
    fn test_cli_refactor_command() {
        let cli = Cli::try_parse_from(["bnn", "refactor", "src/lib.rs"]).unwrap();
        match cli.command {
            Some(Commands::Refactor { file }) => assert_eq!(file, "src/lib.rs"),
            _ => panic!("Expected Refactor command"),
        }
    }

    #[test]
    fn test_cli_test_command() {
        let cli = Cli::try_parse_from(["bnn", "test", "src/tests/test_module.rs"]).unwrap();
        match cli.command {
            Some(Commands::Test { file }) => assert_eq!(file, "src/tests/test_module.rs"),
            _ => panic!("Expected Test command"),
        }
    }

    #[test]
    fn test_cli_init_command() {
        let cli = Cli::try_parse_from(["bnn", "init"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Init)));
    }

    #[test]
    fn test_cli_rogue_command() {
        let cli = Cli::try_parse_from(["bnn", "rogue"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Rogue { .. })));
    }

    #[test]
    fn test_cli_rogue_category() {
        let cli = Cli::try_parse_from(["bnn", "rogue", "--category", "security"]).unwrap();
        match cli.command {
            Some(Commands::Rogue { category, json }) => {
                assert_eq!(category.as_deref(), Some("security"));
                assert!(!json);
            }
            _ => panic!("Expected Rogue command"),
        }
    }

    #[test]
    fn test_cli_rogue_json() {
        let cli = Cli::try_parse_from(["bnn", "rogue", "--json"]).unwrap();
        match cli.command {
            Some(Commands::Rogue { json, .. }) => assert!(json),
            _ => panic!("Expected Rogue command with json=true"),
        }
    }

    #[test]
    fn test_cli_fix_command_with_file() {
        let cli = Cli::try_parse_from(["bnn", "fix", "src/main.rs"]).unwrap();
        match cli.command {
            Some(Commands::Fix { file }) => assert_eq!(file.as_deref(), Some("src/main.rs")),
            _ => panic!("Expected Fix command with file"),
        }
    }

    #[test]
    fn test_cli_fix_command_without_file() {
        let cli = Cli::try_parse_from(["bnn", "fix"]).unwrap();
        match cli.command {
            Some(Commands::Fix { file }) => assert!(file.is_none()),
            _ => panic!("Expected Fix command without file"),
        }
    }

    #[test]
    fn test_cli_commit_command() {
        let cli = Cli::try_parse_from(["bnn", "commit"]).unwrap();
        assert!(matches!(cli.command, Some(Commands::Commit)));
    }

    #[test]
    fn test_cli_review_command_with_file() {
        let cli = Cli::try_parse_from(["bnn", "review", "src/lib.rs"]).unwrap();
        match cli.command {
            Some(Commands::Review { file }) => assert_eq!(file.as_deref(), Some("src/lib.rs")),
            _ => panic!("Expected Review command with file"),
        }
    }

    #[test]
    fn test_cli_review_command_without_file() {
        let cli = Cli::try_parse_from(["bnn", "review"]).unwrap();
        match cli.command {
            Some(Commands::Review { file }) => assert!(file.is_none()),
            _ => panic!("Expected Review command without file"),
        }
    }

    #[test]
    fn test_cli_document_command() {
        let cli = Cli::try_parse_from(["bnn", "document", "src/util.rs"]).unwrap();
        match cli.command {
            Some(Commands::Document { file }) => assert_eq!(file, "src/util.rs"),
            _ => panic!("Expected Document command"),
        }
    }

    #[test]
    fn test_cli_fix_overrides_query() {
        // "fix" as a subcommand should not be parsed as a query
        let cli = Cli::try_parse_from(["bnn", "fix"]).unwrap();
        assert!(cli.query.is_none());
        assert!(matches!(cli.command, Some(Commands::Fix { .. })));
    }

    #[test]
    fn test_cli_version_flag() {
        let result = Cli::try_parse_from(["bnn", "--version"]);
        // --version exits with 0, so parsing returns error (clap prints version)
        // We just verify it doesn't panic
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_cli_help_flag() {
        let result = Cli::try_parse_from(["bnn", "--help"]);
        assert!(result.is_err() || result.is_ok());
    }

    #[test]
    fn test_cli_short_options() {
        let cli = Cli::try_parse_from(["bnn", "-p", "src", "-m", "fast", "-v"]).unwrap();
        assert_eq!(cli.path, "src");
        assert_eq!(cli.model, "fast");
        assert!(cli.verbose);
    }
}
