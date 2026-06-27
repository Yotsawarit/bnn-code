use anyhow::Result;

use crate::indexer::chunker::CodeChunk;
use crate::indexer::CodebaseIndexer;

/// Simple keyword-based search using SQLite LIKE queries
/// In production, replace with BM25 + vector search
pub fn keyword_search(
    query: &str,
    _indexer: &CodebaseIndexer,
    top_k: usize,
) -> Result<Vec<CodeChunk>> {
    // For now, return placeholder results
    // This would use the database's search_by_keyword method
    let keywords: Vec<&str> = query.split_whitespace().collect();
    tracing::debug!("Searching for keywords: {:?}", keywords);

    // Placeholder: simulate retrieval
    let results: Vec<CodeChunk> = (0..top_k)
        .map(|i| CodeChunk {
            content: format!(
                "// Retrieved chunk {} for query: {}\n// Keywords: {:?}",
                i + 1,
                query,
                keywords
            ),
            start_line: 0,
            end_line: 5,
            symbol_name: Some(format!("retrieved_{}", i)),
            chunk_type: crate::indexer::chunker::ChunkType::Module,
        })
        .collect();

    Ok(results.into_iter().take(top_k).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keyword_search_basic() {
        // This tests the current placeholder implementation
        let query = "calculate sum";
        let top_k = 3;

        // We need an indexer — create a minimal one for testing
        let indexer = crate::indexer::CodebaseIndexer::new(".").unwrap();
        let results = keyword_search(query, &indexer, top_k).unwrap();

        // Placeholder returns top_k results with mock content
        assert_eq!(results.len(), top_k);
        for result in &results {
            assert!(result.content.contains(query));
        }
    }

    #[test]
    fn test_keyword_search_top_k() {
        let query = "test query";
        let indexer = crate::indexer::CodebaseIndexer::new(".").unwrap();
        let results = keyword_search(query, &indexer, 5).unwrap();
        assert_eq!(results.len(), 5);

        let results_0 = keyword_search(query, &indexer, 0).unwrap();
        assert!(results_0.is_empty());
    }

    #[test]
    fn test_keyword_search_keywords_parsed() {
        let query = "multiple word query here";
        let indexer = crate::indexer::CodebaseIndexer::new(".").unwrap();
        let results = keyword_search(query, &indexer, 1).unwrap();
        assert_eq!(results.len(), 1);
        // Placeholder includes keywords in output
        assert!(results[0].content.contains("multiple"));
    }
}
