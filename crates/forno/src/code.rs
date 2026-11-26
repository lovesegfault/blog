//! Code blocks.

use std::ops::{Range, RangeInclusive};

use anyhow::{Context, Result, bail, ensure};
use maud::{Markup, html};
use typst::foundations::{OneOrMultiple, Smart};

use crate::highlight::{self, COMMENT, Ink, Language};
use crate::tree::Node;

type Lines = Vec<RangeInclusive<usize>>;

/// What `code` takes, in `blog.typ`.
#[derive(Default)]
pub struct Options {
    pub title: Option<String>,
    /// Whether it is a whole file.
    pub whole: bool,
    highlight: Lines,
    mark: Vec<String>,
    fold: Lines,
    lines: Lines,
    start: Option<usize>,
    numbers: Smart<bool>,
    diff: bool,
    wrap: bool,
    run: Option<String>,
}

#[derive(Clone, Copy, PartialEq)]
enum Role {
    Code,
    Added,
    Removed,
    Command,
    Continued,
    Output,
}

struct Line<'a> {
    text: &'a str,
    role: Role,
    number: usize,
    note: Option<usize>,
    inks: Vec<Ink>,
}

pub struct Listing<'a> {
    options: &'a Options,
    language: Option<&'static Language>,
    patch: bool,
    /// What a program printed, rather than a program.
    transcript: bool,
    lines: Vec<Line<'a>>,
}

/// How many columns fit beside the text.
const COLUMNS: usize = 80;

/// Compiler Explorer knows these by the same names.
const RUNNABLE: [&str; 7] = ["C", "C++", "Go", "Haskell", "Python", "Rust", "Zig"];

/// As in `3,5-7`.
fn ranges(text: &str) -> Result<Lines> {
    let range = |text: &str| {
        let (first, last) = text.split_once('-').unwrap_or((text, text));
        let lines = first.parse().and_then(|first| Ok(first..=last.parse()?)).ok().filter(|lines| !lines.is_empty());
        lines.with_context(|| format!("`{text}` is not a line or a range of lines"))
    };
    text.split([',', ' ']).filter(|part| !part.is_empty()).map(range).collect()
}

impl Options {
    pub fn new(node: &Node) -> Result<Self> {
        let lines = |name| ranges(&node.option::<String>(name)?);
        Ok(Self {
            title: node.option("title")?,
            whole: node.option("whole")?,
            highlight: lines("hl")?,
            mark: node.option::<OneOrMultiple<_>>("mark")?.0,
            fold: lines("fold")?,
            lines: lines("lines")?,
            start: node.option("start")?,
            numbers: node.option("numbers")?,
            diff: node.option("diff")?,
            wrap: node.option("wrap")?,
            run: node.option("run")?,
        })
    }
}

impl Line<'_> {
    fn among(&self, lines: &Lines) -> bool {
        lines.iter().any(|lines| lines.contains(&self.number))
    }
}

/// Splits the name of a note off the end of a line, with the comment it was in if nothing else is left of it.
fn note<'a>(text: &'a str, names: &[String]) -> (&'a str, Option<usize>) {
    const LEADERS: &[&str] = &["///", "//", "#", "--", ";;", ";", "%"];
    for (open, close) in [(&["/*"][..], "*/"), (&["<!--"], "-->"), (LEADERS, ""), (&[""], "")] {
        let reference = text.trim_end().strip_suffix(close).and_then(|text| text.trim_end().strip_suffix('>'));
        if let Some((before, name)) = reference.and_then(|text| text.rsplit_once(" <"))
            && let Some(note) = names.iter().position(|known| known == name)
            && let Some(code) = open.iter().find_map(|open| before.trim_end().strip_suffix(open))
            && (open != LEADERS || !code.ends_with(|c: char| !c.is_whitespace()))
        {
            return (code.trim_end(), Some(note));
        }
    }
    (text, None)
}

