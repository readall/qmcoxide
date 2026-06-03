//! CLI: clap parser for all commands/subcommands/flags from CLAUDE.md + README + 2.5+ (collection*, context*, get/multi-get with :suffix/full-path/line-nums, search/vsearch/query with intent/explain/format/full-path, embed, update --pull, status, doctor, mcp --http --daemon, ls, cleanup, bench?).
//! Formatters for outputs (default TTY color + OSC8 hyperlinks via QMD_EDITOR_URI, json/csv/md/xml/files, explain traces).
//! Progress, doctor impl (diagnostics for sqlite, fingerprints, device/GPU, env, models), error handling.
//! See original src/cli/qmd.ts (huge), formatter.ts , bin/qmd (launcher logic eliminated in Rust), requirements CLI inventory.

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "qmd", version, about = "qmd (Rust port) - on-device hybrid search")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
    // --index <name> global, etc.
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Stub for collection subcommands
    Collection {
        #[command(subcommand)]
        action: CollectionAction,
    },
    Context {
        #[arg(value_name = "TEXT")]
        text: Option<String>,
    },
    Get {
        path_or_docid: String,
        #[arg(long)]
        full: bool,
        #[arg(short = 'l', long)]
        max_lines: Option<usize>,
        #[arg(long)]
        full_path: bool,
    },
    // Add more: Search { query: String, json: bool, ... }, Query, Embed, Update, Status, Doctor, Mcp { http: bool, port: Option<u16>, daemon: bool, }, Ls, Cleanup, Bench { fixture: String }, ...
    /// Fallback / help
    #[command(external_subcommand)]
    Other(Vec<String>),
}

#[derive(Subcommand, Debug)]
pub enum CollectionAction {
    Add { path: String, #[arg(long)] name: Option<String> },
    List,
    Remove { name: String },
    // etc.
}

pub fn run(cli: Cli) {
    // TODO: dispatch to store, format results per --format or flags, handle TTY, doctor, etc.
    match &cli.command {
        Commands::Get { path_or_docid, .. } => println!("get stub for {}", path_or_docid),
        Commands::Other(args) if !args.is_empty() => println!("other/unknown: {:?}", args),
        _ => println!("CLI stub (clap parsed) - see plan, features/*.feature, docs/requirements.md for full parity implementation"),
    }
}