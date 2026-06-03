//! Paths, qmd:// URIs, docid (first 6 of content hash), fidelity (verbatim store/roundtrip for special chars, case, unicode).
//! handelize inverse from history (recent fixes require exact paths in index/search/get/ls).
//! See original src/paths.ts , path-fidelity.test.ts , recent changelog fixes, specs path_fidelity.feature .

use sha2::{Sha256, Digest};

pub fn make_docid(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    let hex = format!("{:x}", result);
    format!("#{}", &hex[0..6])
}

pub fn to_qmd_uri(collection: &str, path: &str) -> String {
    format!("qmd://{}/{}", collection, path.trim_start_matches('/'))
}

/// For full-path: if under PWD use ./rel else abs realpath (as in 2.5+).
pub fn display_path_for_output(fs_path: &str, pwd: &str) -> String {
    // TODO: real impl with std::fs::canonicalize, strip prefix, add ./
    if fs_path.starts_with(pwd) {
        format!("./{}", fs_path.strip_prefix(pwd).unwrap_or(fs_path).trim_start_matches('/'))
    } else {
        fs_path.to_string()
    }
}

// TODO: parse qmd:// , fuzzy suggestions for get (from index), NFC normalize on mac, case preserve, special char roundtrip (no mangling).
// Fidelity test: paths with # & [ ] ( ) space . emoji must survive index -> search -> get exactly.

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
}