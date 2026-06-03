//! Paths, qmd:// URIs, docid (first 6 of content hash), fidelity (verbatim store/roundtrip for special chars, case, unicode).
//! handelize inverse from history.
//! See original src/paths.ts , path-fidelity.test.ts , recent changelog fixes.

pub fn make_docid(content: &str) -> String {
    // TODO: sha256 or equiv, take first 6 hex chars, with #
    format!("#abc123")
}

pub fn to_qmd_uri(collection: &str, path: &str) -> String {
    format!("qmd://{}/{}", collection, path)
}

// TODO: parse, fuzzy match suggestions, full_path handling (./ under PWD or abs realpath)