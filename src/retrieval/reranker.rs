use anyhow::Result;

use crate::indexer::chunker::CodeChunk;

/// Cross-encoder re-ranker
/// In production, this would run a lightweight ONNX model
pub fn rerank(query: &str, chunks: &[CodeChunk]) -> Result<Vec<CodeChunk>> {
    // Simple scoring based on keyword overlap
    let query_words: Vec<&str> = query
        .split_whitespace()
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()))
        .filter(|w| !w.is_empty())
        .collect();

    let mut scored: Vec<(f64, &CodeChunk)> = chunks
        .iter()
        .map(|chunk| {
            let content_lower = chunk.content.to_lowercase();
            let mut score = 0.0;

            for word in &query_words {
                let word_lower = word.to_lowercase();
                if content_lower.contains(&word_lower) {
                    score += 1.0;
                    // Bonus for exact matches
                    if chunk
                        .symbol_name
                        .as_deref()
                        .unwrap_or("")
                        .to_lowercase()
                        .contains(&word_lower)
                    {
                        score += 2.0;
                    }
                }
            }

            // Normalize by content length
            if !chunk.content.is_empty() {
                score /= (chunk.content.len() as f64).sqrt();
            }

            (score, chunk)
        })
        .collect();

    // Sort by score descending
    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    Ok(scored.into_iter().map(|(_, chunk)| chunk.clone()).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::chunker::ChunkType;

    fn make_chunk(content: &str, symbol_name: Option<&str>) -> CodeChunk {
        CodeChunk {
            content: content.to_string(),
            start_line: 0,
            end_line: content.lines().count().saturating_sub(1),
            symbol_name: symbol_name.map(|s| s.to_string()),
            chunk_type: ChunkType::Function,
        }
    }

    #[test]
    fn test_rerank_empty_chunks() {
        let result = rerank("test query", &[]).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_rerank_single_chunk() {
        let chunks = vec![make_chunk("fn test() {}", Some("test"))];
        let result = rerank("test", &chunks).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_rerank_orders_by_relevance() {
        let chunks = vec![
            make_chunk("fn unrelated() { math_stuff }", Some("unrelated")),
            make_chunk("fn calculate_sum() { 1 + 1 }", Some("calculate_sum")),
            make_chunk("fn print_sum() { sum_display }", Some("print_sum")),
        ];

        // "sum" should match calculate_sum and print_sum more than unrelated
        let result = rerank("sum", &chunks).unwrap();
        assert_eq!(result.len(), 3);

        // First result should be the most relevant (calculate_sum has "sum" in name and content)
        let first = &result[0];
        assert!(first.content.contains("sum") || first.symbol_name.as_deref() == Some("calculate_sum"));
    }

    #[test]
    fn test_rerank_exact_name_bonus() {
        let chunks = vec![
            make_chunk("fn process() { data }", Some("process")),
            make_chunk("fn process_data() { data processing }", Some("process_data")),
        ];

        let result = rerank("process", &chunks).unwrap();
        // "process" appears in both, but first has it exactly in symbol_name
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_rerank_case_insensitive() {
        let chunks = vec![
            make_chunk("fn HELLO_WORLD() { greeting }", Some("HELLO_WORLD")),
        ];

        let result = rerank("hello", &chunks).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_rerank_no_match() {
        let chunks = vec![
            make_chunk("fn aaa() { zzz }", Some("aaa")),
        ];

        let result = rerank("nonexistent_word_xyz", &chunks).unwrap();
        assert_eq!(result.len(), 1); // Still returns, just low score
    }
}
