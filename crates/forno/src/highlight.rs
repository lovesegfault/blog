//! Syntax highlighting, with tree-sitter.

use std::cmp::Reverse;
use std::sync::{LazyLock, OnceLock};

use arborium_highlight::{CompiledGrammar, GrammarConfig, Injection, ParseContext};

/// A class in the stylesheet, or nothing for plain text.
pub type Ink = &'static str;

pub const COMMENT: Ink = "c";
const PUNCTUATION: Ink = "o";

pub struct Language {
    pub label: &'static str,
    names: &'static [&'static str],
    /// The glaze of the tile beside its name.
    pub glaze: &'static str,
    /// Whether all its punctuation is syntax, which some grammars leave unmarked.
    punctuated: bool,
    compile: fn() -> CompiledGrammar,
    grammar: OnceLock<CompiledGrammar>,
}

macro_rules! languages {
    ($($grammar:ident, $label:literal, $names:expr, $glaze:literal, $punctuated:literal, $extra:expr;)*) => {
        vec![$(Language {
            label: $label,
            names: &$names,
            glaze: $glaze,
            punctuated: $punctuated,
            compile: || {
                CompiledGrammar::new(GrammarConfig {
                    language: $grammar::language().into(),
                    highlights_query: &format!("{}\n{}\n(ERROR) @error", &*$grammar::HIGHLIGHTS_QUERY, $extra),
                    injections_query: $grammar::INJECTIONS_QUERY,
                    locals_query: "",
                })
                .expect(concat!("the queries for ", $label, " compile"))
            },
            grammar: OnceLock::new(),
        }),*]
    };
}

const C: &str = r##"
    (escape_sequence) @escape
    [(true) (false)] @constant
    ["goto" "register" "restrict"] @keyword
    ["#define" "#elif" "#else" "#endif" "#if" "#ifdef" "#ifndef" "#include" (preproc_directive)] @preproc
