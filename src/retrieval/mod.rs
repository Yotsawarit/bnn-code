pub mod reranker;
pub mod search;

use anyhow::Result;

/// Retrieve relevant chunks for a query using FTS5 full-text search.
/// Opens the persistent database written by `bnn-code index`.
pub async fn search(query: &str, top_k: usize) -> Result<Vec<String>> {
    let results = search::keyword_search(query, top_k * 2)?;
    let reranked = reranker::rerank(query, &results)?;
    let contexts: Vec<String> = reranked
        .into_iter()
        .take(top_k)
        .map(|chunk| chunk.content)
        .collect();
    Ok(contexts)
}
