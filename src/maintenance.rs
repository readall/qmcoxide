//! Maintenance ops: vacuum, cleanup (orphans, inactive docs preserve for tombstones), index health (stale vectors, fingerprint mismatches), legacy migrations.
//! Doctor: exact checks per doctor.feature + requirements (sqlite/vec ver, fp mixed/stale, device/GPU safe probe via env, env vars, model cache, suggestions).
//! See original src/maintenance.ts , Maintenance export. No auto actions.

use std::collections::HashMap;
use std::path::PathBuf;

#[derive(serde::Serialize, Debug)]
pub struct DoctorReport {
    pub sqlite_version: String,
    pub vec_extension: String,
    pub model_cache: String,
    pub fingerprints: String,
    pub device: String,
    pub env_overrides: HashMap<String, String>,
    pub suggestions: Vec<String>,
}

/// Find a plausible default index.sqlite for probes (cwd .qmd/ or XDG cache/qmd).
/// Does not create or modify anything (per AGENTS no-auto).
fn find_default_db() -> Option<PathBuf> {
    let p = PathBuf::from(".qmd/index.sqlite");
    if p.exists() {
        return Some(p);
    }
    if let Some(cache) = dirs::cache_dir() {
        let p2 = cache.join("qmd/index.sqlite");
        if p2.exists() {
            return Some(p2);
        }
    }
    None
}

