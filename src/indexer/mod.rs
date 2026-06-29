pub mod chunker;
pub mod database;
pub mod parser;

use anyhow::Result;
use std::path::Path;
use walkdir::WalkDir;

/// Supported file extensions for indexing
/// Languages: Rust, Python, JS/TS, Go, Java, C/C++, Ruby, Swift, Kotlin
const SUPPORTED_EXTENSIONS: &[&str] = &[
    "rs", "py", "js", "ts", "tsx", "jsx", "mjs", "cjs",
    "go", "java", "cpp", "c", "h", "hpp", "cc", "cxx",
    "rb", "swift", "kt", "kts",
];

pub struct CodebaseIndexer {
    root_path: String,
    parser: parser::AstParser,
    chunker: chunker::SemanticChunker,
    database: database::CodeDatabase,
}

impl CodebaseIndexer {
    pub fn new(root_path: &str) -> Result<Self> {
        let parser = parser::AstParser::new()?;
        let chunker = chunker::SemanticChunker::new(50);
        let database = database::CodeDatabase::open_default()?;

        Ok(Self {
            root_path: root_path.to_string(),
            parser,
            chunker,
            database,
        })
    }

    /// Get a reference to the underlying database for search operations
    #[allow(dead_code)]
    pub fn database(pub fn database(&self) -> &database::CodeDatabase {self) -> pub fn database(&self) -> &database::CodeDatabase {database::CodeDatabase {
        &self.database
    }

    pub async fn index(&mut self) -> Result<usize> {
        tracing::info!("Indexing codebase at: {}", self.root_path);
        let mut total_chunks = 0;

        for entry in WalkDir::new(&self.root_path)
            .into_iter()
            .filter_entry(|e| {
                let name = e.file_name().to_string_lossy();
                !name.starts_with('.') && name != "node_modules" && name != "target"
            })
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if SUPPORTED_EXTENSIONS.contains(&ext) {
                    match self.index_file(path) {
                        Ok(chunks) => {
                            total_chunks += chunks;
                            tracing::debug!("Indexed {:?} ({} chunks)", path, chunks);
                        }
                        Err(e) => {
                            tracing::warn!("Failed to index {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        tracing::info!("Indexing complete: {} chunks", total_chunks);
        Ok(total_chunks)
    }

    fn index_file(&mut self, path: &Path) -> Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        // Parse AST and extract symbols in one pass
        let symbols = self.parser.parse(&content, &ext)?;

        // Chunk the content
        let chunks = self.chunker.chunk(&content, &symbols)?;

        // Store in database
        let rel_path = path.strip_prefix(&self.root_path)?.to_string_lossy();
        self.database
            .store_chunks(&rel_path, &content, &chunks)?;

        Ok(chunks.len())
    }
}
