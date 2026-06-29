use anyhow::Result;

use super::parser::CodeSymbol;

/// Represents a semantic chunk of code
#[derive(Debug, Clone)]
pub struct CodeChunk {
    pub content: String,
    pub start_line: usize,
    pub end_line: usize,
    pub symbol_name: Option<String>,
    pub chunk_type: ChunkType,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
#[allow(dead_code)]
#[allow(dead_code)]
pub enum ChunkType {
    Function,
    Class,
    Module,
    Comment,
}

/// Splits code into semantically meaningful chunks
pub struct SemanticChunker {
    max_lines: usize,
}

impl SemanticChunker {
    pub fn new(max_lines: usize) -> Self {
        Self { max_lines }
    }

    pub fn chunk(
        &self,
        source: &str,
        symbols: &[CodeSymbol],
    ) -> Result<Vec<CodeChunk>> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = source.lines().collect();

        // Create chunks for each symbol
        for symbol in symbols {
            let end_line = symbol.end_line.min(lines.len().saturating_sub(1));
            let content = lines[symbol.start_line..=end_line].join("\n");

            let chunk_type = match symbol.kind {
                super::parser::SymbolKind::Function | super::parser::SymbolKind::Method => {
                    ChunkType::Function
                }
                super::parser::SymbolKind::Class
                | super::parser::SymbolKind::Struct
                | super::parser::SymbolKind::Interface
                | super::parser::SymbolKind::Enum => ChunkType::Class,
            };

            chunks.push(CodeChunk {
                content,
                start_line: symbol.start_line,
                end_line,
                symbol_name: Some(symbol.name.clone()),
                chunk_type,
            });
        }

        // If no symbols found, chunk by line count
        if chunks.is_empty() {
            for (i, chunk_lines) in lines.chunks(self.max_lines).enumerate() {
                let content = chunk_lines.join("\n");
                chunks.push(CodeChunk {
                    content,
                    start_line: i * self.max_lines,
                    end_line: (i + 1) * self.max_lines - 1,
                    symbol_name: None,
                    chunk_type: ChunkType::Module,
                });
            }
        }

        Ok(chunks)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::parser::SymbolKind;

    fn make_symbol(name: &str, kind: SymbolKind, start: usize, end: usize) -> CodeSymbol {
        CodeSymbol {
            name: name.to_string(),
            kind,
            start_line: start,
            end_line: end,
            doc_comment: None,
        }
    }

    #[test]
    fn test_chunk_with_symbols() {
        let chunker = SemanticChunker::new(50);
        let source = "fn hello() {\n    println!(\"Hello\");\n}\n\nfn world() {\n    println!(\"World\");\n}";
        let symbols = vec![
            make_symbol("hello", SymbolKind::Function, 0, 2),
            make_symbol("world", SymbolKind::Function, 4, 6),
        ];

        let chunks = chunker.chunk(source, &symbols).unwrap();
        assert_eq!(chunks.len(), 2);

        assert_eq!(chunks[0].symbol_name.as_deref(), Some("hello"));
        assert_eq!(chunks[0].start_line, 0);
        assert_eq!(chunks[0].end_line, 2);
        assert_eq!(chunks[0].chunk_type, ChunkType::Function);
        assert!(chunks[0].content.contains("hello"));

        assert_eq!(chunks[1].symbol_name.as_deref(), Some("world"));
        assert_eq!(chunks[1].chunk_type, ChunkType::Function);
    }

    #[test]
    fn test_chunk_fallback_when_no_symbols() {
        let chunker = SemanticChunker::new(3);
        let source = "line1\nline2\nline3\nline4\nline5\nline6\nline7\nline8";
        let symbols = vec![];

        let chunks = chunker.chunk(source, &symbols).unwrap();
        assert_eq!(chunks.len(), 3); // 8 lines / 3 = ~3 chunks
        assert!(chunks[0].symbol_name.is_none());
        assert_eq!(chunks[0].chunk_type, ChunkType::Module);
    }

    #[test]
    fn test_chunk_class_symbol() {
        let chunker = SemanticChunker::new(50);
        let source = "class MyClass {\n    fn method() {}\n}";
        let symbols = vec![
            make_symbol("MyClass", SymbolKind::Class, 0, 2),
        ];

        let chunks = chunker.chunk(source, &symbols).unwrap();
        assert_eq!(chunks[0].chunk_type, ChunkType::Class);
        assert_eq!(chunks[0].symbol_name.as_deref(), Some("MyClass"));
    }

    #[test]
    fn test_chunk_empty_source() {
        let chunker = SemanticChunker::new(10);
        let chunks = chunker.chunk("", &[]).unwrap();
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_out_of_bounds() {
        let chunker = SemanticChunker::new(50);
        let source = "fn one_line() {}";
        // Symbol spanning beyond source lines
        let symbols = vec![
            make_symbol("oob", SymbolKind::Function, 0, 100),
        ];

        let chunks = chunker.chunk(source, &symbols).unwrap();
        assert_eq!(chunks.len(), 1);
        // Should handle out of bounds gracefully (min with source length)
        assert_eq!(chunks[0].end_line, 0);
    }
}