"##;
const NIX: &str = r#"
    (binding attrpath: (attrpath (identifier) @key))
    (inherited_attrs attr: (identifier) @key)
    ((identifier) @constant (#any-of? @constant "true" "false" "null"))
"#;
const RUST: &str = r#"
    ((identifier) @constant (#match? @constant "^[A-Z][A-Z\\d_]+$"))
    (call_expression function: (identifier) @type (#match? @type "^[A-Z]"))
    (call_expression function: (scoped_identifier name: (identifier) @type) (#match? @type "^[A-Z]"))
    (lifetime "'" @label)
"#;
const SHELL: &str = r#"
    (variable_name) @key
    ["||" ";" "&" "=" "$(" "${" "(" ")" "{" "}"] @operator
    ["local" "readonly" "declare"] @keyword
"#;
const YAML: &str = r"
    (_ key: (flow_node (plain_scalar (string_scalar) @key)))
    (_ key: (flow_node [(double_quote_scalar) (single_quote_scalar)] @key))
";

static LANGUAGES: LazyLock<Vec<Language>> = LazyLock::new(|| {
    languages![
        arborium_asm, "assembly", ["asm", "s", "armasm"], "slate", false, "";
        arborium_bash, "shell", ["bash", "sh", "shell", "zsh", "console"], "moss", false, SHELL;
        arborium_c, "C", ["c", "h"], "cobalt", true, C;
        arborium_cpp, "C++", ["cpp", "c++", "cc", "hpp"], "cobalt", true, C;
        arborium_css, "CSS", ["css"], "salmon", false, "(property_name) @key";
        arborium_dockerfile, "Dockerfile", ["dockerfile"], "slate", false, "";
        arborium_go, "Go", ["go"], "slate", true, "";
        arborium_haskell, "Haskell", ["haskell", "hs"], "salmon", false, "";
        arborium_html, "HTML", ["html", "xml", "svg"], "sienna", false, "(attribute_name) @key";
        arborium_ini, "INI", ["ini", "systemd"], "olive", false, "(setting_name) @key";
        arborium_javascript, "JavaScript", ["javascript", "js"], "ochre", false, "";
        arborium_json, "JSON", ["json"], "olive", true, "(pair key: (_) @key)";
        arborium_lua, "Lua", ["lua"], "cobalt", false, "";
        arborium_nix, "Nix", ["nix"], "slate", false, NIX;
        arborium_python, "Python", ["python", "py"], "ochre", true, "";
        arborium_rust, "Rust", ["rust", "rs"], "sienna", true, RUST;
        arborium_toml, "TOML", ["toml"], "olive", false, "[(bare_key) (quoted_key)] @key";
        arborium_typescript, "TypeScript", ["typescript", "ts"], "cobalt", false, "";
        arborium_typst, "Typst", ["typst", "typ"], "cream", false, "";
        arborium_x86asm, "x86", ["x86", "x86asm", "nasm"], "slate", false, "";
        arborium_yaml, "YAML", ["yaml", "yml"], "olive", false, YAML;
        arborium_zig, "Zig", ["zig"], "ochre", false, "";
    ]
});

pub fn language(name: &str) -> Option<&'static Language> {
    LANGUAGES.iter().find(|language| language.names.iter().any(|known| known.eq_ignore_ascii_case(name)))
}

fn ink(capture: &str) -> Option<Ink> {
    Some(match capture.split_once('.').unwrap_or((capture, "")) {
        ("spell" | "error", _) => return None,
        ("attribute" | "preproc" | "label", _) | ("keyword", "directive") | ("function", "macro" | "special") => "p",
        ("keyword" | "conditional" | "repeat" | "include" | "exception" | "storageclass", _) => "k",
        ("type", "qualifier") | ("markup", "heading.marker" | "list") => "k",
        ("type" | "constructor" | "module" | "tag" | "key", _) | ("variable", "builtin") => "t",
        ("function" | "method", _) => "f",
        ("number" | "float" | "boolean" | "constant" | "escape", _) | ("string", "escape") => "n",
        ("string" | "character", _) | ("markup", "raw.block") => "s",
        ("comment", _) => COMMENT,
        ("operator" | "punctuation" | "delimiter", _) => PUNCTUATION,
        _ => "",
    })
}

impl Language {
    /// The ink for each byte of `code`.
    pub fn paint(&self, code: &str) -> Vec<Ink> {
        let (mut inks, mut braced) = (vec![""; code.len()], vec![""; code.len() + 3]);
        // An excerpt is often the inside of something.
        if !self.coat(code, &mut inks, 0) && self.coat(&format!("{{{code}\n}}"), &mut braced, 0) {
            inks.copy_from_slice(&braced[1..=code.len()]);
        }
        for (byte, ink) in code.bytes().zip(&mut inks) {
            if self.punctuated && ink.is_empty() && byte.is_ascii_punctuation() && byte != b'_' {
                *ink = PUNCTUATION;
            }
        }
        inks
    }

    /// Returns whether the code made sense.
    fn coat(&self, code: &str, inks: &mut [Ink], depth: usize) -> bool {
        let grammar = self.grammar.get_or_init(self.compile);
        let parsed = grammar.parse(&mut ParseContext::for_grammar(grammar).expect("the grammar loads"), code);
        let mut spans: Vec<_> = parsed.spans.iter().filter_map(|span| Some((span, ink(&span.capture)?))).collect();
        // Inner over outer, coloured over plain, and then later patterns over earlier ones.
        spans.sort_by_key(|(span, ink)| (Reverse(span.end - span.start), !ink.is_empty(), span.pattern_index));
        for (span, ink) in spans {
            inks[span.start as usize..span.end as usize].fill(ink);
        }
        // Nix names the language in a comment, which goes before what is only a guess.
        let known = |inner: &Injection| language(inner.language.trim_matches(['/', '*', '#', ' ']));
        let mut inner: Vec<_> = parsed.injections.iter().filter(|inner| depth < 3 && known(inner).is_some()).collect();
        inner.sort_by_key(|inner| (inner.start, &inner.language));
        inner.dedup_by_key(|inner| inner.start);
        // What is spliced into a string does not end it.
        let spliced = |gap: &str| gap.len() < 5 && gap.starts_with("''") || gap.starts_with("${") && gap.ends_with('}');
        for parts in inner.chunk_by(|a, b| code.get(a.end as usize..b.start as usize).is_some_and(spliced)) {
            let (start, end) = (parts[0].start as usize, parts[parts.len() - 1].end as usize);
            let (code, inks) = (&code[start..end], &mut inks[start..end]);
            let (mut whole, mut coat) = ("x".repeat(code.len()), inks.to_vec());
            let ranges = parts.iter().map(|part| part.start as usize - start..part.end as usize - start);
            ranges.clone().for_each(|range| whole.replace_range(range.clone(), &code[range]));
            known(parts[0]).unwrap().coat(&whole, &mut coat, depth + 1);
            ranges.for_each(|range| inks[range.clone()].copy_from_slice(&coat[range]));
        }
        parsed.spans.iter().all(|span| span.capture != "error")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn painted(name: &str, code: &str) -> String {
        language(name).unwrap().paint(code).iter().map(|ink| if ink.is_empty() { "." } else { ink }).collect()
    }

    #[test]
    fn every_language_loads_and_has_a_glaze() {
        for language in &*LANGUAGES {
            language.paint("x");
            assert!(mural::LABELS.iter().any(|(glaze, _)| *glaze == language.glaze));
        }
    }

    #[test]
    fn the_most_specific_ink_wins() {
        assert_eq!(painted("C", r#"f("\n", N);"#), "fosnnso.noo");
        assert_eq!(painted("nix", "{ a = f b.c; }"), "o.t.o.f..o.o.o");
        assert_eq!(painted("json", r#"{"a":"b"}"#), "otttossso");
    }

    #[test]
    fn excerpts_parse() {
        assert_eq!(painted("nix", "a.b = f c;"), "tot.o.f..o");
    }

    #[test]
    fn languages_nest() {
        assert_eq!(painted("rust", "m!(f(1))"), "ppofonoo");
        assert!(painted("nix", r#"{ script = ''if "${a}"; then b; fi''; }"#).ends_with("kkkksfoskksso.o"));
    }
}
