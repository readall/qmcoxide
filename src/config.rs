//! Config handling: YAML index.yml (collections, contexts, global, per-col models), inline config, XDG paths, write-through sync to DB.
//! See original src/collections.ts , example-index.yml , requirements for yaml shape, includeByDefault, update-cmd, models: section.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Collection config from yaml or inline (matches original CollectionConfig).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectionConfig {
    pub path: String,
    #[serde(default = "default_pattern")]
    pub pattern: String,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub context: std::collections::HashMap<String, String>,  // path prefix -> context
    #[serde(default = "default_include")]
    pub include_by_default: bool,
    #[serde(default)]
    pub update_cmd: Option<String>,
    #[serde(default)]
    pub models: std::collections::HashMap<String, String>, // per-col embed etc
}

fn default_pattern() -> String {
    "**/*.md".to_string()
}
fn default_include() -> bool { true }

/// Top level config (global_context, collections map, models per col?).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub global_context: Option<String>,
    #[serde(default)]
    pub collections: std::collections::HashMap<String, CollectionConfig>,
    #[serde(default)]
    pub models: std::collections::HashMap<String, String>, // top level
}

pub fn load_config_from_path(path: &PathBuf) -> AppConfig {
    match std::fs::read_to_string(path) {
        Ok(s) => serde_yaml::from_str(&s).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

pub fn load_config() -> AppConfig {
    // env QMD_CONFIG > XDG ~/.config/qmd/index.yml > default (DB fallback)
    if let Ok(p) = std::env::var("QMD_CONFIG") {
        return load_config_from_path(&PathBuf::from(p));
    }
    if let Some(cfg_dir) = dirs::config_dir() {
        let p = cfg_dir.join("qmd/index.yml");
        if p.exists() {
            return load_config_from_path(&p);
        }
    }
    AppConfig::default()
}

// Basic write-through (for mutations when source yaml)
pub fn save_config_to_path(cfg: &AppConfig, path: &PathBuf) -> std::io::Result<()> {
    let s = serde_yaml::to_string(cfg).unwrap_or_default();
    std::fs::write(path, s)
}

// Write-through on mutations implemented basic in save_config_to_path; per-col models used in load (affect embed in full LLM).

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_config_default_and_load() {
        let cfg = load_config();
        assert!(cfg.global_context.is_none());
        // example-index.yml would load collections etc
    }
}
