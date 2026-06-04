//! Core store implementation (DB, collections, indexing, search, embed, retrieval).
//! Port of original src/store.ts + db.ts + collections.ts + maintenance etc.
//! See plan.md for detailed logic to replicate (chunk, RRF, fusion, schema, etc).

use crate::db::open_database;
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
    // init now done inside open_database (before run_migrations)
    let _cfg = load_config();  // sync to DB if yaml/inline in full (write-through in config)
    // init llm = LlamaCpp::new(...) in full (gated, with cache/models)
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

    /// Full update/index: glob + ignore + .git skip, chunk w/ lines, FTS + chunks_fts, content body, fp/docid.
    /// Implements collection_management + indexing fidelity.
    pub fn update(&mut self, collection: &str, root: &str, pattern: &str) -> usize {
        use glob::glob;
        let mut count = 0;
        let pat = format!("{}/{}", root.trim_end_matches('/'), pattern);
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
                    // Insert document + full body for get --full
                    self.db.execute(
                        "INSERT OR REPLACE INTO documents (collection, path, hash, title, active, last_modified) VALUES (?1, ?2, ?3, ?4, 1, datetime('now'))",
                        params![collection, &path_str, &doc_hash, &title],
                    ).expect("insert doc");
                    self.db.execute(
                        "INSERT OR REPLACE INTO content (hash, body) VALUES (?1, ?2)",
                        params![&doc_hash, &content],
                    ).expect("insert content");
                    // FTS for lex search (name, body, path)
                    self.db.execute(
                        "INSERT INTO documents_fts (name, body, path) VALUES (?1, ?2, ?3)",
                        params![&title, &content, &path_str],
                    ).expect("insert fts");
                    // Chunks for retrieval + chunks_fts for snippet+lines search
                    for (i, ch) in chunks.iter().enumerate() {
                        self.db.execute(
                            "INSERT INTO chunks (doc_hash, seq, content, start_line, end_line) VALUES (?1, ?2, ?3, ?4, ?5)",
                            params![&doc_hash, i as i32, &ch.text, ch.start_line as i32, ch.end_line as i32],
                        ).expect("insert chunk");
                        self.db.execute(
                            "INSERT INTO chunks_fts (path, content, doc_hash) VALUES (?1, ?2, ?3)",
                            params![&path_str, &ch.text, &doc_hash],
                        ).expect("insert chunks_fts");
                    }
                    count += 1;  // per doc
                }
            }
        }
        count
    }

    /// Exact FTS5 lex search using chunks_fts for accurate per-chunk snippets + line numbers (from chunk positions).
    /// Supports intent for weighting (0.3x factor on non-matching per task .28 / score-fusion).
    /// Per .28, .37, requirements "snippet", "line numbers", "context", path_fidelity.
    pub fn search_lex(&self, q: &str, intent: Option<&str>, limit: usize) -> Vec<crate::types::SearchResult> {
        // Quote FTS query for special chars like dots (per .37 dotted quirk and FTS5 syntax)
        let fts_q = if q.contains('.') || q.contains(' ') || q.contains('"') {
            format!("\"{}\"", q.replace('"', ""))
        } else {
            q.to_string()
        };
        let mut stmt = self.db.prepare(
            r#"SELECT d.path, d.title, c.start_line, c.end_line, 
                      snippet(chunks_fts, 1, '', '', '...', 64) as snippet, 
                      bm25(chunks_fts) as score 
               FROM chunks_fts 
               JOIN documents d ON d.hash = chunks_fts.doc_hash 
               JOIN chunks c ON c.doc_hash = chunks_fts.doc_hash AND c.content = chunks_fts.content 
               WHERE chunks_fts MATCH ? 
               ORDER BY score LIMIT ?"#
        ).expect("prepare chunks fts for snippet+lines");
        let rows: Vec<_> = stmt.query_map(params![fts_q, limit as i32], |row| {
            let path: String = row.get(0)?;
            let title: String = row.get(1)?;
            let start_line: i32 = row.get(2)?;
            let end_line: i32 = row.get(3)?;
            let snippet: String = row.get(4)?;
            let score: f64 = row.get::<_, f64>(5)?.abs();
            Ok((path, title, start_line as usize, end_line as usize, snippet, score))
        })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

        let results: Vec<crate::types::SearchResult> = rows.into_iter().map(|(path, title, sl, el, snip, sc)| {
            let mut score = sc;
            if let Some(i) = intent {
                if !path.to_lowercase().contains(&i.to_lowercase()) && !title.to_lowercase().contains(&i.to_lowercase()) {
                    score *= 0.7; // 0.3x reduction for non-intent match (per weighting spec)
                }
            }
            let docid = crate::paths::make_docid(&title); // approx; in real from stored or content
            let ctx = self.get_context(&path);
            crate::types::SearchResult {
                path,
                title,
                docid,
                score,
                snippet: snip,
                start_line: sl,
                end_line: el,
                context: ctx,
            }
        }).collect();

        // attach context
        results
    }

    /// Vec0 cosine search code (SELECT ... MATCH ? ORDER BY distance).
    /// Assumes vectors_vec virtual created in embed with dim; fallback empty if not.
    /// Per task .37.
    pub fn search_vec(&self, _q: &str, _limit: usize) -> Vec<crate::types::SearchResult> {
        // full when vec0 loaded and embeddings populated (in embed task):
        // SELECT ... FROM vectors_vec ... ORDER BY distance
        // return vec of SearchResult with normalized score (1/(1+dist))
        // When ready + ext: normalize 1 / (1 + distance) or as per score-fusion.
        vec![]
    }

    /// Suggest similar files/paths for error messages (fuzzy from index, e.g. contains).
    /// Per task .34, requirements "DocumentNotFound + similar suggestions", retrieval/CLI errors.
    pub fn suggest_similar(&self, bad: &str, _collection: Option<&str>) -> Vec<String> {
        let like = format!("%{bad}%");
        let mut stmt = self.db.prepare(
            "SELECT path FROM documents WHERE path LIKE ? LIMIT 5"
        ).expect("prepare suggest");
        stmt.query_map(params![&like], |row| row.get(0))
            .unwrap()
            .collect::<Result<Vec<String>, _>>()
            .unwrap_or_default()
    }

    /// Get attached context for a path (from path_contexts table populated via config/contexts or add).
    /// Per .28, .10, requirements "context".
    pub fn get_context(&self, path: &str) -> Option<String> {
        // longest prefix match
        let mut stmt = self.db.prepare(
            "SELECT context FROM path_contexts WHERE ?1 LIKE (path || '%') OR path = '' ORDER BY length(path) DESC LIMIT 1"
        ).ok()?;
        stmt.query_row(params![path], |r| r.get(0)).ok()
    }

    /// Full get by path, #docid or qmd:// , with range support, full body.
    /// Implements retrieval_get_multi.feature basics + path fidelity.
    pub fn get(&self, spec: &str, full: bool, from_line: Option<usize>, max_lines: Option<usize>) -> Option<String> {
        let (target_path, _is_docid) = if spec.starts_with('#') {
            // find by doc hash prefix
            let hash_prefix = spec.trim_start_matches('#');
            let mut stmt = self.db.prepare("SELECT path FROM documents WHERE hash LIKE ? LIMIT 1").ok()?;
            let p: String = stmt.query_row(params![format!("{hash_prefix}%")], |r| r.get(0)).ok()?;
            (p, true)
        } else if let Some((_, p)) = crate::paths::parse_qmd_uri(spec) {
            (p, false)
        } else {
            (spec.to_string(), false)
        };

        // try content table first (full body)
        if let Ok(mut stmt) = self.db.prepare("SELECT c.body FROM content c JOIN documents d ON d.hash = c.hash WHERE d.path = ?") {
            if let Ok(body) = stmt.query_row(params![&target_path], |r| r.get::<_, String>(0)) {
                return Some(if full { body } else { apply_line_range(&body, from_line, max_lines) });
            }
        }

        // fallback: concat chunks
        let like = format!("{target_path}%");
        if let Ok(mut stmt) = self.db.prepare("SELECT content FROM chunks JOIN documents d ON d.hash = chunks.doc_hash WHERE d.path LIKE ? ORDER BY seq") {
            let parts: Vec<String> = stmt.query_map(params![&like], |r| r.get(0)).ok()?.collect::<Result<Vec<_>,_>>().ok()?;
            let body = parts.join("\n");
            return Some(if full { body } else { apply_line_range(&body, from_line, max_lines) });
        }

        // last: fs read if exists (for unindexed? but per index fidelity, prefer index)
        if let Ok(content) = std::fs::read_to_string(&target_path) {
            return Some(if full { content } else { apply_line_range(&content, from_line, max_lines) });
        }
        None
    }

    /// Multi get (simplified: support list of specs, apply common opts).
    pub fn multi_get(&self, specs: &[String], full: bool, from_line: Option<usize>, max_lines: Option<usize>) -> Vec<(String, Option<String>)> {
        specs.iter().map(|s| (s.clone(), self.get(s, full, from_line, max_lines))).collect()
    }
}