/// The first address in a line, if it is in a comment.
fn link(text: &str, inks: &[Ink]) -> Option<Range<usize>> {
    let start = text.find("https://").or(text.find("http://")).filter(|&start| inks[start] == COMMENT)?;
    let address = text[start..].split([' ', '\t', '"', '<', '>', '`']).next()?;
    Some(start..start + address.trim_end_matches(['.', ',', ';', ':', ')', '\'']).len())
}

impl<'a> Listing<'a> {
    pub fn new(name: &str, options: &'a Options, code: &'a str, notes: &[String]) -> Result<Self> {
        let is = |names: &[&str]| names.contains(&&*name.to_lowercase());
        let (patch, console, transcript) =
            (is(&["diff", "patch"]), is(&["console"]), is(&["", "text", "txt", "console"]));
        let language = highlight::language(name);
        ensure!(
            language.is_some() || patch || transcript,
            "there is no grammar for `{name}`: use `text`, or add one to highlight.rs"
        );
        let runs = language.is_some_and(|language| RUNNABLE.contains(&language.label));
        ensure!(runs || options.run.is_none(), "`run` is for {}", RUNNABLE.join(", "));
        let mut number = options.start.unwrap_or(1);
        let mut lines: Vec<Line> = Vec::new();
        for text in code.lines() {
            let continued = lines.last().is_some_and(|last| last.role != Role::Output && last.text.ends_with('\\'));
            let header = ["+++ ", "--- ", "@@", "diff ", "index "].iter().any(|start| text.starts_with(start));
            let (role, text) = match text.split_at_checked(1).unwrap_or(("", text)) {
                _ if console && continued => (Role::Continued, text),
                ("$", rest) if console && !rest.starts_with(|c| c != ' ') => (Role::Command, rest.trim_start()),
                _ if console || patch && header => (Role::Output, text),
                ("+", _) if patch => (Role::Added, text),
                ("-", _) if patch => (Role::Removed, text),
                ("+", rest) if options.diff => (Role::Added, rest),
                ("-", rest) if options.diff => (Role::Removed, rest),
                (" ", rest) if options.diff => (Role::Code, rest),
                _ if options.diff && !text.is_empty() => bail!("lines in a diff start with `+`, `-` or a space"),
                _ => (Role::Code, text),
            };
            let (text, note) = note(text, notes);
            lines.push(Line { text, role, number, note, inks: vec![""; text.len()] });
            number += usize::from(role != Role::Removed || patch);
        }

        // The code as it was, and then as it is, so that both parse.
        for left_out in if options.diff { &[Role::Added, Role::Removed][..] } else { &[Role::Output] } {
            let Some(language) = language else { break };
            let side: Vec<&mut Line> = lines.iter_mut().filter(|line| line.role != *left_out).collect();
            let code: String = side.iter().flat_map(|line| [line.text, "\n"]).collect();
            let mut inks = language.paint(&code).into_iter();
            side.into_iter().for_each(|line| line.inks = inks.by_ref().take(line.text.len() + 1).collect());
        }

        if !options.lines.is_empty() {
            lines.retain(|line| line.among(&options.lines));
            let indent = |line: &Line| line.text.len() - line.text.trim_start_matches([' ', '\t']).len();
            let margin = lines.iter().filter(|line| !line.text.trim().is_empty()).map(indent).min().unwrap_or(0);
            for line in &mut lines {
                line.text = &line.text[line.inks.drain(..indent(line).min(margin)).len()..];
            }
        }
        for wanted in options.lines.iter().chain(&options.highlight).chain(&options.fold) {
            let missing = [wanted.start(), wanted.end()].into_iter().find(|n| lines.iter().all(|l| l.number != **n));
            ensure!(missing.is_none(), "there is no line {} here", missing.unwrap());
        }
        for mark in &options.mark {
            ensure!(lines.iter().any(|line| line.text.contains(mark)), "there is no `{mark}` to mark");
        }
        for (note, name) in notes.iter().enumerate() {
            ensure!(lines.iter().any(|line| line.note == Some(note)), "no line ends in `<{name}>`");
        }
        Ok(Self { options, language, patch, transcript, lines })
    }

