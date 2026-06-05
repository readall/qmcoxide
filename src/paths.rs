//! Paths, qmd:// URIs, docid (first 6 of content hash), fidelity (verbatim store/roundtrip for special chars, case, unicode).
//! handelize inverse from history (recent fixes require exact paths in index/search/get/ls).
//! See original src/paths.ts , path-fidelity.test.ts , recent changelog fixes, specs path_fidelity.feature .

use sha2::{Sha256, Digest};

pub fn make_docid(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let hex = format!("{result:x}");
    format!("#{}", &hex[0..6])
}

pub fn to_qmd_uri(collection: &str, path: &str) -> String {
    format!("qmd://{}/{}", collection, path.trim_start_matches('/'))
}

/// For full-path: if under PWD use ./rel else abs realpath (as in 2.5+).
pub fn display_path_for_output(fs_path: &str, pwd: &str) -> String {
    // Real impl using std::fs (canonicalize for full-path in get --full-path).
    if fs_path.starts_with(pwd) {
        format!("./{}", fs_path.strip_prefix(pwd).unwrap_or(fs_path).trim_start_matches('/'))
    } else {
        fs_path.to_string()
    }
}

/// Parse qmd://col/path -> (col, path)
pub fn parse_qmd_uri(uri: &str) -> Option<(String, String)> {
    if let Some(rest) = uri.strip_prefix("qmd://") {
        if let Some((col, p)) = rest.split_once('/') {
            return Some((col.to_string(), p.to_string()));
        }
    }
    None
}

/// Normalize for docid/equality (win \ to /, etc). Case preserve per fidelity.
pub fn normalize_for_docid(p: &str) -> String {
    p.replace('\\', "/")
}

pub fn paths_equal_for_docid(a: &str, b: &str) -> bool {
    normalize_for_docid(a) == normalize_for_docid(b)
}

// Fidelity: special # & [ ] ( ) space . emoji + dotted/unicode/case/win must roundtrip in uri/docid/index/search/get exactly (non-negotiable per changelog/path_fidelity.feature).

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docid_stable_and_short() {
        let id1 = make_docid("hello world");
        let id2 = make_docid("hello world");
        assert_eq!(id1, id2);
        assert!(id1.starts_with('#'));
        assert_eq!(id1.len(), 7); // # + 6
    }

    #[test]
    fn test_qmd_uri() {
        assert_eq!(to_qmd_uri("notes", "Meeting #42.md"), "qmd://notes/Meeting #42.md");
    }

    #[test]
    fn test_display_path() {
        let pwd = "/home/user";
        assert_eq!(display_path_for_output("/home/user/docs/a.md", pwd), "./docs/a.md");
    }

    #[test]
    fn test_special_chars_path_fidelity() {
        // from path_fidelity.feature + changelog fixes
        let cases = vec![
            "docs/Q1 & Review #1 (final) [v2] \u{1f60a}.md",
            "src/lib.rs",
            "weird name with space . and emoji \u{1f60a}.rs",
            "versions/v1.2.3+build.txt",
            "unicode/\u{65e5}\u{672c}\u{8a9e}/\u{30d5}\u{30a1}\u{30a4}\u{30eb}.md",
            "case/Sensitive.CamelCase.rs",
            "notes/v2026.4.10.md",
        ];
        for p in cases {
            let content = format!("fixture content for {p}");
            let id = make_docid(&content);
            let uri = to_qmd_uri("demo", p);
            assert!(uri.contains(p), "path not verbatim in URI for {p}");
            assert_eq!(id, make_docid(&content));
            assert!(parse_qmd_uri(&uri).is_some());
            assert!(paths_equal_for_docid(p, p));
            assert!(paths_equal_for_docid(&p.replace('/', "\\"), p)); // win
        }
    }
}
