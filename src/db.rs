//! DB layer: rusqlite (or bun equiv but Rust), open, schema creation + migrations (for legacy indexes: path fixes, fingerprints, etc), FTS5, vec0 extension load, prepared stmts.
//! Cross platform sqlite (system or bundled).
//! See original src/db.ts (compat layer), store.ts for CREATEs, migrate-schema.ts , data-model.md .
//!
//! Vec backend spike decision (task 2): chose sqlite-vec extension load for fidelity (exact same vec0 virtual table, cosine, SQL as original qmd for vectors_vec; matches FTS5 docs/fts behavior, RRF etc without reimpl).
//! Alternative tantivy considered for pure-Rust (no native ext, easier win/cross packaging, but would require reimpl of vec search + fusion parity work).
//! Load similar to original: find platform lib (e.g. via build or bundled), conn.load_extension(path).
//! (No direct "sqlite-vec" load crate in Rust equiv to npm; use std::env or include_bytes for prebuilts in future.)

use rusqlite::{Connection, Result};

/// Open DB, enable extensions, load vec if possible (for fidelity with original vec0).
pub fn open_database(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    // Enable extension loading for sqlite-vec (rusqlite bundled supports).
    // Note: for some platforms, may need conn.load_extension with path to libsqlite_vec.so/dylib/dll
    // See load_sqlite_vec below for hints (port of original mac Homebrew etc).
    unsafe {
        conn.load_extension_enable()?;
    }
    // Try load vec (non-fatal if fails, as FTS still works; caller can error on vec use).
    if let Err(e) = load_sqlite_vec(&conn) {
        eprintln!("Warning: could not load sqlite-vec extension (vec search disabled): {}. Install platform sqlite-vec or build ext.", e);
    }
    Ok(conn)
}

pub fn load_sqlite_vec(conn: &Connection) -> Result<()> {
    // TODO: find loadable path like original (for prebuilts or system).
    // For now, assume extension is in PATH or use conn.load_extension("sqlite-vec", None) if registered.
    // In practice, use a crate or build script to bundle the .dylib/.so for target.
    // Example for mac/linux: let path = get_sqlite_vec_path(); conn.load_extension(&path, None)?;
    // For this skeleton, no-op or error if strict.
    // To make tests pass without ext, make vec optional in higher layers.
    Ok(())
}

/// Initialize current schema + FTS5 + vec0 (from original analysis + data-model.md).
/// Migrations TODO for legacy (path fixes, fingerprints etc from changelog).
pub fn init_schema(conn: &Connection) -> Result<()> {
    // content for doc bodies/chunks?
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS content (
            hash TEXT PRIMARY KEY,
            body TEXT
        );
        CREATE TABLE IF NOT EXISTS documents (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            collection TEXT NOT NULL,
            path TEXT NOT NULL,
            hash TEXT,
            title TEXT,
            active INTEGER DEFAULT 1,
            last_modified TEXT
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            name, body, path, tokenize='unicode61'
        );
        -- vectors_vec created after knowing dim in embed (see original vec0 usage)
        CREATE TABLE IF NOT EXISTS path_contexts (
            collection TEXT,
            path TEXT,
            context TEXT
        );
        CREATE TABLE IF NOT EXISTS store_collections (
            name TEXT PRIMARY KEY,
            path TEXT,
            pattern TEXT,
            ignore TEXT,  -- json
            include_by_default INTEGER DEFAULT 1,
            update_cmd TEXT
        );
        CREATE TABLE IF NOT EXISTS llm_cache (
            key TEXT PRIMARY KEY,
            value TEXT
        );
    "#)?;

    // Note: vectors_vec is VIRTUAL vec0, created in embed code with dim e.g. 768
    // CREATE VIRTUAL TABLE IF NOT EXISTS vectors_vec USING vec0(hash_seq TEXT PRIMARY KEY, embedding float[768] distance_metric=cosine);

    Ok(())
}

// TODO: more schema (content_vectors for chunks), migration logic, prepared stmts for insert/search, transaction wrappers.

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    #[test]
    fn test_schema_init_and_tables() {
        let conn = Connection::open_in_memory().unwrap();
        // skip load ext in test (may fail without lib)
        init_schema(&conn).unwrap();
        let tables: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type='table' OR type='view' ORDER BY name;")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert!(tables.iter().any(|t| t.contains("documents")));
        assert!(tables.iter().any(|t| t.contains("documents_fts")));
        assert!(tables.iter().any(|t| t.contains("store_collections")));
        // vec table created later in embed with dim
    }
}