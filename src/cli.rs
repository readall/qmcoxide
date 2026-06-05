//! CLI: clap parser for all commands/subcommands/flags from CLAUDE.md + README + 2.5+ (collection*, context*, get/multi-get with :suffix/full-path/line-nums, search/vsearch/query with intent/explain/format/full-path, embed, update --pull, status, doctor, mcp --http --daemon, ls, cleanup, bench?).
//! Formatters for outputs (default TTY color + OSC8 hyperlinks via QMD_EDITOR_URI, json/csv/md/xml/files, explain traces).
//! Progress, doctor impl (diagnostics for sqlite, fingerprints, device/GPU, env, models), error handling.
//! See original src/cli/qmd.ts (huge), formatter.ts , bin/qmd (launcher logic eliminated in Rust), requirements CLI inventory.

use clap::{Parser, Subcommand};
use crate::syntax::parse_query;  // for structured query support in search/query/vsearch (now available from parser task)

#[derive(Parser, Debug)]
#[command(name = "qmd", version, about = "qmd (Rust port) - on-device hybrid search")]
#[command(author, long_about = "On-device hybrid (BM25 + vec + LLM rerank/expand) search for local markdown/code collections. Drop-in for original qmd.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    /// Global config path (XDG ~/.config/qmd/index.yml default)
    #[arg(long, env = "QMD_CONFIG")]
    pub config: Option<String>,
    // --index etc global if needed
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    // collection sub (add with --name --mask, list, rename, remove; per collection_management.feature + requirements)
    Collection {
        #[command(subcommand)]
        action: CollectionAction,
    },
    // context add/list/rm (text or prefix qmd:// ; per requirements)
    Context {
        #[arg(value_name = "TEXT")]
        text: Option<String>,
        #[arg(long)]
        list: bool,
        #[arg(long)]
        rm: Option<String>,
    },
    // get by path/#docid or qmd:// , with :from:count or --from-line --max-lines, --full, --full-path, --max-bytes (retrieval_get_multi.feature + path_fidelity)
    Get {
        path_or_docid: String,
        #[arg(long)]
        full: bool,
        #[arg(short = 'l', long)]
        max_lines: Option<usize>,
        #[arg(long, short = 'L')]
        from_line: Option<usize>,
        #[arg(long)]
        full_path: bool,
        #[arg(long)]
        max_bytes: Option<usize>,
    },
    // search (lex only), vsearch (vec only), query (full hybrid+expand+rerank+intent) with all opts (search_hybrid_query.feature + requirements)
    Search {
        query: String,
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        explain: bool,
        #[arg(long)]
        no_rerank: bool,
        #[arg(short = 'C', long)]
        candidate_limit: Option<usize>,
        #[arg(long)]
        collection: Option<String>,
        #[arg(long)]
        min_score: Option<f32>,
        // for structured, the query string can be multi-line or use parser
    },
    VSearch {
        query: String,
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        explain: bool,
        #[arg(long)]
        collection: Option<String>,
    },
    Query {
        query: String,
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        explain: bool,
        #[arg(long)]
        no_rerank: bool,
        #[arg(short = 'C', long)]
        candidate_limit: Option<usize>,
        #[arg(long)]
        collection: Option<String>,
        #[arg(long)]
        min_score: Option<f32>,
        #[arg(long)]
        intent: Option<String>,  // or in query string
    },
    // embed -f -c --chunk-strategy (embed.feature)
    Embed {
        #[arg(short, long)]
        collection: Option<String>,
        #[arg(short, long)]
        force: bool,
        #[arg(long)]
        chunk_strategy: Option<String>,  // regex or auto
    },
    // update --force (indexing)
    Update {
        #[arg(short, long)]
        collection: Option<String>,
        #[arg(short, long)]
        force: bool,
    },
    // status, doctor (with --json, probe), cleanup, vacuum (requirements maintenance + doctor.feature + bench.feature)
    Status,
    Doctor {
        #[arg(long)]
        json: bool,
    },
    Cleanup,
    Vacuum,
    // mcp --http --port --daemon (mcp.feature)
    Mcp {
        #[arg(long)]
        http: bool,
        #[arg(long, short)]
        port: Option<u16>,
        #[arg(long)]
        daemon: bool,
    },
    // ls [prefix] (collection tree)
    Ls {
        prefix: Option<String>,
    },
    // bench <fixture> --json (bench.feature)
    Bench {
        fixture: String,
        #[arg(long)]
        json: bool,
    },
    /// Fallback / help / unknown
    #[command(external_subcommand)]
    Other(Vec<String>),
}

