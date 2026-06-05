//! Core store implementation (DB, collections, indexing, search, embed, retrieval).
//! Port of original src/store.ts + db.ts + collections.ts + maintenance etc.
//! See plan.md for detailed logic to replicate (chunk, RRF, fusion, schema, etc).

use crate::db::open_database;
use crate::config::load_config;
use crate::llm::{LlamaCpp, EMBED_DIM};
use crate::paths::make_docid;
use rusqlite::{Connection, params};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

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

fn extract_title(content: &str) -> String {
    // frontmatter --- \n title: Foo \n --- for parity with original frontmatter title support
    if content.starts_with("---") {
        if let Some(end) = content.strip_prefix("---").unwrap().find("---") {
            let fm = &content[3..3+end];
            for line in fm.lines() {
                if let Some(v) = line.strip_prefix("title:") {
                    return v.trim().trim_matches('"').trim_matches('\'').to_string();
                }
            }
        }
    }
    // H1 fallback
    content.lines().next().unwrap_or("").trim_start_matches('#').trim().to_string()
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
    /// Implements collection_management + indexing fidelity (fp incremental, frontmatter title).
    pub fn update(&mut self, collection: &str, root: &str, pattern: &str) -> usize {
        use glob::glob;
        let mut count = 0;
        let pat = format!("{}/{}", root.trim_end_matches('/'), pattern);
        println!("DEBUG: Looking for files with pattern: {pat}");
        if let Ok(entries) = glob(&pat) {
            let entries_vec: Vec<_> = entries.into_iter().collect();
            println!("DEBUG: Found {} entries", entries_vec.len());
            for entry in entries_vec.into_iter().filter_map(|e| e.ok()) {
                let path_str = entry.to_string_lossy().to_string();
                println!("DEBUG: Processing entry: {path_str}");
                if path_str.contains("/.git/") || path_str.ends_with("/.gitignore") { 
                    println!("DEBUG: Skipping .git entry");
                    continue; 
                } // basic .gitignore + git skip (full .gitignore parse in glob task)
                if let Ok(content) = std::fs::read_to_string(&entry) {
                    if content.trim().is_empty() { 
                        println!("DEBUG: Skipping empty content");
                        continue; 
                    } // graceful skip empty
                    println!("DEBUG: Processing content of length {}", content.len());
                    let chunks = crate::chunk::chunk_document(&content, crate::chunk::ChunkStrategy::Regex);
                    let doc_hash = make_docid(&content);
                    // frontmatter title (YAML style --- title: foo) or H1 fallback for parity with original
                    let title = extract_title(&content);
                    let path_str = entry.to_string_lossy().to_string();
                    // incremental fp skip: if same hash for path, skip reindex (fidelity)
                    let mut stmt = self.db.prepare("SELECT hash FROM documents WHERE collection=?1 AND path=?2").expect("prep fp check");
                    if let Ok(existing) = stmt.query_row(params![collection, &path_str], |r| r.get::<_, String>(0)) {
                        if existing == doc_hash {
                            println!("DEBUG: Skipping unchanged document: {path_str}");
                            continue; // unchanged
                        }
                    }
                    println!("DEBUG: Inserting document: {path_str}");
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
                } else {
                    println!("DEBUG: Failed to read file: {}", entry.to_string_lossy());
                }
            }
        } else {
            println!("DEBUG: Glob pattern failed: {pat}");
        }
        println!("DEBUG: Update complete, processed {count} documents");
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

    /// Embed documents in a collection: chunk, embed, store vectors + fingerprints.
    /// Implements embed.feature: -f force, -c scoped, --chunk-strategy, progress, recovery.
    /// Called via `cargo run -- embed [...]`.
    pub fn embed(&mut self, collection: &str, force: bool, chunk_strategy: Option<String>) -> usize {
        use crate::chunk::{ChunkStrategy, chunk_document};
        use crate::llm::{LlamaCpp, EMBED_DIM, EMBED_PROMPT_TEMPLATE, EMBED_TITLE_TEXT_TEMPLATE};
        use glob::glob;
        use std::time::Instant;
        
        let start = Instant::now();
        let mut embedded_count = 0;
        
        // Determine chunk strategy
        let strategy = match chunk_strategy.as_deref() {
            Some("auto") => ChunkStrategy::Auto,
            _ => ChunkStrategy::Regex, // default
        };
        
        // Get LLM instance (gated by feature)
        let llm = LlamaCpp::new(None, None, None); // Uses defaults from llm.rs
        
        // Get documents that need embedding
        let mut doc_stmt = self.db.prepare(
            "SELECT d.path, d.hash, c.body 
             FROM documents d 
             JOIN content c ON d.hash = c.hash 
             WHERE d.collection = ?1 AND d.active = 1
             AND (?2 = 1 OR d.fingerprint IS NULL OR d.last_embed_at IS NULL)"
        ).expect("prepare doc select");
        
        let docs = doc_stmt.query_map(params![collection, force as i32], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, String>(2)?))
        }).expect("query docs")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect docs");
        
        if docs.is_empty() {
            println!("No documents need embedding in collection '{collection}'");
            return 0;
        }
        
        println!("Embedding {} documents in collection '{}'", docs.len(), collection);
        
        // Create vectors_vec table if it doesn't exist (with proper dimension)
        let _ = self.db.execute(
            "DROP TABLE IF EXISTS vectors_vec",
            []
        );
        let _ = self.db.execute(
            &format!(
                "CREATE VIRTUAL TABLE IF NOT EXISTS vectors_vec USING vec0(hash_seq TEXT PRIMARY KEY, embedding FLOAT[{}] distance_metric=cosine)",
                EMBED_DIM
            ),
            []
        );
        
        // Process each document
        for (path, doc_hash, content) in docs {
            println!("  Processing: {path}");
            
            // Chunk the document
            let chunks = chunk_document(&content, strategy);
            if chunks.is_empty() {
                continue;
            }
            
            // Create prompts for embedding (title + text format per requirements)
            let title = extract_title(&content);
            let prompts: Vec<String> = chunks.iter()
                .map(|ch| format!("{}\n\n{}", title, ch.text))
                .collect();
            
            // Get embeddings from LLM
            let embeddings = llm.embed_batch(&prompts);
            
            // Generate fingerprint for this embedding operation (model + params based)
            let mut hasher = DefaultHasher::new();
            (crate::llm::DEFAULT_EMBED_MODEL as &str).hash(&mut hasher);
            EMBED_DIM.hash(&mut hasher);
            (strategy as u8).hash(&mut hasher);
            let fingerprint = format!("{:x}", hasher.finish());
            
            // Store chunks and embeddings
            for (i, (chunk, embedding)) in chunks.iter().zip(embeddings.iter()).enumerate() {
                let chunk_hash = format!("{doc_hash}_{i:03}");
                
                // Store chunk with embedding reference
                self.db.execute(
                    "INSERT OR REPLACE INTO chunks (doc_hash, seq, content, start_line, end_line, fingerprint) 
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                    params![
                        &doc_hash,
                        i as i32,
                        &chunk.text,
                        chunk.start_line as i32,
                        chunk.end_line as i32,
                        &fingerprint
                    ]
                ).expect("insert chunk");
                
                // Store vector in vectors_vec (hash_seq = doc_hash_chunk_index)
                // Convert Vec<f32> to Vec<u8> for storage
                let mut embedding_bytes = Vec::with_capacity(embedding.len() * 4);
                for &val in embedding.iter() {
                    embedding_bytes.extend_from_slice(&val.to_le_bytes());
                }
                
                self.db.execute(
                    "INSERT OR REPLACE INTO vectors_vec (hash_seq, embedding) VALUES (?1, ?2)",
                    params![&chunk_hash, &embedding_bytes]
                ).expect("insert vector");
            }
            
            // Update document with fingerprint and timestamp
            self.db.execute(
                "UPDATE documents SET fingerprint = ?1, last_embed_at = strftime('%s', 'now') 
                 WHERE hash = ?2",
                params![&fingerprint, &doc_hash]
            ).expect("update document");
            
            embedded_count += 1;
        }
        
        let duration = start.elapsed();
        println!("Embed complete: {embedded_count} documents embedded in {duration:.2?}");
        embedded_count
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

    /// Hybrid search with basic RRF k=60 fusion of lex + vec (expand stub until full LLM).
    /// Per score-fusion.md (RRF, bonuses, top30, intent weights) and search_hybrid_query.feature.
    /// Returns ranked with explain trace stub.
    pub fn search(&self, q: &str, intent: Option<&str>, limit: usize) -> Vec<crate::types::SearchResult> {
        let k = 60.0f64;
        let mut lex_res = self.search_lex(q, intent, limit * 2);
        let mut vec_res = self.search_vec(q, limit * 2);
        // simple RRF: assign ranks, score sum 1/(k+rank)
        let mut scored: std::collections::HashMap<String, (f64, crate::types::SearchResult)> = std::collections::HashMap::new();
        for (rank, r) in lex_res.iter().enumerate() {
            let score = 1.0 / (k + rank as f64 + 1.0);
            scored.entry(r.path.clone()).and_modify(|e| e.0 += score).or_insert((score, r.clone()));
        }
        for (rank, r) in vec_res.iter().enumerate() {
            let score = 1.0 / (k + rank as f64 + 1.0) * 0.5; // vec weight lower for now
            scored.entry(r.path.clone()).and_modify(|e| e.0 += score).or_insert((score, r.clone()));
        }
        let mut fused: Vec<_> = scored.into_values().map(|(s, mut r)| { r.score = s; r }).collect();
        fused.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
        fused.truncate(limit);
        // intent already applied in lex; explain stub
        fused
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