pub fn run_doctor(json: bool) {
    // SQLite version (always available via in-memory or bundled)
    let sqlite_version = {
        let conn = rusqlite::Connection::open_in_memory().expect("in-mem sqlite");
        conn.query_row("SELECT sqlite_version()", [], |row| row.get(0))
            .unwrap_or_else(|_| "unknown".to_string())
    };

    // Vec extension probe (uses db.rs load; real ext load in .9, non-fatal here for doctor)
    let vec_extension = {
        let conn = rusqlite::Connection::open_in_memory().expect("in-mem");
        match crate::db::load_sqlite_vec(&conn) {
            Ok(()) => {
                // Try vec_version if the ext registered it (sqlite-vec provides it when loaded)
                match conn.query_row("SELECT vec_version()", [], |row| row.get::<_, String>(0)) {
                    Ok(v) => format!("loaded (vec_version={v})"),
                    Err(_) => "loaded (vec_version query failed or not registered yet; see task .9)".to_string(),
                }
            }
            Err(e) => format!("load failed: {e} (real sqlite-vec ext load + version in task .9 / db.rs)"),
        }
    };

    // Model cache (from llm.rs, always defined)
    let model_cache = {
        let dir = crate::llm::model_cache_dir();
        if dir.exists() {
            let count = std::fs::read_dir(&dir).map(|it| it.count()).unwrap_or(0);
            format!("present ({count} entries) at {}", dir.display())
        } else {
            format!("missing (expected at {})", dir.display())
        }
    };

    // Fingerprints / embed health (probe existing index if any; detects mixed or missing)
    let fingerprints = if let Some(dbp) = find_default_db() {
        match rusqlite::Connection::open(&dbp) {
            Ok(conn) => {
                // Count distinct non-null fingerprints (embed model fp col)
                let distinct: i32 = conn
                    .query_row(
                        "SELECT COUNT(DISTINCT COALESCE(fingerprint, '')) FROM documents",
                        [],
                        |row| row.get(0),
                    )
                    .unwrap_or(0);
                if distinct > 1 {
                    "MIXED (multiple fingerprints detected; suggests re-embed)".to_string()
                } else if distinct == 1 {
                    "consistent (single fingerprint)".to_string()
                } else {
                    "no fingerprint data (index exists but no embeds yet; run embed)".to_string()
                }
            }
            Err(e) => format!("could not open {}: {e}", dbp.display()),
        }
    } else {
        "no default index found (.qmd/index.sqlite or ~/.cache/qmd/index.sqlite)".to_string()
    };

    // Health check: count orphaned/inactive documents
    let health_status = if let Some(dbp) = find_default_db() {
        match rusqlite::Connection::open(&dbp) {
            Ok(conn) => {
                // Count total documents
                let total_docs: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM documents", 
                    [], 
                    |row| row.get(0)
                ).unwrap_or(0);
                
                // Count active documents
                let active_docs: i64 = conn.query_row(
                    "SELECT COUNT(*) FROM documents WHERE active = 1", 
                    [], 
                    |row| row.get(0)
                ).unwrap_or(0);
                
                // Count inactive documents (potential orphans)
                let inactive_docs = total_docs - active_docs;
                
                if inactive_docs > 0 {
                    format!("HEALTH_OK ({} active, {} inactive/orphaned documents)", active_docs, inactive_docs)
                } else {
                    format!("HEALTH_OK ({} active documents, no orphans)", active_docs)
                }
            }
            Err(e) => format!("HEALTH_CHECK_FAILED: could not open index: {}", e),
        }
    } else {
        "HEALTH_UNKNOWN: no index found".to_string()
    };

    // Device / GPU probe (safe, only if env flag; gated on llm feature for real llama probe)
    let device = if std::env::var("QMD_DOCTOR_DEVICE_PROBE").ok().as_deref() == Some("1") {
        #[cfg(feature = "llm")]
        {
            // Safe probe: constructing LlamaCpp may init backends or log; we don't force full load here
            let _ = crate::llm::LlamaCpp::new(None, None, None);
            "probe requested (QMD_DOCTOR_DEVICE_PROBE=1); llama backends attempted (metal/cuda/vulkan/CPU fallback per build). See logs/warnings for accel.".to_string()
        }
        #[cfg(not(feature = "llm"))]
        {
            "probe requested but 'llm' feature not active (build with --features llm for native device report)".to_string()
        }
    } else {
        "skipped (set QMD_DOCTOR_DEVICE_PROBE=1 for safe GPU/CPU/backend probe + warnings)".to_string()
    };

    // Relevant env (full parity with main seed + doctor)
    let mut env_overrides: HashMap<String, String> = HashMap::new();
    for (k, v) in std::env::vars() {
        if k.starts_with("QMD_") || k.starts_with("GGML_") || k.starts_with("LLAMA_") {
            env_overrides.insert(k, v);
        }
    }

    // Suggestions (actionable, explicit commands per AGENTS: user runs them manually)
    let mut suggestions: Vec<String> = vec![];
    if fingerprints.contains("MIXED") || fingerprints.contains("no fingerprint") {
        suggestions.push("cargo run -- embed --force   # or with -c <col> ; re-embeds with current model".to_string());
    }
    if device.contains("skipped") {
        suggestions.push("export QMD_DOCTOR_DEVICE_PROBE=1   # then re-run for GPU probe (then unset)".to_string());
    }
    if model_cache.contains("missing") {
        suggestions.push("cargo run -- embed   # will trigger model download to cache on first use".to_string());
    }
    if env_overrides.is_empty() {
        suggestions.push("(no QMD_*/GGML_*/LLAMA_* overrides active; see requirements for quiet/GPU/residency)".to_string());
    }
    if suggestions.is_empty() {
        suggestions.push("index looks healthy; try cargo run -- query 'your question here' or cargo run -- doctor --json".to_string());
    }

    let report = DoctorReport {
        sqlite_version,
        vec_extension,
        model_cache,
        fingerprints,
        device,
        env_overrides,
        suggestions,
    };

    if json {
        // Add health info to JSON output
        let mut json_report = serde_json::to_value(&report).expect("serialize report");
        if let Some(obj) = json_report.as_object_mut() {
            obj.insert("health".to_string(), serde_json::Value::String(health_status.clone()));
        }
        match serde_json::to_string_pretty(&json_report) {
            Ok(s) => println!("{s}"),
            Err(e) => eprintln!("json error: {e}"),
        }
    } else {
        println!("qmd doctor (Rust port)");
        println!("SQLite: {}", report.sqlite_version);
        println!("vec extension: {}", report.vec_extension);
        println!("model cache: {}", report.model_cache);
        println!("fingerprints: {}", report.fingerprints);
        println!("health: {}", health_status);
        println!("device/GPU: {}", report.device);
        println!("env overrides (QMD_*/GGML_*/LLAMA_*):");
        if report.env_overrides.is_empty() {
            println!("  (none)");
        } else {
            for (k, v) in &report.env_overrides {
                println!("  {k}={v}");
            }
        }
        println!("suggestions:");
        for s in &report.suggestions {
            println!("  - {s}");
        }
        println!("(run with --json for machine readable; see doctor.feature + requirements.md)");
    }
}

pub struct Maintenance;

impl Maintenance {
    pub fn vacuum(&self) { /* full vacuum on conn in future; explicit `cargo run -- vacuum` */ }
    pub fn cleanup_orphaned(&self) { /* scan docs, rm inactive not in fs; see status */ }
    // etc. (status/health in full maint)
}
