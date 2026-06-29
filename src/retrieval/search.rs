use anyhow::Result;
use crate::indexer::chunker::CodeChunk;
use crate::indexer::database::CodeDatabase;

/// Search the indexed codebase using FTS5 full-text search.
///
/// Opens the persistent database written by `bnn-code index`.
/// Returns up to `top_k` chunks ranked by relevance.
pub fn keyword_search(query: &str, top_k: usize) -> Result<Vec<CodeChunk>> {
    tracing::debug!("FTS search: {:?} (top_k={})", query, top_k);
    let db = CodeDatabase::open_default()?;
    let results = db.search_by_keyword(query, top_k)?;
    tracing::debug!("Found {} results", results.len());
    Ok(results)
}

/// Search using an already-open database (useful for testing and reuse).
pub fn keyword_search_with_db(
    query: &str,
    top_k: usize,
    db: &CodeDatabase,
) -> Result<Vec<CodeChunk>> {
    tracing::debug!("FTS search (with_db): {:?} (top_k={})", query, top_k);
    db.search_by_keyword(query, top_k)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::chunker::{ChunkType, CodeChunk};
    use crate::indexer::database::CodeDatabase;

    fn make_chunk(content: &str, name: &str) -> CodeChunk {
        CodeChunk {
            content: content.to_string(),
            start_line: 0,
            end_line: 5,
            symbol_name: Some(name.to_string()),
            chunk_type: ChunkType::Function,
        }
    }

    fn seeded_db() -> CodeDatabase {
        let db = CodeDatabase::open_in_memory().unwrap();
        db.store_chunks("math.rs", "h1", &[
            make_chunk("fn calculate_sum(a: i32, b: i32) -> i32 { a + b }", "calculate_sum"),
            make_chunk("fn print_result() { println!(\"done\") }", "print_result"),
        ]).unwrap();
        db.store_chunks("auth.rs", "h2", &[
            make_chunk("fn authenticate(token: &str) -> bool { !token.is_empty() }", "authenticate"),
        ]).unwrap();
        db
    }

    #[test]
    fn test_search_returns_matching_chunks() {
        let db = seeded_db();
        let results = keyword_search_with_db("calculate_sum", 10, &db).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol_name.as_deref(), Some("calculate_sum"));
    }

    #[test]
    fn test_search_no_results_for_unknown_term() {
        let db = seeded_db();
        let results = keyword_search_with_db("zzz_not_found", 10, &db).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_top_k_limits_results() {
        let db = CodeDatabase::open_in_memory().unwrap();
        // Insert 10 chunks all matching "fn handler"
        let chunks: Vec<CodeChunk> = (0..10).map(|i| CodeChunk {
            content: format!("fn handler_{}() {{}}", i),
            start_line: i,
            end_line: i + 2,
            symbol_name: Some(format!("handler_{}", i)),
            chunk_type: ChunkType::Function,
        }).collect();
        db.store_chunks("handlers.rs", "h1", &chunks).unwrap();

        let results = keyword_search_with_db("handler", 3, &db).unwrap();
        assert!(results.len() <= 3);
    }

    #[test]
    fn test_search_empty_db_returns_empty() {
        let db = CodeDatabase::open_in_memory().unwrap();
        let results = keyword_search_with_db("anything", 10, &db).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_search_across_multiple_files() {
        let db = seeded_db();
        // "fn" appears in all chunks
        let results = keyword_search_with_db("authenticate", 10, &db).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol_name.as_deref(), Some("authenticate"));
    }
}