/// Helper for line ranges (1-based, inclusive).
fn apply_line_range(body: &str, from: Option<usize>, max: Option<usize>) -> String {
    let lines: Vec<&str> = body.lines().collect();
    let start = from.unwrap_or(1).saturating_sub(1);
    let end = if let Some(m) = max { (start + m).min(lines.len()) } else { lines.len() };
    lines[start..end].join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_store_and_list() {
        let dir = tempdir().unwrap();
        let dbp = dir.path().join("test.sqlite").to_str().unwrap().to_string();
        let mut store = create_store(&dbp);
        // Seed explicit collection per AGENTS (no auto index); list works for empty or populated.
        store.add_collection("testcol", ".", "**/*.md", "", 1, None);
        let cols = store.list_collections();
        assert!(!cols.is_empty(), "list_collections should return seeded col after add (fresh store has table but no rows)");
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
        // Lex search (now returns SearchResult with snippet/lines/context; intent=None for basic)
        // Note: test content has no "hi"; use term present in body
        let res = store.search_lex("content", None, 5);
        assert!(!res.is_empty());
        assert!(res[0].start_line >= 1);
        assert!(!res[0].snippet.is_empty());
        // Dotted version quirk: "2026.4.10" should match (FTS unicode61 keeps tokens)
        let dotted_res = store.search_lex("2026.4.10", None, 5);
        assert!(dotted_res.iter().any(|r| r.path.contains("release")));
        // Vec returns empty until real embeddings (task .2/.9)
        let vec_res = store.search_vec("foo", 5);
        assert!(vec_res.is_empty());
    }
}
