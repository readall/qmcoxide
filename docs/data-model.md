# Data Model & SQLite Schema (current target for Rust port)

From original README "Data Storage", store.ts init, migrate-schema.ts, test setups, vec0 usage.

## Key Tables (approximate; extract exact CREATEs + indexes during impl from src/store.ts + migrate)
- content (hash TEXT PRIMARY KEY, ... body or processed text, title?, ...)
- documents (id INTEGER PRIMARY KEY, collection TEXT, path TEXT NOT NULL, hash, docid?, title?, active, last_modified?, ...)
- documents_fts (VIRTUAL TABLE ... USING fts5(name, body, path?, ... ) WITH tokenizer unicode61 or custom; content=... )
- content_vectors / chunk table (hash, seq, pos, text, model fingerprint, chunk_strategy?, ...)
- vectors_vec (VIRTUAL TABLE ... USING vec0( hash_seq TEXT PRIMARY KEY, embedding FLOAT[768 or dim] distance_metric=cosine ))
- path_contexts / contexts (collection, path, context TEXT)
- collections / store_collections (name, path, pattern, ignore JSON?, includeByDefault, update_cmd?, ...)
- llm_cache (keyed by prompt hash or similar for expand/rerank)

## Migrations / Legacy Handling
- Historical: collection_id removal, path "handelize" fixes (must store/return verbatim now), fingerprint columns for embed staleness, lowercase path fixes, vec dim changes (recreate table), etc.
- On open/update: detect legacy, run equivalent ALTER/ data fixups so old indexes "just work" or give clear migration path (per "clean break" decision: new index recommended, but cache dir compat for co-install).

## In Rust
- Use rusqlite + exec for schema + prepared for queries.
- Replicate all FTS5 creation options, vec0 load (extension), pragmas.
- Full schema + migration logic to be in src/db.rs or store init.
- See plan "SQLite Schema Summary" and original for complete.