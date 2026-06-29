//! Integration tests for bnn-code
//!
//! These tests verify the full pipeline:
//!   1. Code parsing → chunking → database storage
//!   2. Search → reranking
//!   3. Configuration init/load
//!
//! NOTE: Tests requiring an actual ONNX model are excluded.
//!       Run them manually with: cargo test -- --ignored

use std::process::Command;

// ============================================================
//  Smoke tests — verify the binary can start
// ============================================================

#[test]
fn test_binary_help() {
    let output = Command::new(env!("CARGO_BIN_EXE_bnn-code"))
        .arg("--help")
        .output()
        .expect("Failed to run bnn-code --help");

    assert!(output.status.success());

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("bnn"));
    assert!(stdout.contains("explain"));
    assert!(stdout.contains("refactor"));
    assert!(stdout.contains("test"));
    assert!(stdout.contains("init"));
}

#[test]
fn test_binary_version() {
    let output = Command::new(env!("CARGO_BIN_EXE_bnn-code"))
        .arg("--version")
        .output()
        .expect("Failed to run bnn-code --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0.1.0"));
}

// ============================================================
//  Parser integration — parse real source files
// ============================================================

#[test]
fn test_parser_rust_source() {
    use bnn_code::indexer::parser::CodeParser;

    let parser = CodeParser::new().unwrap();
    let source = r#"
/// Adds two numbers
fn add(a: i32, b: i32) -> i32 {
    a + b
}

/// A point in 2D space
struct Point {
    x: f64,
    y: f64,
}

impl Point {
    /// Creates a new point
    fn new(x: f64, y: f64) -> Self {
        Point { x, y }
    }
}
"#;

    let symbols = parser.extract_symbols(source).unwrap();
    assert_eq!(symbols.len(), 3, "Should find 3 symbols: add, Point, new");

    let add = symbols.iter().find(|s| s.name == "add").unwrap();
    assert_eq!(add.doc_comment.as_deref(), Some("Adds two numbers"));

    let point = symbols.iter().find(|s| s.name == "Point").unwrap();
    assert_eq!(point.kind, bnn_code::indexer::parser::SymbolKind::Struct);
}

#[test]
fn test_parser_multiple_languages() {
    use bnn_code::indexer::parser::CodeParser;

    let parser = CodeParser::new().unwrap();

    // Python
    let py_symbols = parser.extract_symbols("def hello():\n    pass").unwrap();
    assert!(py_symbols.iter().any(|s| s.name == "hello"));

    // JavaScript
    let js_symbols = parser.extract_symbols("function greet() { return 'hi'; }").unwrap();
    assert!(js_symbols.iter().any(|s| s.name == "greet"));

    // Go
    let go_symbols = parser.extract_symbols("func main() {}").unwrap();
    assert!(go_symbols.iter().any(|s| s.name == "main"));
}

// ============================================================
//  Chunker integration — parse + chunk pipeline
// ============================================================

#[test]
fn test_parse_then_chunk() {
    use bnn_code::indexer::chunker::SemanticChunker;
    use bnn_code::indexer::parser::CodeParser;

    let parser = CodeParser::new().unwrap();
    let chunker = SemanticChunker::new(50);

    let source = "fn a() {}\nfn b() {}\nfn c() {}";
    let symbols = parser.extract_symbols(source).unwrap();
    let chunks = chunker.chunk(source, &symbols).unwrap();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].symbol_name.as_deref(), Some("a"));
    assert_eq!(chunks[1].symbol_name.as_deref(), Some("b"));
    assert_eq!(chunks[2].symbol_name.as_deref(), Some("c"));
}

// ============================================================
//  Database integration — full store + search
// ============================================================

#[test]
fn test_database_full_pipeline() {
    use bnn_code::indexer::chunker::{CodeChunk, ChunkType};
    use bnn_code::indexer::database::CodeDatabase;

    let db = CodeDatabase::open_in_memory().unwrap();

    // Store chunks
    let chunks = vec![
        CodeChunk {
            content: "fn calculate() { 1 + 1 }".to_string(),
            start_line: 0,
            end_line: 2,
            symbol_name: Some("calculate".to_string()),
            chunk_type: ChunkType::Function,
        },
        CodeChunk {
            content: "fn display() { println!(\"done\") }".to_string(),
            start_line: 4,
            end_line: 6,
            symbol_name: Some("display".to_string()),
            chunk_type: ChunkType::Function,
        },
    ];

    db.store_chunks("test.rs", "full source", &chunks).unwrap();
    assert_eq!(db.chunk_count().unwrap(), 2);

    // Search
    let results = db.search_by_keyword("calculate", 20).unwrap();
    assert_eq!(results.len(), 1);

    // Get file chunks
    let file_chunks = db.get_file_chunks("test.rs").unwrap();
    assert_eq!(file_chunks.len(), 2);
}

