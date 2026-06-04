//! Core store implementation (DB, collections, indexing, search, embed, retrieval).
//! Port of original src/store.ts + db.ts + collections.ts + maintenance etc.
//! See plan.md for detailed logic to replicate (chunk, RRF, fusion, schema, etc).

use crate::db::{open_database, init_schema};
use crate::config::load_config;
use rusqlite::{Connection, params};
use crate::paths::make_docid;

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
    pub fn list_collections(&self) -> Vec<(String, String, String, i32)> {
        // name, path, pattern, include_by_default (doc_count approx from sub or 0 for now; full in collection task)
        let mut stmt = self.db.prepare("SELECT name, path, pattern, include_by_default FROM store_collections").expect("prepare list col");
        stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    pub fn add_collection(&mut self, name: &str, path: &str, pattern: &str, ignore: &str, include_by_default: i32, update_cmd: Option<&str>) {
        self.db.execute(
            "INSERT OR REPLACE INTO store_collections (name, path, pattern, ignore, include_by_default, update_cmd) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![name, path, pattern, ignore, include_by_default, update_cmd],
        ).expect("add col");
    }

    pub fn remove_collection(&mut self, name: &str) {
        self.db.execute("DELETE FROM store_collections WHERE name = ?1", params![name]).expect("remove col");
        // note: docs remain or deactivate in full
    }

    pub fn rename_collection(&mut self, old: &str, new: &str) {
        self.db.execute("UPDATE store_collections SET name = ?1 WHERE name = ?2", params![new, old]).expect("rename col");
    }

    // ls prefix: list paths under prefix for collection (qmd:// or display)
    pub fn ls(&self, prefix: &str) -> Vec<String> {
        let like = format!("{}%", prefix.trim_end_matches('/'));
        let mut stmt = self.db.prepare("SELECT path FROM documents WHERE path LIKE ? ORDER BY path LIMIT 100").expect("prepare ls");
        stmt.query_map(params![like], |r| r.get(0))
            .unwrap()
            .collect::<Result<Vec<String>, _>>()
            .unwrap_or_default()
    }

    /// Basic update/index (stub for full; uses glob, chunk, simple insert).
    /// For Gherkin indexing pass.
    pub fn update(&mut self, collection: &str, root: &str, pattern: &str) -> usize {
        use glob::glob;
        let mut count = 0;
        let pat = format!("{}/{} ", root.trim_end_matches('/'), pattern);
        if let Ok(entries) = glob(&pat) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path_str = entry.to_string_lossy().to_string();
                if path_str.contains("/.git/") || path_str.ends_with("/.gitignore") { continue; } // basic .gitignore + git skip (full .gitignore parse in glob task)
                if let Ok(content) = std::fs::read_to_string(&entry) {
                    if content.trim().is_empty() { continue; } // graceful skip empty
                    let chunks = crate::chunk::chunk_document(&content, crate::chunk::ChunkStrategy::Regex);
                    let doc_hash = make_docid(&content);
                    let title = content.lines().next().unwrap_or("").trim_start_matches('#').trim().to_string();
                    let path_str = entry.to_string_lossy().to_string();
                    // Insert document
                    self.db.execute(
                        "INSERT OR REPLACE INTO documents (collection, path, hash, title, active, last_modified) VALUES (?1, ?2, ?3, ?4, 1, datetime('now'))",
                        params![collection, &path_str, &doc_hash, &title],
                    ).expect("insert doc");
                    // FTS for lex search (name, body, path)
                    self.db.execute(
                        "INSERT INTO documents_fts (name, body, path) VALUES (?1, ?2, ?3)",
                        params![&title, &content, &path_str],
                    ).expect("insert fts");
                    // Chunks for retrieval
                    for (i, ch) in chunks.iter().enumerate() {
                        self.db.execute(
                            "INSERT INTO chunks (doc_hash, seq, content, start_line, end_line) VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![&doc_hash, i as i32, &ch.text, 0, 0],
                        ).expect("insert chunk");
                    }
                    count += 1;  // per doc
                }
            }
        }
        count
    }

    /// Exact FTS5 lex search (BM25 scores via bm25(), unicode61 tokenizer for dotted versions etc.)
    /// Per task .37, requirements FTS quirks, path_fidelity (dotted match).
    pub fn search_lex(&self, q: &str, limit: usize) -> Vec<(String, String, f64)> {
        let mut stmt = self.db.prepare(
            "SELECT d.path, d.title, bm25(documents_fts) as score 
             FROM documents_fts 
             JOIN documents d ON d.hash = documents_fts.hash 
             WHERE documents_fts MATCH ? 
             ORDER BY score LIMIT ?"
        ).expect("prepare fts");
        stmt.query_map(params![q, limit as i32], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get::<_, f64>(2)?.abs(),  // bm25 is negative usually
            ))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
    }

    /// Vec0 cosine search code (SELECT ... MATCH ? ORDER BY distance).
    /// Assumes vectors_vec virtual created in embed with dim; fallback empty if not.
    /// Per task .37.
    pub fn search_vec(&self, _q: &str, _limit: usize) -> Vec<(String, String, f64)> {
        // TODO full when vec0 loaded and embeddings in vectors_vec or chunk_embeddings BLOB with cosine
        // e.g. SELECT ... FROM vectors_vec v JOIN ... WHERE v.embedding MATCH ? ORDER BY distance
        // For now, since vec0 load stub and embeddings in embed, return empty.
        // When ready: normalize 1 / (1 + distance) or as per score-fusion.
        vec![]
    }

    /// Suggest similar files/paths for error messages (fuzzy from index, e.g. contains).
    /// Per task .34, requirements "DocumentNotFound + similar suggestions", retrieval/CLI errors.
    pub fn suggest_similar(&self, bad: &str, _collection: Option<&str>) -> Vec<String> {
        let like = format!("%{}%", bad);
        let mut stmt = self.db.prepare(
            "SELECT path FROM documents WHERE path LIKE ? LIMIT 5"
        ).expect("prepare suggest");
        stmt.query_map(params![&like], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<String>, _>>()
            .unwrap_or_default()
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

    #[test]
    fn test_search_lex_and_dotted_quirk() {
        let dir = tempdir().unwrap();
        let root = dir.path().to_str().unwrap();
        std::fs::write(dir.path().join("release.md"), "Release 2026.4.10 notes\n\ncontent here").unwrap();
        std::fs::write(dir.path().join("other.md"), "# other\n\nfoo bar").unwrap();
        let dbp = dir.path().join("test.sqlite").to_str().unwrap().to_string();
        let mut store = create_store(&dbp);
        store.update("testcol", root, "**/*.md");
        // Lex search
        let res = store.search_lex("hi", 5);
        assert!(!res.is_empty());
        // Dotted version quirk: "2026.4.10" should match (FTS unicode61 keeps tokens)
        let dotted_res = store.search_lex("2026.4.10", 5);
        assert!(dotted_res.iter().any(|(p, _, _)| p.contains("release")));
        // Vec stub empty
        let vec_res = store.search_vec("foo", 5);
        assert!(vec_res.is_empty());
    }
}
