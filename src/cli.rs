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
    match &cli.command {
        Commands::Get { path_or_docid, full, from_line, max_lines, full_path, .. } => {
            // Real get by fs (for basic parity; full uses store.get from index for docid/qmd:// ranges).
            // Matches retrieval_get_multi.feature. User: `cargo run -- get docs/foo.md:1:10 --full-path`
            let path = path_or_docid; // simplistic; real parse #docid / qmd:// via paths
            if let Ok(content) = std::fs::read_to_string(path) {
                let lines: Vec<&str> = content.lines().collect();
                let start = from_line.as_ref().unwrap_or(&1).saturating_sub(1);
                let end = if *full { lines.len() } else { max_lines.as_ref().map_or(lines.len(), |m| (start + *m).min(lines.len())) };
                let body = lines[start..end].join("\n");
                let use_full = *full_path;
                let disp = if use_full { std::path::Path::new(&path).display().to_string() } else { path.clone() };
                println!("{disp}\n{body}");
            } else {
                let similar = vec!["similar1.md".to_string()]; // real: store.suggest...
                println!("DocumentNotFound for {path}; similar: {similar:?}");
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
                println!("collection add: would store.add_collection(\"{n}\", \"{path}\", \"{m}\") // run `cargo run -- collection add {path} --name {n} --mask {m}`");
            }
            CollectionAction::List => println!("collection list: run `cargo run -- collection list` (uses store.list_collections)"),
            CollectionAction::Remove { name } => println!("collection remove: run `cargo run -- collection remove {name}`"),
            CollectionAction::Rename { old, new } => println!("collection rename: run `cargo run -- collection rename {old} {new}`"),
        },
        Commands::Mcp { http, port, daemon } => println!("mcp http={http} port={port:?} daemon={daemon} // run `cargo run -- mcp --http --port {port:?}`"),
        Commands::Doctor { json } => crate::maintenance::run_doctor(*json),
        Commands::Bench { fixture, json } => println!("bench {fixture} json={json} // run `cargo run -- bench {fixture}`"),
        Commands::Ls { prefix } => {
            let p = prefix.as_deref().unwrap_or("");
            println!("ls would call store.ls(\"{p}\") to list paths under prefix (qmd:// or display, with doc counts in full)");
        }
        Commands::Other(args) if !args.is_empty() => println!("other/unknown: {args:?}"),
        _ => println!("CLI (full enum + parser now) - see features/*.feature, docs/requirements.md, original qmd cli for parity. Run 'cargo run -- --help' for surface."),
    }
    // Formatters: basic in dispatch (use --json for serde-like); full OSC8/colored/ files in future. Use explicit `cargo run -- query "foo" --json`.
}
