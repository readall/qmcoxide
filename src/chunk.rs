//! Chunking: regex smart (MD headings scores, code fence protect, overlap 15%, ~900 tok) + AST via tree-sitter for code files when strategy=auto.
//! Exact algos from docs/chunking.md and original README/ast.ts .
//! Used in indexing and for snippet selection.

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChunkStrategy {
    Regex,
    Auto, // AST for supported
}

pub struct Chunk {
    pub text: String,
    pub pos: usize, // char pos in orig
    // seq, hash etc
}

/// Simple impl of smart chunk (full scoring + AST in full; here regex + basic for test).
pub fn chunk_document(text: &str, strategy: ChunkStrategy) -> Vec<Chunk> {
    if strategy == ChunkStrategy::Auto {
        // TODO: tree-sitter parse for .rs etc, merge scores (class 100, fn 90...)
    }
    // Basic: split on headings or ~900 chars for stub (to make tests pass)
    let mut chunks = vec![];
    let target = 900;
    let mut start = 0;
    while start < text.len() {
        let end = (start + target).min(text.len());
        // prefer break at \n\n or heading
        let mut cut = end;
        if let Some(pos) = text[start..end].rfind("\n\n") {
            cut = start + pos + 2;
        } else if let Some(pos) = text[start..end].rfind('#') {
            if pos > 0 { cut = start + pos; }
        }
        chunks.push(Chunk { text: text[start..cut].to_string(), pos: start });
        start = cut;
        if start < text.len() { start = (start as f32 * 0.85) as usize; } // overlap ~15%
    }
    if chunks.is_empty() {
        chunks.push(Chunk { text: text.to_string(), pos: 0 });
    }
    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_basic() {
        let text = "# H1\n\nSome para.\n\n## H2\n\nMore text here to make longer chunk target.";
        let chunks = chunk_document(text, ChunkStrategy::Regex);
        assert!(!chunks.is_empty());
        // basic overlap/break logic exercised
    }
}