// ============================================================
//  Reranker integration
// ============================================================

#[test]
fn test_reranker_orders_results() {
    use bnn_code::indexer::chunker::{CodeChunk, ChunkType};
    use bnn_code::retrieval::reranker::rerank;

    let chunks = vec![
        CodeChunk {
            content: "def sort(arr): return sorted(arr)".to_string(),
            start_line: 0, end_line: 0,
            symbol_name: Some("sort".to_string()),
            chunk_type: ChunkType::Function,
        },
        CodeChunk {
            content: "def search(arr, target): return -1".to_string(),
            start_line: 0, end_line: 0,
            symbol_name: Some("search".to_string()),
            chunk_type: ChunkType::Function,
        },
        CodeChunk {
            content: "def filter(arr): return arr".to_string(),
            start_line: 0, end_line: 0,
            symbol_name: Some("filter".to_string()),
            chunk_type: ChunkType::Function,
        },
    ];

    let results = rerank("sort filter", &chunks).unwrap();
    assert_eq!(results.len(), 3);

    // sort and filter should be ahead of search
    let first_two: Vec<&str> = results[..2]
        .iter()
        .map(|c| c.symbol_name.as_deref().unwrap())
        .collect();
    assert!(first_two.contains(&"sort"), "sort should be in top 2");
    assert!(first_two.contains(&"filter"), "filter should be in top 2");
}

// ============================================================
//  Config integration
// ============================================================

#[test]
fn test_config_default_roundtrip() {
    use bnn_code::utils::config::BnnConfig;

    let config = BnnConfig::default();
    let json = serde_json::to_string_pretty(&config).unwrap();
    let deserialized: BnnConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.model.max_tokens, deserialized.model.max_tokens);
    assert_eq!(config.indexing.max_file_size_kb, deserialized.indexing.max_file_size_kb);
    assert_eq!(config.ui.theme, deserialized.ui.theme);
}

// ============================================================
//  End-to-end: parse → chunk → database → search → rerank
// ============================================================

#[test]
fn test_end_to_end_pipeline() {
    use bnn_code::indexer::chunker::SemanticChunker;
    use bnn_code::indexer::database::CodeDatabase;
    use bnn_code::indexer::parser::CodeParser;
    use bnn_code::retrieval::reranker::rerank;

    // Step 1: Parse
    let parser = CodeParser::new().unwrap();
    let source = r#"
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

fn calculate_sum(a: i32, b: i32) -> i32 {
    a + b
}
"#;
    let symbols = parser.extract_symbols(source).unwrap();
    assert_eq!(symbols.len(), 2);

    // Step 2: Chunk
    let chunker = SemanticChunker::new(50);
    let chunks = chunker.chunk(source, &symbols).unwrap();
    assert_eq!(chunks.len(), 2);

    // Step 3: Store
    let db = CodeDatabase::open_in_memory().unwrap();
    db.store_chunks("lib.rs", source, &chunks).unwrap();
    assert_eq!(db.chunk_count().unwrap(), 2);

    // Step 4: Search
    let db_results = db.search_by_keyword("calculate", 20).unwrap();
    assert_eq!(db_results.len(), 1);
    assert_eq!(db_results[0].symbol_name.as_deref(), Some("calculate_sum"));

    // Step 5: Rerank
    let all_chunks = db.get_file_chunks("lib.rs").unwrap();
    let reranked = rerank("greet", &all_chunks).unwrap();
    assert_eq!(reranked.len(), 2);
    // First result should be the "greet" function
    if let Some(first) = reranked.first() {
        assert_eq!(first.symbol_name.as_deref(), Some("greet"));
    }
}

// ============================================================
//  Ignored tests (require ONNX model on disk)
// ============================================================

#[ignore]
#[test]
fn test_onnx_engine_load_model() {
    use bnn_code::inference::onnx::OnnxEngine;
    use std::path::Path;

    let engine = OnnxEngine::new(Path::new("models/model.onnx")).unwrap();
    // If this runs, model loaded successfully
}
