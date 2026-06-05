# Chunking Algorithm (exact rules for fidelity)

Target ~900 tokens per chunk, 15% overlap.

## Regex (default, always for MD and unknown)
Break point scores (higher better):
- # H1: 100
- ## H2: 90
- ### H3: 80
- #### H4: 70
- ##### H5: 60
- ###### H6: 50
- ``` code fence: 80
- --- / ***: 60
- blank line: 20
- - item / 1. item: 5
- line break: 1

Algorithm:
- Scan all candidate breaks with scores.
- Near 900 tok target, look back in 200 tok window.
- finalScore = base * (1 - (dist/window)^2 * 0.7)
- Cut at highest.
- Code fences: ignore breaks inside ``` blocks (keep code together; if block > chunk, keep whole when possible).

## AST-aware (--chunk-strategy auto, for .ts .tsx .js .jsx .py .go .rs)
Merge with regex scores:
- class/interface/struct/impl/trait: 100
- fn/method: 90
- type alias/enum: 80
- import/use: 60

Uses tree-sitter (same grammars as original). Falls back to regex if grammars unavailable.

Port must produce identical or equivalent chunk boundaries and pos for the same input (for vector parity and snippet lines).

See original README "Smart Chunking" + ast.ts + ast*.test.ts + test/eval for exact.
