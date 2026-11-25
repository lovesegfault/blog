//! Markdown, as a tree.

use std::ops::Range;

use collision::{Kind, Piece};
use pulldown_cmark::{BrokenLink, Event, LinkType, Options, Parser, Tag, TextMergeWithOffset};

#[derive(Clone)]
pub struct Node<'a> {
    /// A `Start` for anything that has children.
    pub event: Event<'a>,
    pub children: Vec<Node<'a>>,
    pub span: Range<usize>,
}

const INLINE: Options = Options::ENABLE_MATH.union(Options::ENABLE_SMART_PUNCTUATION);
const EVERYTHING: Options = INLINE
    .union(Options::ENABLE_DEFINITION_LIST)
    .union(Options::ENABLE_FOOTNOTES)
    .union(Options::ENABLE_GFM)
    .union(Options::ENABLE_HEADING_ATTRIBUTES)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TABLES)
    .union(Options::ENABLE_TASKLISTS);

fn grow<'a>(events: &mut impl Iterator<Item = (Event<'a>, Range<usize>)>) -> Vec<Node<'a>> {
    let mut nodes = Vec::new();
    while let Some((event, span)) = events.next() {
        let children = match event {
            Event::Start(_) => grow(events),
            Event::End(_) => break,
            _ => Vec::new(),
        };
        nodes.push(Node { event, children, span });
    }
    nodes
}

/// `[text][nothing]` comes out as a link of an unknown type. `[text]` alone is just text.
pub fn parse(text: &str) -> Vec<Node<'_>> {
    let keep = |link: BrokenLink| (link.link_type != LinkType::Shortcut).then(|| ("".into(), "".into()));
    let parser = Parser::new_with_broken_link_callback(text, EVERYTHING, Some(keep));
    grow(&mut TextMergeWithOffset::new(parser.into_offset_iter()))
}

/// The inside of a `# heading`, where `<T>` is not a tag.
pub fn parse_heading(text: &str) -> Vec<Node<'_>> {
    let literal = |(event, span)| (if let Event::InlineHtml(tag) = event { Event::Text(tag) } else { event }, span);
    let events = Parser::new_ext(text, INLINE).into_offset_iter().map(literal);
    grow(&mut TextMergeWithOffset::new(events)).swap_remove(0).children
}

impl Node<'_> {
    pub fn is_inline(&self) -> bool {
        match &self.event {
            Event::Start(Tag::Emphasis | Tag::Strong | Tag::Strikethrough | Tag::Link { .. } | Tag::Image { .. }) => {
                true
            }
            Event::Start(_) | Event::Html(_) | Event::Rule => false,
            _ => true,
        }
    }

    fn is_image(&self) -> bool {
        matches!(self.event, Event::Start(Tag::Image { .. }))
    }

    pub fn count(&self, wanted: fn(&Event) -> bool) -> usize {
        usize::from(wanted(&self.event)) + self.children.iter().map(|child| child.count(wanted)).sum::<usize>()
    }
}

/// The words alone, a block to a line. `formula` says what to put for TeX.
pub fn plain(nodes: &[Node], formula: fn(&str) -> &str) -> String {
    let mut out = String::new();
    for node in nodes {
        match &node.event {
            Event::Text(text) | Event::Code(text) => out += text,
            Event::InlineMath(tex) | Event::DisplayMath(tex) => out += formula(tex),
            Event::SoftBreak | Event::HardBreak => out.push(' '),
            _ if node.is_image() => {}
            _ if node.is_inline() => out += &plain(&node.children, formula),
            _ => out += &format!("\n{}\n", plain(&node.children, formula)),
        }
    }
    out
}

pub fn words(nodes: &[Node]) -> usize {
    plain(nodes, |_| "x").split_whitespace().count()
}

/// A paragraph that opens with pictures on lines of their own is a figure, and the rest of it is the caption.
pub fn figure<'n, 'a>(inlines: &'n [Node<'a>]) -> Option<(Vec<&'n Node<'a>>, &'n [Node<'a>])> {
    let blank = |node: &Node| match &node.event {
        Event::Text(text) => text.trim().is_empty(),
        event => *event == Event::SoftBreak,
    };
    let (pictures, caption) = inlines.split_at(inlines.iter().take_while(|n| n.is_image() || blank(n)).count());
    let apart = caption.is_empty() || pictures.last()?.event == Event::SoftBreak;
    (apart && pictures.first()?.is_image()).then(|| (pictures.iter().filter(|n| n.is_image()).collect(), caption))
}

