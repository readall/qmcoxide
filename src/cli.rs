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
    // TODO full: dispatch to store (create_store with config), format per --json etc, TTY/OSC8 (QMD_EDITOR_URI), progress, doctor, errors.
    // Now with parser available for query strings.
    match &cli.command {
        Commands::Get { path_or_docid, .. } => {
            // Error handling per .34: DocumentNotFound with similarFiles suggestions (fuzzy from index)
            // In full impl: if not found in store.get, return DocumentNotFound { error: ..., similar_files: store.suggest_similar(...) }
            // Graceful: unreadable/empty skipped in update/index.
            if path_or_docid.starts_with('#') || path_or_docid.contains(':') {
                println!("get (with range/full-path etc) stub for {} (would suggest similar if not found)", path_or_docid);
            } else {
                println!("get stub for {} (DocumentNotFound example: similar files from index)", path_or_docid);
            }
        }
        Commands::Query { query, json, explain, .. } => {
            if let Ok(p) = parse_query(query) {
                println!("query parsed (structured or bare): {:?} json={} explain={}", p, json, explain);
            } else {
                println!("query stub for {}", query);
            }
        }
        Commands::Search { query, .. } | Commands::VSearch { query, .. } => {
            println!("search/vsearch stub for {}", query);
        }
        Commands::Collection { action } => match action {
            CollectionAction::Add { path, name, mask } => println!("collection add stub path={} name={:?} mask={:?}", path, name, mask),
            CollectionAction::List => println!("collection list stub"),
            CollectionAction::Remove { name } => println!("collection remove stub {}", name),
            CollectionAction::Rename { old, new } => println!("collection rename {} -> {}", old, new),
        },
        Commands::Mcp { http, port, daemon } => println!("mcp stub http={} port={:?} daemon={}", http, port, daemon),
        Commands::Doctor { json } => println!("doctor stub json={}", json),
        Commands::Bench { fixture, json } => println!("bench stub {} json={}", fixture, json),
        Commands::Ls { prefix } => println!("ls stub {:?}", prefix),
        Commands::Other(args) if !args.is_empty() => println!("other/unknown: {:?}", args),
        _ => println!("CLI (full enum + parser now) - see features/*.feature, docs/requirements.md, original qmd cli for parity. Run 'cargo run -- --help' for surface."),
    }
}
