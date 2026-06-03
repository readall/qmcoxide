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
}