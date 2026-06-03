//! DB layer: rusqlite (or bun equiv but Rust), open, schema creation + migrations (for legacy indexes: path fixes, fingerprints, etc), FTS5, vec0 extension load, prepared stmts.
//! Cross platform sqlite (system or bundled).
//! See original src/db.ts (compat layer), store.ts for CREATEs, migrate-schema.ts , data-model.md .

use rusqlite::{Connection, Result};

pub fn open_database(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    // TODO: load extension for vec if available (platform hints like Homebrew on mac)
    // conn.load_extension...
    Ok(conn)
}

pub fn init_schema(conn: &Connection) -> Result<()> {
    // TODO: CREATE TABLE documents, content, ... documents_fts USING fts5(...), vectors_vec USING vec0(...), etc.
    // Migrations for legacy.
    Ok(())
}

// TODO: load_sqlite_vec, transaction helpers, specific queries for fts, vec, etc.