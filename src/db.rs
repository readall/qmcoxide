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
    // PRAGMAs for durability/performance parity with original (WAL, etc.)
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL; PRAGMA busy_timeout=5000;")?;
    // Enable extension loading for sqlite-vec (rusqlite bundled supports).
    // Note: for some platforms, may need conn.load_extension with path to libsqlite_vec.so/dylib/dll
    // See load_sqlite_vec below for hints (port of original mac Homebrew etc).
    unsafe {
        conn.load_extension_enable()?;
    }
    // Try load vec (non-fatal if fails, as FTS still works; caller can error on vec use).
    if let Err(e) = load_sqlite_vec(&conn) {
        eprintln!("Warning: could not load sqlite-vec extension (vec search disabled): {e}. Install platform sqlite-vec or build ext.");
    }
    // Init schema then migrations (mig table etc must exist for run_migrations query/inserts)
    init_schema(&conn)?;
    run_migrations(&conn)?;
    Ok(conn)
}

pub fn load_sqlite_vec(conn: &Connection) -> Result<()> {
    // Real load per .9: try env SQLITE_VEC_PATH, then common names (original uses platform lib like Homebrew /usr/local/opt/sqlite-vec/lib/libsqlite_vec.dylib etc).
    // Non-fatal: caller warns, vec search disabled if not present (parity with original when ext missing).
    // For win: user provides dll via env or build ext; see AGENTS Windows note.
    if let Ok(p) = std::env::var("SQLITE_VEC_PATH") {
        unsafe {
            conn.load_extension(&p, None)?;
        }
        return Ok(());
    }
    for candidate in &["sqlite-vec", "libsqlite_vec", "sqlite_vec", "libsqlite_vec.so", "libsqlite_vec.dylib", "sqlite_vec.dll"] {
        unsafe {
            if conn.load_extension(candidate, None).is_ok() {
                return Ok(());
            }
        }
    }
    // fallback no-op (common on CI/dev without the platform sqlite-vec .so/dylib/dll installed)
    // Warn here for the common case; explicit bad SQLITE_VEC_PATH still errors above.
    eprintln!("Warning: could not load sqlite-vec extension (vec search disabled). Install platform sqlite-vec or build ext (or set SQLITE_VEC_PATH).");
    Ok(())
}

/// Initialize current schema + FTS5 + vec0 (from original analysis + data-model.md).
/// Migrations for legacy (path fixes, fp columns, vec dim, case from CHANGELOG implicit + requirements).
/// Run on every open; use simple ALTER IF NOT EXISTS pattern (or catch).
pub fn init_schema(conn: &Connection) -> Result<()> {
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
            last_modified TEXT,
            fingerprint TEXT,
            last_embed_at INTEGER
        );
        CREATE VIRTUAL TABLE IF NOT EXISTS documents_fts USING fts5(
            name, body, path, tokenize='unicode61'
        );
        CREATE TABLE IF NOT EXISTS chunks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            doc_hash TEXT NOT NULL,
            seq INTEGER,
            start_line INTEGER,
            end_line INTEGER,
            content TEXT,
            token_count INTEGER,
            fingerprint TEXT,
            chunk_strategy TEXT
        );
        -- chunks_fts added in migration v3 for snippet+line search (.28)
        -- vectors_vec VIRTUAL vec0 created dynamically in embed (after knowing dim, e.g. 768 or model specific)
        -- CREATE VIRTUAL TABLE IF NOT EXISTS vectors_vec USING vec0(hash_seq TEXT PRIMARY KEY, embedding FLOAT[768] distance_metric=cosine);
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
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY
        );
    "#)?;

    Ok(())
}

/// Basic migrations for legacy indexes (path fixes, fp columns, vec dim, case from CHANGELOG implicit + requirements).
/// Run on every open; use simple ALTER IF NOT EXISTS pattern (or catch).
pub fn run_migrations(conn: &Connection) -> Result<()> {
    // Simple versioned mig (extend as needed)
    let current: i32 = conn.query_row("SELECT COALESCE(MAX(version), 0) FROM schema_migrations", [], |r| r.get(0)).unwrap_or(0);

    if current < 1 {
        // Example: add fp columns if missing (for legacy)
        let _ = conn.execute("ALTER TABLE documents ADD COLUMN fingerprint TEXT", []);
        let _ = conn.execute("ALTER TABLE documents ADD COLUMN last_embed_at INTEGER", []);
        conn.execute("INSERT OR IGNORE INTO schema_migrations (version) VALUES (1)", [])?;
    }
    if current < 2 {
        // vec dim or other; in practice recreate virtual if needed in embed path
        conn.execute("INSERT OR IGNORE INTO schema_migrations (version) VALUES (2)", [])?;
    }
    if current < 3 {
        // chunks_fts for accurate snippet extraction + line numbers in search (.28)
        let _ = conn.execute_batch(
            "CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5( path, content, doc_hash, tokenize='unicode61' );"
        );
        conn.execute("INSERT OR IGNORE INTO schema_migrations (version) VALUES (3)", [])?;
    }
    // Add more for path verbatim fixes, case etc. (data fixups if needed)
    Ok(())
}

// More schema (content_vectors) and tx wrappers in future; current supports indexing/search/get parity.

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
        for exp in ["content", "documents", "documents_fts", "chunks", "path_contexts", "store_collections", "llm_cache", "schema_migrations"] {
            assert!(tables.iter().any(|t| t.contains(exp)), "missing table: {exp}");
        }
        // vectors_vec is virtual, created on demand in embed (with dim); FTS/vec queries exercised in higher tests
        // run_migrations called in open (test uses init directly)
    }
}
