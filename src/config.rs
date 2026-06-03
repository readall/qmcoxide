//! Config handling: YAML index.yml (collections, contexts, global, per-col models), inline config, XDG paths, write-through sync to DB.
//! See original src/collections.ts , example-index.yml , requirements for yaml shape, includeByDefault, update-cmd, models: section.

use std::path::PathBuf;

/// Placeholder for config loading / collection defs.
#[derive(Debug, Default)]
pub struct Config {
    pub global_context: Option<String>,
    // collections: ...
}

pub fn load_config() -> Config {
    // TODO: support configPath, inline, or DB-only; XDG ~/.config/qmd/index.yml
    Config::default()
}

// TODO: add/remove collection, context, set_global, per-collection model overrides (embed/rerank/generate)