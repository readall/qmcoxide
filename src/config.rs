//! Config handling: YAML index.yml (collections, contexts, global, per-col models), inline config, XDG paths, write-through sync to DB.
//! See original src/collections.ts , example-index.yml , requirements for yaml shape, includeByDefault, update-cmd, models: section.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Collection config from yaml or inline (matches original CollectionConfig).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionConfig {
    pub path: String,
    #[serde(default = "**/*.md")]
    pub pattern: String,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub context: std::collections::HashMap<String, String>,  // path prefix -> context
}

/// Top level config (global_context, collections map, models per col?).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub global_context: Option<String>,
    #[serde(default)]
    pub collections: std::collections::HashMap<String, CollectionConfig>,
    // models: { embed: , ... } per top or per col in advanced
}

pub fn load_config_from_path(path: &PathBuf) -> AppConfig {
    // TODO: use serde_yaml::from_str( &std::fs::read_to_string(path)? )
    // XDG: dirs::config_dir().join("qmd/index.yml")
    AppConfig::default()
}

pub fn load_config() -> AppConfig {
    // support configPath, inline, DB-only fallback
    AppConfig::default()
}

// TODO: write through for add/remove etc to yaml if source was yaml; per-col models support.

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_config_default_and_load() {
        let cfg = load_config();
        assert!(cfg.global_context.is_none());
        // TODO: test yaml parse with example-index.yml content
    }
}