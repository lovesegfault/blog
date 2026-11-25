//! Code blocks.

use std::ops::{Range, RangeInclusive};

use anyhow::{Context, Result, bail, ensure};
use maud::{Markup, html};

use crate::highlight::{self, COMMENT, Ink, Language};

type Lines = RangeInclusive<usize>;

/// What follows the opening fence: `rust title="main.rs" hl=3,5-7`.
#[derive(Default)]
pub struct Options<'a> {
    pub language: &'a str,
    pub title: Option<&'a str>,
    /// Takes the code from one of the page's files.
    pub file: Option<&'a str>,
    lines: Option<Lines>,
    start: Option<usize>,
    numbers: Option<bool>,
    diff: bool,
    wrap: bool,
    /// Offers the code on Compiler Explorer, with these flags.
    run: Option<&'a str>,
    highlight: Vec<Lines>,
    fold: Vec<Lines>,
    mark: Vec<&'a str>,
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
    note: Option<&'a str>,
    inks: Vec<Ink>,
}

pub struct Listing<'a> {
    options: Options<'a>,
    language: Option<&'static Language>,
    lines: Vec<Line<'a>>,
}

/// How many columns fit beside the text.
const COLUMNS: usize = 80;

/// Compiler Explorer knows these by the same names.
const RUNNABLE: [&str; 7] = ["C", "C++", "Go", "Haskell", "Python", "Rust", "Zig"];

fn range(text: &str) -> Result<Lines> {
    let (first, last) = text.split_once('-').unwrap_or((text, text));
    let lines = first.parse().and_then(|first| Ok(first..=last.parse()?)).ok().filter(|lines| !lines.is_empty());
    lines.with_context(|| format!("`{text}` is not a line or a range of lines"))
}

impl<'a> Options<'a> {
    pub fn parse(info: &'a str) -> Result<Self> {
        let (language, mut rest) = info.split_once(' ').unwrap_or((info, ""));
        let mut options = Self { language, ..Self::default() };
        while let text @ [_, ..] = rest.trim_start().as_bytes() {
            rest = &rest[rest.len() - text.len()..];
            let (key, after) = rest.split_at(rest.find([' ', '=']).unwrap_or(rest.len()));
            let value;
            (value, rest) = match after.strip_prefix('=') {
                Some(after) if after.starts_with(['"', '\'']) => after[1..]
                    .split_once(&after[..1])
                    .with_context(|| format!("the quote after `{key}` is not closed"))?,
                Some(after) => after.split_once(' ').unwrap_or((after, "")),
                None => ("", after),
            };
            let ranges = || value.split(',').map(range).collect::<Result<Vec<_>>>();
            match (key, value) {
                ("diff", "") => options.diff = true,
                ("wrap", "") => options.wrap = true,
                ("run", _) => options.run = Some(value),
                ("numbers", "") => options.numbers = Some(true),
                ("nonumbers", "") => options.numbers = Some(false),
                (_, "") => bail!("`{key}` is not something a code block can be"),
                ("title", _) => options.title = Some(value),
                ("file", _) => options.file = Some(value),
                ("lines", _) => options.lines = Some(range(value)?),
                ("start", _) => options.start = Some(value.parse().context("`start` takes a number")?),
                ("hl", _) => options.highlight.extend(ranges()?),
                ("fold", _) => options.fold.extend(ranges()?),
                ("mark", _) => options.mark.push(value),
                _ => bail!("code blocks have no `{key}`"),
            }
        }
        Ok(options)
    }

    fn is(&self, language: &str) -> bool {
        self.language.eq_ignore_ascii_case(language)
    }

    /// What a program printed, rather than a program.
    fn is_transcript(&self) -> bool {
        ["", "text", "console"].iter().any(|language| self.is(language))
    }
}

