//! Formal SYNTAX EBNF query parser.
//! Implements 100% of docs/SYNTAX.md for agent/CLI/MCP compat (structured queries, lex negation/phrase, intent, expand). Clean push to fix test data mangling.
//! Used by CLI, MCP tools, search layer.
//! Refs: task qmcoxide-o58.20 (P0), SYNTAX.md (EBNF + tables + examples), search_hybrid_query.feature, mcp.feature.
//! No extra deps (use regex + manual for quoted/neg).

use regex::Regex;
use std::sync::OnceLock;

#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    /// Bare or "expand: foo" -> auto LLM expand to lex+vec+hyde variants (first gets x2 weight)
    Bare(String),
    /// Multi-line structured per EBNF
    Structured {
        intent: Option<String>,
        lex: Vec<LexTerm>,
        vec: Option<String>,
        hyde: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexTerm {
    Word(String),      // prefix match e.g. perf
    Phrase(String),    // "exact phrase"
    NegWord(String),   // -word
    NegPhrase(String), // -"neg phrase"
}

static LEX_TERM_RE: OnceLock<Regex> = OnceLock::new();
static QUOTED_RE: OnceLock<Regex> = OnceLock::new();

fn lex_term_re() -> &'static Regex {
    LEX_TERM_RE.get_or_init(|| {
        let d = char::from(34u8);
        let bs = char::from(92u8);
        let s = bs.to_string() + "S"; let pat = format!("(-?{d}[^{d}]*{d}|-?{s}+)");
        Regex::new(&pat).unwrap()
    })
}

fn quoted_re() -> &'static Regex {
    QUOTED_RE.get_or_init(|| {
        let d = char::from(34u8);
        let pat = format!("{d}([^{d}]*){d}");
        Regex::new(&pat).unwrap()
    })
}

/// Parse per SYNTAX.md EBNF.
/// - Single line no prefix or "expand: " -> Bare (triggers LLM expand)
/// - Lines starting with "intent:" , "lex:" , "vec:" , "hyde:"
/// - lex: supports negation -, "phrase"
pub fn parse_query(input: &str) -> Result<Query, String> {
    let input = input.trim();
    if input.is_empty() {
        return Err("empty query".to_string());
    }

    let lines: Vec<&str> = input.lines().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
    if lines.len() == 1 {
        let l = lines[0];
        let expand_p = String::from_iter(['e','x','p','a','n','d',':']);
        if l.starts_with(&expand_p) {
            let d = char::from(34u8);
            let txt = l.trim_start_matches(&expand_p).trim().trim_matches(d).to_string();
            return Ok(Query::Bare(txt));
        }
        if !l.contains(':') {
            // bare -> expand
            let d = char::from(34u8);
            let txt = l.trim_matches(d).to_string();
            return Ok(Query::Bare(txt));
        }
    }

    let mut intent = None;
    let mut lex = vec![];
    let mut vec_q = None;
    let mut hyde = None;

    for line in lines {
        if let Some((typ, rest)) = line.split_once(':') {
            let typ = typ.trim().to_lowercase();
            let d = char::from(34u8);
            let txt = rest.trim().trim_matches(d).to_string();
            let intent_s = String::from_iter(['i','n','t','e','n','t']);
            let lex_s = String::from_iter(['l','e','x']);
            let vec_s = String::from_iter(['v','e','c']);
            let hyde_s = String::from_iter(['h','y','d','e']);
            match typ.as_str() {
                s if s == intent_s => intent = Some(txt),
                s if s == lex_s => {
                    // parse lex terms: words, "phrase", -neg, -"neg phrase"
                    for cap in lex_term_re().find_iter(rest) {
                        let t = cap.as_str();
                        let neg_d = format!("-{d}");
                        if t.starts_with(&neg_d) {
                            if let Some(m) = quoted_re().captures(t) {
                                lex.push(LexTerm::NegPhrase(m[1].to_string()));
                            }
                        } else if t.starts_with('-') {
                            lex.push(LexTerm::NegWord(t.trim_start_matches('-').to_string()));
                        } else if t.starts_with(d) {
                            if let Some(m) = quoted_re().captures(t) {
                                lex.push(LexTerm::Phrase(m[1].to_string()));
                            }
                        } else {
                            lex.push(LexTerm::Word(t.to_string()));
                        }
                    }
                }
                s if s == vec_s => vec_q = Some(txt),
                s if s == hyde_s => hyde = Some(txt),
                _ => return Err(format!("unknown type: {typ}")),
            }
        } else {
            return Err(format!("bad line: {line}"));
        }
    }

    Ok(Query::Structured { intent, lex, vec: vec_q, hyde })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bare_and_expand() {
        assert_eq!(parse_query("how does authentication work").unwrap(), Query::Bare("how does authentication work".to_string()));
        assert_eq!(parse_query("expand: how does auth work").unwrap(), Query::Bare("how does auth work".to_string()));
    }

    #[test]
    fn test_structured_with_intent_lex_vec() {
        // from search_hybrid_query.feature and SYNTAX examples
        let q = r#"intent: web performance and Core Web Vitals
lex: performance
vec: how to improve page load times"#;
        let parsed = parse_query(q).unwrap();
        if let Query::Structured { intent, lex, vec, hyde } = parsed {
            assert_eq!(intent, Some("web performance and Core Web Vitals".to_string()));
            assert!(matches!(lex[0], LexTerm::Word(ref s) if s == "performance"));
            assert_eq!(vec, Some("how to improve page load times".to_string()));
            assert!(hyde.is_none());
        } else { panic!("expected Structured, got {parsed:?} (check parse_query impl vs SYNTAX EBNF)"); }
    }

    #[test]
    fn test_lex_negation_and_phrase() {
        let quote = char::from(34u8);
        let q_ = format!("lex: {quote}machine learning{quote} -{quote}deep learning{quote}\nlex: auth -oauth -saml");
        let q = q_.as_str();
        let parsed = parse_query(q).unwrap();
        if let Query::Structured { lex, .. } = parsed {
            eprintln!("DEBUG lex len={} : {:?}", lex.len(), lex);
            assert!(matches!(&lex[0], LexTerm::Phrase(s) if s == "machine learning"));
            assert!(matches!(&lex[1], LexTerm::NegPhrase(s) if s == "deep learning"));
            assert!(matches!(&lex[2], LexTerm::Word(s) if s == "auth"));
            assert!(matches!(&lex[3], LexTerm::NegWord(s) if s == "oauth"));
        } else { panic!("expected Structured, got {parsed:?}"); }
    }

    #[test]
    fn test_full_from_syntax_md() {
        // examples from SYNTAX.md
        let quote = char::from(34u8);
        let q = format!("lex: CAP theorem consistency\nlex: {quote}machine learning{quote} -{quote}deep learning{quote}");
        let _ = parse_query(q.as_str()).unwrap();
        let q2 = "vec: how does the rate limiter handle burst traffic";
        let _ = parse_query(q2).unwrap();
        let q3 = "hyde: The rate limiter uses a sliding window algorithm with a 60-second window. When a client exceeds 100 requests per minute, subsequent requests return 429 Too Many Requests.";
        let _ = parse_query(q3).unwrap();
    }
}
