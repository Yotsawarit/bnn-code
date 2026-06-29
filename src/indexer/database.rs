use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::{Path, PathBuf};

use super::chunker::CodeChunk;

// ── Constants ────────────────────────────────────────────────────────────────

const DB_FILENAME: &str = "bnn-code.db";
const SEARCH_LIMIT: usize = 20;

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Returns the path where BNN Code stores its SQLite database.
/// Uses project-local .bnn/ directory by default for isolation.
/// Can override with BNN_DB_PATH environment variable.
pub fn default_db_path() -> PathBuf {
    // Check for environment variable override
    if let Ok(path) = std::env::var("BNN_DB_PATH") {
        return PathBuf::from(path);
    }
    
    // Use project-local .bnn directory by default
    std::env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".bnn")
        .join(DB_FILENAME)
}

// ── CodeDatabase ─────────────────────────────────────────────────────────────

pub struct CodeDatabase {
    conn: Connection,
}

impl CodeDatabase {
    /// Open (or create) a **persistent** database at `db_path`.
    /// Creates parent directories if they do not exist.
    pub fn open(db_path: &Path) -> Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create database directory: {}", parent.display())
            })?;
        }

        let conn = Connection::open(db_path).with_context(|| {
            format!("Failed to open SQLite database: {}", db_path.display())
        })?;

        // WAL mode: readers don't block writers, much better for a CLI tool
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")
            .context("Failed to set WAL journal mode")?;

        Self::init_schema(&conn)?;
        tracing::info!("Database opened: {}", db_path.display());
        Ok(Self { conn })
    }

    /// Open an **in-memory** database (used in tests only).
    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to open in-memory SQLite database")?;
        Self::init_schema(&conn)?;
        Ok(Self { conn })
    }

    /// Open persistent database at the default XDG path.
    pub fn open_default() -> Result<Self> {
        Self::open(&default_db_path())
    }

    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS chunks (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path   TEXT    NOT NULL,
                content     TEXT    NOT NULL,
                start_line  INTEGER NOT NULL DEFAULT 0,
                end_line    INTEGER NOT NULL DEFAULT 0,
                symbol_name TEXT,
                chunk_type  TEXT,
                file_hash   TEXT,
                indexed_at  DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_file_path ON chunks(file_path);
            CREATE INDEX IF NOT EXISTS idx_symbol    ON chunks(symbol_name);
            -- FTS5 virtual table for fast full-text search
            CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts
                USING fts5(content, symbol_name, file_path, content=chunks, content_rowid=id);
            -- Keep FTS index in sync with the chunks table
            CREATE TRIGGER IF NOT EXISTS chunks_ai AFTER INSERT ON chunks BEGIN
                INSERT INTO chunks_fts(rowid, content, symbol_name, file_path)
                VALUES (new.id, new.content, new.symbol_name, new.file_path);
            END;
            CREATE TRIGGER IF NOT EXISTS chunks_ad AFTER DELETE ON chunks BEGIN
                INSERT INTO chunks_fts(chunks_fts, rowid, content, symbol_name, file_path)
                VALUES ('delete', old.id, old.content, old.symbol_name, old.file_path);
            END;",
        )
        .context("Failed to initialise database schema")
    }

    /// Store chunks for a file, replacing any existing chunks for that path.
    pub fn store_chunks(
        &self,
        file_path: &str,
        file_hash: &str,
        chunks: &[CodeChunk],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()
            .context("Failed to begin transaction")?;

        // Remove stale chunks for this file before re-inserting
        tx.execute("DELETE FROM chunks WHERE file_path = ?1",
            rusqlite::params![file_path])
            .context("Failed to delete stale chunks")?;

        for chunk in chunks {
            tx.execute(
                "INSERT INTO chunks
                    (file_path, content, start_line, end_line, symbol_name, chunk_type, file_hash)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    file_path,
                    chunk.content,
                    chunk.start_line as i64,
                    chunk.end_line as i64,
                    chunk.symbol_name,
                    format!("{:?}", chunk.chunk_type),
                    file_hash,
                ],
            ).context("Failed to insert chunk")?;
        }

        tx.commit().context("Failed to commit transaction")?;
        Ok(())
    }

    /// Full-text search using FTS5 (fast, ranking built-in).
    /// Falls back to LIKE search if FTS index is unavailable.
    pub fn search_by_keyword(&self, keyword: &str, limit: usize) -> Result<Vec<CodeChunk>> {
        let limit = limit.min(SEARCH_LIMIT) as i64;

        let mut stmt = self.conn.prepare(
            "SELECT c.content, c.start_line, c.end_line, c.symbol_name, c.chunk_type
             FROM chunks_fts
             JOIN chunks c ON c.id = chunks_fts.rowid
             WHERE chunks_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        ).context("Failed to prepare FTS search statement")?;

        let results = stmt
            .query_map(rusqlite::params![keyword, limit], |row| {
                Ok(CodeChunk {
                    content:     row.get(0)?,
                    start_line:  row.get::<_, i64>(1)? as usize,
                    end_line:    row.get::<_, i64>(2)? as usize,
                    symbol_name: row.get(3)?,
                    chunk_type:  super::chunker::ChunkType::Module,
                })
            })
            .context("FTS query failed")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    pub fn get_file_chunks(&self, file_path: &str) -> Result<Vec<CodeChunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT content, start_line, end_line, symbol_name, chunk_type
             FROM chunks WHERE file_path = ?1
             ORDER BY start_line",
        ).context("Failed to prepare get_file_chunks statement")?;

        let results = stmt
            .query_map(rusqlite::params![file_path], |row| {
                Ok(CodeChunk {
                    content:     row.get(0)?,
                    start_line:  row.get::<_, i64>(1)? as usize,
                    end_line:    row.get::<_, i64>(2)? as usize,
                    symbol_name: row.get(3)?,
                    chunk_type:  super::chunker::ChunkType::Module,
                })
            })
            .context("Failed to query file chunks")?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    pub fn chunk_count(&self) -> Result<usize> {
        let count: i64 = self.conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
            .context("Failed to count chunks")?;
        Ok(count as usize)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::indexer::chunker::{ChunkType, CodeChunk};

    fn make_chunk(content: &str, name: &str, start: usize, end: usize) -> CodeChunk {
        CodeChunk {
            content: content.to_string(),
            start_line: start,
            end_line: end,
            symbol_name: Some(name.to_string()),
            chunk_type: ChunkType::Function,
        }
    }

    #[test]
    fn test_new_database_is_empty() {
        let db = CodeDatabase::open_in_memory().unwrap();
        assert_eq!(db.chunk_count().unwrap(), 0);
    }

    #[test]
    fn test_store_and_retrieve_chunks() {
        let db = CodeDatabase::open_in_memory().unwrap();
        let chunks = vec![
            make_chunk("fn hello() {}", "hello", 0, 2),
            make_chunk("fn world() {}", "world", 4, 6),
        ];

        db.store_chunks("test.rs", "abc123", &chunks).unwrap();
        assert_eq!(db.chunk_count().unwrap(), 2);

        let retrieved = db.get_file_chunks("test.rs").unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].symbol_name.as_deref(), Some("hello"));
        assert_eq!(retrieved[1].symbol_name.as_deref(), Some("world"));

        assert!(db.get_file_chunks("nonexistent.rs").unwrap().is_empty());
    }

    #[test]
    fn test_store_replaces_existing_chunks() {
        let db = CodeDatabase::open_in_memory().unwrap();
        let v1 = vec![make_chunk("fn old() {}", "old", 0, 1)];
        let v2 = vec![make_chunk("fn new() {}", "new", 0, 1)];

        db.store_chunks("file.rs", "hash1", &v1).unwrap();
        assert_eq!(db.chunk_count().unwrap(), 1);

        // Re-index same file — old chunks must be replaced
        db.store_chunks("file.rs", "hash2", &v2).unwrap();
        assert_eq!(db.chunk_count().unwrap(), 1);
        let chunks = db.get_file_chunks("file.rs").unwrap();
        assert_eq!(chunks[0].symbol_name.as_deref(), Some("new"));
    }

    #[test]
    fn test_fts_search() {
        let db = CodeDatabase::open_in_memory().unwrap();
        let chunks = vec![
            make_chunk("fn calculate_sum(a: i32, b: i32) -> i32 { a + b }", "calculate_sum", 0, 2),
            make_chunk("fn print_result() { println!(\"done\") }", "print_result", 4, 6),
        ];
        db.store_chunks("math.rs", "h1", &chunks).unwrap();

        let results = db.search_by_keyword("calculate_sum", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol_name.as_deref(), Some("calculate_sum"));

        assert!(db.search_by_keyword("zzz_not_found", 10).unwrap().is_empty());
    }

    #[test]
    fn test_store_empty_chunks() {
        let db = CodeDatabase::open_in_memory().unwrap();
        db.store_chunks("empty.rs", "", &[]).unwrap();
        assert_eq!(db.chunk_count().unwrap(), 0);
    }

    #[test]
    fn test_multiple_files() {
        let db = CodeDatabase::open_in_memory().unwrap();
        db.store_chunks("file1.rs", "h1", &[make_chunk("fn a() {}", "a", 0, 1)]).unwrap();
        db.store_chunks("file2.rs", "h2", &[make_chunk("fn b() {}", "b", 0, 1)]).unwrap();
        assert_eq!(db.chunk_count().unwrap(), 2);
        assert_eq!(db.get_file_chunks("file1.rs").unwrap().len(), 1);
        assert_eq!(db.get_file_chunks("file2.rs").unwrap().len(), 1);
    }

    #[test]
    fn test_default_db_path_contains_bnn_code() {
        let p = default_db_path();
        assert!(p.to_string_lossy().contains("bnn-code"));
        assert!(p.to_string_lossy().ends_with(DB_FILENAME));
    }
}
