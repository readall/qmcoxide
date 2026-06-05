# Score Normalization & Fusion (must match original exactly for parity)

From original README "Score Normalization & Fusion" section + architecture diagram + explain traces.

## Backends
- FTS (BM25 via FTS5): `Math.abs(score)` (0 to ~25+)
- Vector (cosine distance via vec0): `1 / (1 + distance)` (0.0-1.0)
- Reranker (LLM 0-10 rating / logprobs): `score / 10` (0.0-1.0)

## Pipeline (query / hybrid)
1. Original query (×2 weight) + 1 LLM expansion (or user-provided lex/vec/hyde lines).
2. For each: parallel FTS + Vector.
3. RRF: `score = Σ(1/(k + rank + 1))` k=60 across all lists.
4. Top-rank bonus: #1 in any list +0.05, #2-3 +0.02.
5. Keep top 30 candidates.
6. LLM rerank (yes/no + confidence).
7. Position-aware blend:
   - RRF 1-3: 75% retrieval / 25% reranker
   - 4-10: 60/40
   - 11+: 40/60
   (preserves exact matches from strong original query while trusting reranker more for lower ranks)

## Intent effects
- Prepended to expansion and rerank.
- 0.5x weight in chunk selection.
- 0.3x in snippet.
- Disables "strong signal" BM25 early exit.

## Explain
--explain / MCP must surface per-backend scores, RRF contribs, bonuses, reranker, final blended.

Port must replicate math, ordering, and trace shape for --explain and agent use.

See also plan.md and original README for the full ASCII diagram.