/// A quotation may end with a paragraph that opens with `--`, to say whose words they are. A longer dash is speech.
pub fn attribution<'n, 'a>(blocks: &'n [Node<'a>]) -> (&'n [Node<'a>], Option<Vec<Node<'a>>>) {
    if let [quoted @ .., last] = blocks
        && !quoted.is_empty()
        && last.event == Event::Start(Tag::Paragraph)
        && let Event::Text(opening) = &last.children[0].event
        && let Some(source) = opening.strip_prefix('–')
    {
        let mut inlines = last.children.clone();
        inlines[0].event = Event::Text(source.trim_start().to_owned().into());
        return (quoted, Some(inlines));
    }
    (blocks, None)
}

pub fn summary(blocks: &[Node]) -> String {
    let prose = |node: &&Node| node.event == Event::Start(Tag::Paragraph) && figure(&node.children).is_none();
    let text = blocks.iter().find(prose).map_or(String::new(), |node| plain(&node.children, |tex| tex));
    let mut summary = String::new();
    for word in text.split_whitespace() {
        if summary.len() + word.len() > 200 {
            return summary.trim_end_matches([' ', ',', ';', ':', '.']).to_owned() + "…";
        }
        summary += word;
        summary.push(' ');
    }
    summary.trim_end().to_owned()
}

/// The text as the detector sees it: by the paragraph, and by the sentence.
pub fn shape(blocks: &[Node]) -> [Vec<Piece>; 2] {
    let (mut paragraphs, mut sentences) = (Vec::new(), Vec::new());
    for block in blocks {
        let kind = match block.event {
            Event::Start(Tag::Heading { .. } | Tag::HtmlBlock | Tag::FootnoteDefinition(_)) | Event::Rule => continue,
            Event::Start(Tag::CodeBlock(_)) => Kind::Code,
            _ if figure(&block.children).is_some() => Kind::Figure,
            _ => Kind::Text,
        };
        let mut notes = block.count(|event| matches!(event, Event::FootnoteReference(_))) as u32;
        paragraphs.push(Piece { kind, words: words(&block.children) as u32, notes });
        if kind != Kind::Text {
            sentences.push(Piece { kind, words: if kind == Kind::Code { 20 } else { 8 }, notes: 0 });
            continue;
        }
        let text = [". ", "! ", "? "].iter().fold(plain(&block.children, |_| "x"), |text, end| text.replace(end, "\n"));
        for words in text.lines().map(|line| line.split_whitespace().count() as u32).filter(|&words| words > 0) {
            sentences.push(Piece { kind, words, notes: std::mem::take(&mut notes) });
        }
    }
    // A page of short lines would be one knot of curls.
    let mut lengths: Vec<_> = sentences.iter().map(|piece| piece.words).collect();
    lengths.sort();
    let stretch = lengths.get(lengths.len() / 2).map_or(1, |median| (14 / median).max(1));
    sentences.iter_mut().for_each(|piece| piece.words *= stretch);
    [paragraphs, sentences]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn figures_start_on_a_line_of_their_own() {
        let parts = |text| figure(&parse(text)[0].children).map(|(pictures, caption)| (pictures.len(), words(caption)));
        assert_eq!(parts("![a](a.jpg)"), Some((1, 0)));
        assert_eq!(parts("![a](a.jpg) ![b](b.jpg)\nTwo *of* them"), Some((2, 3)));
        assert_eq!(parts("![a](a.jpg) in a sentence"), None);
        assert_eq!(parts("text").or(parts("&nbsp;")), None);
    }

    #[test]
    fn quotations_can_be_attributed() {
        let source = |text| attribution(&parse(text)[0].children).1.map(|nodes| plain(&nodes, |tex| tex));
        assert_eq!(source("> words\n>\n> -- A, *B*").as_deref(), Some("A, B"));
        assert_eq!(source("> -- only this").or(source("> — Vamos?\n>\n> — Vamos.")), None);
    }

    #[test]
    fn titles_are_never_blocks() {
        assert_eq!(plain(&parse_heading("# 1. It's <T> {C#}"), |tex| tex), "1. It’s <T> {C#}");
    }
}
