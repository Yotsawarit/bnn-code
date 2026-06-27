use anyhow::Result;
use rusqlite::Connection;

use super::chunker::CodeChunk;

pub struct CodeDatabase {
    conn: Connection,
}

impl CodeDatabase {
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory()?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS chunks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                file_path TEXT NOT NULL,
                content TEXT NOT NULL,
                start_line INTEGER,
                end_line INTEGER,
                symbol_name TEXT,
                chunk_type TEXT,
                file_content TEXT,
                indexed_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_file_path ON chunks(file_path);
            CREATE INDEX IF NOT EXISTS idx_symbol ON chunks(symbol_name);
            ",
        )?;

        Ok(Self { conn })
    }

    pub fn store_chunks(
        &self,
        file_path: &str,
        file_content: &str,
        chunks: &[CodeChunk],
    ) -> Result<()> {
        let tx = self.conn.unchecked_transaction()?;

        for chunk in chunks {
            tx.execute(
                "INSERT INTO chunks (file_path, content, start_line, end_line, symbol_name, chunk_type, file_content)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![
                    file_path,
                    chunk.content,
                    chunk.start_line as i64,
                    chunk.end_line as i64,
                    chunk.symbol_name,
                    format!("{:?}", chunk.chunk_type),
                    file_content,
                ],
            )?;
        }

        tx.commit()?;
        Ok(())
    }

    pub fn search_by_keyword(&self, keyword: &str) -> Result<Vec<CodeChunk>> {
        let pattern = format!("%{}%", keyword);
        let mut stmt = self.conn.prepare(
            "SELECT content, start_line, end_line, symbol_name, chunk_type
             FROM chunks
             WHERE content LIKE ?1
             LIMIT 20",
        )?;

        let results = stmt
            .query_map(rusqlite::params![pattern], |row| {
                Ok(CodeChunk {
                    content: row.get(0)?,
                    start_line: row.get::<_, i64>(1)? as usize,
                    end_line: row.get::<_, i64>(2)? as usize,
                    symbol_name: row.get(3)?,
                    chunk_type: super::chunker::ChunkType::Module,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    pub fn get_file_chunks(&self, file_path: &str) -> Result<Vec<CodeChunk>> {
        let mut stmt = self.conn.prepare(
            "SELECT content, start_line, end_line, symbol_name, chunk_type
             FROM chunks WHERE file_path = ?1
             ORDER BY start_line",
        )?;

        let results = stmt
            .query_map(rusqlite::params![file_path], |row| {
                Ok(CodeChunk {
                    content: row.get(0)?,
                    start_line: row.get::<_, i64>(1)? as usize,
                    end_line: row.get::<_, i64>(2)? as usize,
                    symbol_name: row.get(3)?,
                    chunk_type: super::chunker::ChunkType::Module,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();

        Ok(results)
    }

    pub fn chunk_count(&self) -> Result<usize> {
        let count: i64 =
            self.conn.query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))?;
        Ok(count as usize)
    }
}

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
        let db = CodeDatabase::new().unwrap();
        assert_eq!(db.chunk_count().unwrap(), 0);
    }

    #[test]
    fn test_store_and_retrieve_chunks() {
        let db = CodeDatabase::new().unwrap();
        let chunks = vec![
            make_chunk("fn hello() {}", "hello", 0, 2),
            make_chunk("fn world() {}", "world", 4, 6),
        ];

        db.store_chunks("test.rs", "full file content", &chunks).unwrap();
        assert_eq!(db.chunk_count().unwrap(), 2);

        // Retrieve by file path
        let retrieved = db.get_file_chunks("test.rs").unwrap();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].symbol_name.as_deref(), Some("hello"));
        assert_eq!(retrieved[1].symbol_name.as_deref(), Some("world"));

        // Retrieve non-existent file
        let empty = db.get_file_chunks("nonexistent.rs").unwrap();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_search_by_keyword() {
        let db = CodeDatabase::new().unwrap();
        let chunks = vec![
            make_chunk("fn calculate_sum() { 1 + 1 }", "calculate_sum", 0, 2),
            make_chunk("fn print_result() { println!(\"done\") }", "print_result", 4, 6),
        ];

        db.store_chunks("math.rs", "full file", &chunks).unwrap();

        let results = db.search_by_keyword("calculate").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].symbol_name.as_deref(), Some("calculate_sum"));

        // Search for non-existent keyword
        let no_results = db.search_by_keyword("zzz_not_found").unwrap();
        assert!(no_results.is_empty());
    }

    #[test]
    fn test_store_empty_chunks() {
        let db = CodeDatabase::new().unwrap();
        db.store_chunks("empty.rs", "", &[]).unwrap();
        assert_eq!(db.chunk_count().unwrap(), 0);
    }

    #[test]
    fn test_multiple_files() {
        let db = CodeDatabase::new().unwrap();

        db.store_chunks("file1.rs", "", &[make_chunk("fn a() {}", "a", 0, 1)]).unwrap();
        db.store_chunks("file2.rs", "", &[make_chunk("fn b() {}", "b", 0, 1)]).unwrap();

        assert_eq!(db.chunk_count().unwrap(), 2);

        assert_eq!(db.get_file_chunks("file1.rs").unwrap().len(), 1);
        assert_eq!(db.get_file_chunks("file2.rs").unwrap().len(), 1);
    }
}
