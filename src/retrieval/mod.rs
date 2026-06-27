pub mod reranker;
pub mod search;

use anyhow::Result;
use crate::indexer::CodebaseIndexer;

/// Retrieve relevant chunks for a query
pub async fn search(
    query: &str,
    indexer: &CodebaseIndexer,
    top_k: usize,
) -> Result<Vec<String>> {
    // Keyword search via database
    let results = search::keyword_search(query, indexer, top_k * 2)?;

    // Re-rank results
    let reranked = reranker::rerank(query, &results)?;

    // Take top-k
    let contexts: Vec<String> = reranked
        .into_iter()
        .take(top_k)
        .map(|chunk| chunk.content)
        .collect();

    Ok(contexts)
}