/// Splits a reference to a note off the end of a line, with the comment it was in if nothing else is left of it.
fn note<'a>(text: &'a str, is_note: &dyn Fn(&str) -> bool) -> (&'a str, Option<&'a str>) {
    const LEADERS: &[&str] = &["///", "//", "#", "--", ";;", ";", "%"];
    for (open, close) in [(&["/*"][..], "*/"), (&["<!--"], "-->"), (LEADERS, ""), (&[""], "")] {
        let reference = text.trim_end().strip_suffix(close).and_then(|text| text.trim_end().strip_suffix(']'));
        if let Some((before, label)) = reference.and_then(|text| text.rsplit_once("[^")).filter(|(_, l)| is_note(l))
            && let Some(code) = open.iter().find_map(|open| before.trim_end().strip_suffix(open))
            && (open != LEADERS || !code.ends_with(|c: char| !c.is_whitespace()))
        {
            return (code.trim_end(), Some(label));
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
    pub fn new(options: Options<'a>, code: &'a str, is_note: &dyn Fn(&str) -> bool) -> Result<Self> {
        let (patch, console) = (options.is("diff"), options.is("console"));
        let language = highlight::language(options.language);
        ensure!(
            language.is_some() || patch || options.is_transcript(),
            "there is no grammar for `{}`: use `text`, or add one to highlight.rs",
            options.language
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
            let (text, note) = note(text, is_note);
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

        if let Some(wanted) = &options.lines {
            lines.retain(|line| wanted.contains(&line.number));
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
        Ok(Self { options, language, lines })
    }

    pub fn title(&self) -> Option<&'a str> {
        self.options.title.or(self.options.file.map(|file| file.rsplit('/').next().unwrap()))
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
    pub fn render(&self, id: &str, source: Option<&str>, cite: &mut dyn FnMut(&str) -> Markup) -> Markup {
        let options = &self.options;
        let (patch, wrap) = (options.is("diff"), options.wrap || options.is_transcript());
        let numbered = options.numbers.unwrap_or(!patch && !options.is_transcript());
        let label = if patch { Some("diff") } else { self.language.map(|language| language.label) };
        let width = |line: &Line| line.text.chars().count() + 3 * line.text.matches('\t').count();
        let columns = self.lines.iter().map(width).max().unwrap_or(0);
        let wide = columns > COLUMNS && !wrap && self.lines.iter().all(|line| line.note.is_none());
        let digits = self.lines.last().map_or(0, |line| line.number.to_string().len()).max(2);
        let fold = |line: &Line| options.fold.iter().position(|lines| lines.contains(&line.number));

        let mut pre = |lines: &[Line]| {
            html! { pre { code { @for line in lines {
                @let anchor = (numbered && (line.role != Role::Removed || patch)).then(|| format!("{id}-L{}", line.number));
                @let lit = options.highlight.iter().any(|lines| lines.contains(&line.number));
                span.l.lit[lit].cmd[line.role == Role::Command].out[line.role == Role::Output]
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
                    @if let Some(label) = line.note { (cite(label)) }
                }
            } } } }
        };
        html! {
            figure.code.wide[wide].wrap[wrap].patch[patch] id=(id)
                style={ "--columns:" (columns) ";--digits:" (digits) }
                data-language=[options.run.and(label).map(str::to_lowercase)] data-options=[options.run]
            {
                @if let Some(name) = self.title().or(label) {
                    figcaption class=(self.language.map_or("cream", |language| language.glaze)) {
                        @if let Some(source) = source { a href=(source) { (name) } } @else { (name) }
                    }
                }
                div.lines {
                    @for part in self.lines.chunk_by(|a, b| fold(a) == fold(b)) {
                        @if fold(&part[0]).is_some() {
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
    pub fn portable(&self, cite: &mut dyn FnMut(&str) -> Markup) -> Markup {
        let sign = |role| match role {
            Role::Command => "$ ",
            _ if !self.options.diff => "",
            Role::Added => "+",
            Role::Removed => "-",
            _ => " ",
        };
        html! {
            @if let Some(title) = self.title() { p { i { (title) } } }
            pre { code { @for line in &self.lines {
                (sign(line.role)) (line.text) @if let Some(label) = line.note { " " (cite(label)) } "\n"
            } } }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn listing<'a>(info: &'a str, code: &'a str) -> Result<Listing<'a>> {
        Listing::new(Options::parse(info)?, code, &|label| label == "a")
    }

    fn page(info: &str, code: &str) -> String {
        listing(info, code).unwrap().render("x", None, &mut |label| html! { sup { (label) } }).0
    }

    #[test]
    fn options_are_checked() {
        let options = Options::parse(r#"rust  title="a b" hl=1,3-4 mark='"x"' diff"#).unwrap();
        assert_eq!((options.title, options.highlight, options.mark), (Some("a b"), vec![1..=1, 3..=4], vec!["\"x\""]));
        for wrong in ["c hl", "c hl=3-1", "c title=\"x", "c colour=red", "cobol", "c hl=2", "c mark=y", "toml run"] {
            assert!(listing(wrong, "x").is_err(), "{wrong}");
        }
    }

    #[test]
    fn notes_take_their_comment_with_them() {
        let split = |text| note(text, &|label| label == "a");
        assert_eq!(split("f(); // [^a]"), ("f();", Some("a")));
        assert_eq!(split("f(); /* [^a] */"), ("f();", Some("a")));
        assert_eq!(split("f(); // 50% [^a]"), ("f(); // 50%", Some("a")));
        assert_eq!(split("grep '[^b]"), ("grep '[^b]", None));
    }

    #[test]
    fn removed_lines_are_not_counted() {
        let html = page("c diff start=9", " a;\n-b;\n+c;\n");
        assert!(html.contains(r#"class="l del"><span class="at"></span><del>b<"#), "{html}");
        assert!(html.contains(r#"class="l add" id="x-L10""#), "{html}");
    }

    #[test]
    fn sessions_tell_commands_from_output() {
        let session = listing("console", "$ a \\\n  b\n$1\n").unwrap();
        assert!(session.lines.iter().map(|line| line.role).eq([Role::Command, Role::Continued, Role::Output]));
        assert_eq!(session.portable(&mut |_| html!()).0, "<pre><code>$ a \\\n  b\n$1\n</code></pre>");
    }

    #[test]
    fn excerpts_keep_their_numbers_and_lose_their_margin() {
        let excerpt = listing("c lines=2-4", "{\n    a;\n\n      b;\n}\n").unwrap();
        assert!(excerpt.lines.iter().map(|line| (line.number, line.text)).eq([(2, "a;"), (3, ""), (4, "  b;")]));
    }

    #[test]
    fn marks_and_links_span_tokens() {
        assert!(
            page("c mark=f(", "f(é);").contains(r#"<mark><span class="f">f</span><span class="o">(</span></mark>é"#)
        );
        assert!(page("c", "// (https://a.b/c).").contains(r#"<a class="c" href="https://a.b/c">https://a.b/c</a>"#));
    }
}
