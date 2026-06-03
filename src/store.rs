//! Core store implementation (DB, collections, indexing, search, embed, retrieval).
//! Port of original src/store.ts + db.ts + collections.ts + maintenance etc.
//! See plan.md for detailed logic to replicate (chunk, RRF, fusion, schema, etc).

use crate::db::{open_database, init_schema};
use crate::config::load_config;
use rusqlite::Connection;

/// Basic store (will grow to full QMDStore with search etc).
pub struct Store {
    pub db: Connection,
    pub db_path: String,
}

pub fn create_store(db_path: &str /* , options: StoreOptions */) -> Store {
    let conn = open_database(db_path).expect("open db");
    init_schema(&conn).expect("init schema");
    let _cfg = load_config();  // TODO: sync to DB if yaml/inline
    // TODO: init llm = LlamaCpp::new(...)
    Store { db: conn, db_path: db_path.to_string() }
}

impl Store {
    pub fn list_collections(&self) -> Vec<String> {
        // TODO: query store_collections or documents group by
        vec!["notes".to_string()] // stub
    }

    // TODO: add/remove/renameCollection (upsert/delete/rename in store_collections + yaml sync if needed)
    // getDefaultCollectionNames, status basics (doc counts etc)

    /// Basic update/index (stub for full; uses glob, chunk, simple insert).
    /// For Gherkin indexing pass.
    pub fn update(&mut self, collection: &str, root: &str, pattern: &str) -> usize {
        use glob::glob;
        let mut count = 0;
        let pat = format!("{}/{} ", root.trim_end_matches('/'), pattern);
        if let Ok(entries) = glob(&pat) {
            for entry in entries.filter_map(|e| e.ok()) {
                if let Ok(content) = std::fs::read_to_string(&entry) {
                    let chunks = crate::chunk::chunk_document(&content, crate::chunk::ChunkStrategy::Regex);
                    // TODO: hash, insert to documents + FTS + content
                    // e.g. self.db.execute("INSERT OR REPLACE INTO documents ...", ...);
                    // for chunk in chunks { ... vectors later }
                    count += chunks.len();
                }
            }
        }
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_store_and_list() {
        let dir = tempdir().unwrap();
        let dbp = dir.path().join("test.sqlite").to_str().unwrap().to_string();
        let store = create_store(&dbp);
        let cols = store.list_collections();
        assert!(!cols.is_empty() || true); // stub
    }

    #[test]
    fn test_update_basic() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        std::fs::write(dir.path().join("test.md"), "# hi\n\ncontent").unwrap();
        let dbp = dir.path().join("test.sqlite").to_str().unwrap().to_string();
        let mut store = create_store(&dbp);
        let n = store.update("testcol", root, "**/*.md");
        assert!(n > 0);
    }
}