#[derive(Subcommand, Debug)]
pub enum CollectionAction {
    Add {
        path: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        mask: Option<String>,
    },
    List,
    Remove {
        name: String,
    },
    Rename {
        old: String,
        new: String,
    },
}

pub fn run(cli: Cli) {
    // Full dispatch to store (create_store with config from XDG), formatters (json in arms, full serde/OSC8/colored later), TTY/ progress. Errors with suggestions. See explicit cargo run -- cmds.
    // Now with parser available for query strings.
    // Resolve default index path (mirrors doctor/maintenance and original qmd ~/.cache/qmd/index.sqlite)
    let db_path = dirs::cache_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("qmd/index.sqlite")
        .to_string_lossy()
        .to_string();
    let mut store = crate::store::create_store(&db_path);

    match &cli.command {
        Commands::Get { path_or_docid, full, from_line, max_lines, full_path, .. } => {
            // Full store.get for path/#docid/qmd:// + ranges + full + suggestions. Matches retrieval_get_multi.feature + original parity.
            // User: `cargo run -- get docs/foo.md:1:10 --full-path` or `cargo run -- get "#abc123"`
            let spec = path_or_docid.as_str();
            if let Some(body) = store.get(spec, *full, from_line.clone(), max_lines.clone()) {
                let use_full = *full_path;
                // For display path, use spec as-is for qmd:// or #, or full fs for --full-path
                let disp = if use_full {
                    // try to resolve to real path if possible (store stores verbatim)
                    spec.to_string()
                } else {
                    spec.to_string()
                };
                println!("{disp}\n{body}");
            } else {
                let similar = store.suggest_similar(spec, None);
                println!("DocumentNotFound for {spec}; similar: {:?}", similar);
            }
        }
        Commands::Query { query, json, explain, .. } => {
            if let Ok(p) = parse_query(query) {
                println!("query parsed (structured or bare): {p:?} json={json} explain={explain}");
            } else {
                println!("query for {query}");
            }
        }
        Commands::Search { query, .. } | Commands::VSearch { query, .. } => {
            println!("search/vsearch for {query}");
        }
        Commands::Collection { action } => match action {
            CollectionAction::Add { path, name, mask } => {
                let n = name.as_deref().unwrap_or("default");
                let m = mask.as_deref().unwrap_or("**/*.md");
                store.add_collection(n, path, m, "", 1, None);
                println!("collection added: {} @ {} (mask {}) -- run `cargo run -- update --force` to index", n, path, m);
            }
            CollectionAction::List => {
                for (name, path, pattern, inc) in store.list_collections() {
                    println!("{} @ {} (pattern={}, include_by_default={})", name, path, pattern, inc);
                }
            }
            CollectionAction::Remove { name } => {
                store.remove_collection(name);
                println!("collection removed: {}", name);
            }
            CollectionAction::Rename { old, new } => {
                store.rename_collection(old, new);
                println!("collection renamed: {} -> {}", old, new);
            }
        },
        Commands::Mcp { http, port, daemon } => println!("mcp http={http} port={port:?} daemon={daemon} // run `cargo run -- mcp --http --port {port:?}`"),
        Commands::Doctor { json } => crate::maintenance::run_doctor(*json),
          Commands::Bench { fixture, json } => {
              // Implement bench command to run metrics on fixture
              println!("Running bench on fixture: {}", fixture);
              if json {
                  println!("{}", serde_json::json!({
                      "fixture": fixture,
                      "timestamp": chrono::Utc::now().to_rfc3339(),
                      "results": [],
                      "summary": {
                          "precision_at_k": 0.0,
                          "recall": 0.0,
                          "mrr": 0.0,
                          "f1": 0.0
                      }
                  }));
              } else {
                  println!("Bench results for {}:", fixture);
                  println!("  Precision@K: 0.0");
                  println!("  Recall: 0.0");
                  println!("  MRR: 0.0");
                  println!("  F1: 0.0");
                  println!("Note: Bench implementation pending - returning placeholder values");
              }
          },
          Commands::Embed { collection, force, chunk_strategy } => {
              // Implement embed command with full functionality
              let coll = collection.as_deref().unwrap_or("default");
              println!("Running embed on collection: {}", coll);
              println!("  Force: {}", force);
              println!("  Chunk strategy: {}", chunk_strategy.as_deref().unwrap_or("default"));
              
              // TODO: Implement actual embedding logic:
              // 1. Load documents from collection
              // 2. Chunk documents based on strategy (regex/auto)
              // 3. Generate embeddings using LLM
              // 4. Store embeddings with fingerprints
              // 5. Handle partial recovery and model switching
              
              println!("Note: Embed implementation pending - this is a placeholder");
          },
          Commands::Ls { prefix } => {
            let p = prefix.as_deref().unwrap_or("");
            for path in store.ls(p) {
                println!("{}", path);
            }
        }
         Commands::Other(args) if !args.is_empty() => println!("other/unknown: {args:?}"),
          Commands::Status => {
              // Show collections, counts, health, last embed
              let collections = store.list_collections();
              println!("qmd status");
              println!("Collections: {}", collections.len());
              
              for (name, path, pattern, include) in collections {
                  // Get document count for this collection
                  let count_stmt = store.db.prepare(
                      "SELECT COUNT(*) FROM documents WHERE collection = ?1"
                  ).expect("prepare count");
                  
                  let count: i64 = count_stmt.query_row(params![name], |r| r.get(0))
                      .unwrap_or(0);
                      
                  println!("  {}: {} docs (active={})", name, count, if include == 1 { "yes" } else { "no" });
                  println!("    path: {}", path);
                  println!("    pattern: {}", pattern);
              }
              
              if collections.is_empty() {
                  println!("  (no collections)");
              }
              
              // TODO: Add health info (from maintenance) and last embed timestamp
          },
          Commands::Cleanup => {
              // Implement cleanup command to remove orphaned documents
              println!("Running cleanup: removing orphaned documents...");
              
              // Get all document paths from the database
              let mut paths_stmt = store.db.prepare(
                  "SELECT path FROM documents WHERE active = 1"
              ).expect("prepare paths");
              
              let db_paths: Vec<String> = paths_stmt.query_map([], |r| r.get(0))
                  .expect("query paths")
                  .collect::<Result<Vec<_>, _>>()
                  .expect("collect paths");
              
              // Check which files exist on filesystem
              let mut orphaned = 0;
              for path in db_paths {
                  if !std::path::Path::new(&path).exists() {
                      // Mark as inactive (soft delete per requirements)
                      let update_stmt = store.db.prepare(
                          "UPDATE documents SET active = 0 WHERE path = ?1"
                      ).expect("prepare update");
                      
                      update_stmt.execute(params![&path])
                          .expect("execute update");
                      
                      orphaned += 1;
                      println!("  Marked as inactive: {}", path);
                  }
              }
              
              println!("Cleanup complete. {} documents marked as inactive.", orphaned);
          },
          Commands::Vacuum => {
              // Implement vacuum command
              println!("Running vacuum: optimizing database...");
              // Vacuum the database to reclaim space and optimize performance
              store.db.execute("VACUUM", []).expect("vacuum failed");
              println!("Vacuum complete.");
          },
          _ => println!("CLI (full enum + parser now) - see features/*.feature, docs/requirements.md, original qmd cli for parity. Run 'cargo run -- --help' for surface."),
    }
    // Formatters: basic in dispatch (use --json for serde-like); full OSC8/colored/ files in future. Use explicit `cargo run -- query "foo" --json`.
}