    fn tokens(&self, line: &Line) -> Markup {
        let text = line.text;
        let link = link(text, &line.inks).unwrap_or_default();
        let marks = self.options.mark.iter().flat_map(|mark| text.match_indices(mark));
        let marks: Vec<_> = marks.map(|(start, mark)| start..start + mark.len()).collect();
        let marked = |at: &usize| marks.iter().any(|mark| mark.contains(at));
        let starts: Vec<usize> = text.char_indices().map(|(start, _)| start).collect();
        let inked = |starts: &[usize]| {
            let alike = |a: &usize, b: &usize| line.inks[*a] == line.inks[*b] && link.contains(a) == link.contains(b);
            html! {
                @for run in starts.chunk_by(alike) {
                    @let (ink, run_text) = (line.inks[run[0]], &text[run[0]..text.ceil_char_boundary(run[run.len() - 1] + 1)]);
                    @if link.contains(&run[0]) {
                        a class=(ink) href=(text[link.clone()]) { (run_text) }
                    } @else if ink.is_empty() {
                        (run_text)
                    } @else {
                        span class=(ink) { (run_text) }
                    }
                }
            }
        };
        html! {
            @for run in starts.chunk_by(|a, b| marked(a) == marked(b)) {
                @if marked(&run[0]) { mark { (inked(run)) } } @else { (inked(run)) }
            }
        }
    }

    /// `id` is the block's, `source` is where its file can be had, and `cite` makes the mark that leads to a note.
    pub fn render(&self, id: &str, source: Option<&str>, cite: &mut dyn FnMut(usize) -> Markup) -> Markup {
        let (options, patch) = (self.options, self.patch);
        let (wrap, numbered) = (options.wrap || self.transcript, options.numbers.unwrap_or(!patch && !self.transcript));
        let label = if patch { Some("diff") } else { self.language.map(|language| language.label) };
        let width = |line: &Line| line.text.chars().count() + 3 * line.text.matches('\t').count();
        let columns = self.lines.iter().map(width).max().unwrap_or(0);
        let wide = columns > COLUMNS && !wrap && self.lines.iter().all(|line| line.note.is_none());
        let digits = self.lines.last().map_or(0, |line| line.number.to_string().len()).max(2);

        let mut pre = |lines: &[Line]| {
            html! { pre { code { @for line in lines {
                @let anchor = (numbered && (line.role != Role::Removed || patch)).then(|| format!("{id}-L{}", line.number));
                span.l.lit[line.among(&options.highlight)].cmd[line.role == Role::Command].out[line.role == Role::Output]
                    .add[line.role == Role::Added].del[line.role == Role::Removed] id=[&anchor]
                {
                    @if let Some(anchor) = anchor {
                        a.at href={ "#" (anchor) } data-n=(line.number) tabindex="-1" aria-hidden="true" {}
                    } @else {
                        span.at {}
                    }
                    @match line.role {
                        Role::Added => ins { (self.tokens(line)) "\n" },
                        Role::Removed => del { (self.tokens(line)) "\n" },
                        _ => span { (self.tokens(line)) "\n" },
                    }
                    @if let Some(note) = line.note { (cite(note)) }
                }
            } } } }
        };
        html! {
            figure.code.wide[wide].wrap[wrap].patch[patch] id=(id)
                style={ "--columns:" (columns) ";--digits:" (digits) }
                data-language=[options.run.as_ref().and(label).map(str::to_lowercase)] data-options=[&options.run]
            {
                @if let Some(name) = options.title.as_deref().or(label) {
                    figcaption class=(self.language.map_or("cream", |language| language.glaze)) {
                        @if let Some(source) = source { a href=(source) { (name) } } @else { (name) }
                    }
                }
                div.lines {
                    @for part in self.lines.chunk_by(|a, b| a.among(&options.fold) == b.among(&options.fold)) {
                        @if part[0].among(&options.fold) {
                            details {
                                summary { (part.len()) @if part.len() == 1 { " line" } @else { " lines" } }
                                (pre(part))
                            }
                        } @else {
                            (pre(part))
                        }
                    }
                }
            }
        }
    }

