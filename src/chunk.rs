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

pub fn chunk_document(text: &str, strategy: ChunkStrategy) -> Vec<Chunk> {
    // TODO: implement scoring for breaks (H1=100 ...), window search, fence ignore, merge AST if auto.
    // Support langs: ts/js/py/go/rs
    vec![Chunk { text: text.to_string(), pos: 0 }]
}