    /// For where the stylesheet does not reach.
    pub fn portable(&self, cite: &mut dyn FnMut(usize) -> Markup) -> Markup {
        let sign = |role| match role {
            Role::Command => "$ ",
            _ if !self.options.diff => "",
            Role::Added => "+",
            Role::Removed => "-",
            _ => " ",
        };
        html! {
            @if let Some(title) = &self.options.title { p { i { (title) } } }
            pre { code { @for line in &self.lines {
                (sign(line.role)) (line.text) @if let Some(note) = line.note { " " (cite(note)) } "\n"
            } } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(language: &str, options: Options, code: &str) -> Result<String> {
        Ok(Listing::new(language, &options, code, &[])?.render("x", None, &mut |_| html!()).0)
    }

    #[test]
    fn options_are_checked() {
        assert_eq!(ranges("1, 3-4").unwrap(), [1..=1, 3..=4]);
        assert!(ranges("3-1").or(ranges("x")).is_err());
        assert!(page("cobol", Options::default(), "x").is_err());
        assert!(page("c", Options { highlight: ranges("2").unwrap(), ..Options::default() }, "x").is_err());
        assert!(page("c", Options { mark: vec!["y".into()], ..Options::default() }, "x").is_err());
        assert!(page("toml", Options { run: Some("".into()), ..Options::default() }, "x").is_err());
        assert!(Listing::new("c", &Options::default(), "x", &["a".into()]).is_err());
    }

    #[test]
    fn notes_take_their_comment_with_them() {
        let split = |text| note(text, &["a".into()]);
        assert_eq!(split("f(); // <a>"), ("f();", Some(0)));
        assert_eq!(split("f(); /* <a> */"), ("f();", Some(0)));
        assert_eq!(split("f(); // 50% <a>"), ("f(); // 50%", Some(0)));
        assert_eq!(split("Vec<a>").1.or(split("#include <b>").1), None);
    }

    #[test]
    fn removed_lines_are_not_counted() {
        let html = page("c", Options { diff: true, start: Some(9), ..Options::default() }, " a;\n-b;\n+c;\n").unwrap();
        assert!(html.contains(r#"class="l del"><span class="at"></span><del>b<"#), "{html}");
        assert!(html.contains(r#"class="l add" id="x-L10""#), "{html}");
    }

    #[test]
    fn sessions_tell_commands_from_output() {
        let options = Options::default();
        let session = Listing::new("console", &options, "$ a \\\n  b\n$1\n", &[]).unwrap();
        assert!(session.lines.iter().map(|line| line.role).eq([Role::Command, Role::Continued, Role::Output]));
        assert_eq!(session.portable(&mut |_| html!()).0, "<pre><code>$ a \\\n  b\n$1\n</code></pre>");
    }

    #[test]
    fn excerpts_keep_their_numbers_and_lose_their_margin() {
        let options = Options { lines: ranges("2-4").unwrap(), ..Options::default() };
        let excerpt = Listing::new("c", &options, "{\n    a;\n\n      b;\n}\n", &[]).unwrap();
        assert!(excerpt.lines.iter().map(|line| (line.number, line.text)).eq([(2, "a;"), (3, ""), (4, "  b;")]));
    }

    #[test]
    fn marks_and_links_span_tokens() {
        let marked = page("c", Options { mark: vec!["f(".into()], ..Options::default() }, "f(é);").unwrap();
        assert!(marked.contains(r#"<mark><span class="f">f</span><span class="o">(</span></mark>é"#));
        let linked = page("c", Options::default(), "// (https://a.b/c).").unwrap();
        assert!(linked.contains(r#"<a class="c" href="https://a.b/c">https://a.b/c</a>"#));
    }
}
