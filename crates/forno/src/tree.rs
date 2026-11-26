//! What Typst makes of a page, as a tree.

use std::collections::HashSet;

use anyhow::{Result, anyhow};
use collision::{Kind, Piece};
use typst::comemo::Track;
use typst::foundations::{Content, Dict, FromValue, Value};
use typst::introspection::{Introspector, MetadataElem, Tag};
use typst::model::{EnumElem, LateLinkResolver, Numbering};
use typst::syntax::Span;
use typst_html::{HtmlDocument, HtmlFrame, HtmlNode, HtmlTag, tag};

use crate::content::slugify;
use crate::formula;
use crate::typeset::is_written;

#[derive(Clone)]
pub struct Node {
    /// Empty for text. The ones that start with `x-` are from `blog.typ`.
    pub tag: String,
    pub text: String,
    pub attributes: Vec<(String, String)>,
    pub children: Vec<Node>,
    pub span: Span,
    options: Dict,
}

#[derive(Clone, Default)]
pub struct Ids(HashSet<String>);

#[derive(Clone)]
pub struct Tree {
    pub nodes: Vec<Node>,
    pub ids: Ids,
    /// What Typst's own links call something, and what it is called.
    pub renamed: Vec<(String, String)>,
    named: HashSet<String>,
}

impl Ids {
    pub fn like(&mut self, wanted: &str) -> String {
        let mut candidates = [wanted.to_owned()].into_iter().chain((1..).map(|n| format!("{wanted}-{n}")));
        candidates.find(|id| !id.is_empty() && self.0.insert(id.clone())).unwrap()
    }

    pub fn numbered(&mut self, kind: &str) -> String {
        (1..).map(|n| format!("{kind}{n}")).find(|id| self.0.insert(id.clone())).unwrap()
    }
}

fn label(content: &Content) -> Option<String> {
    Some(content.label()?.resolve().to_string())
}

fn data(content: &Content) -> Option<&Value> {
    Some(&content.to_packed::<MetadataElem>()?.value)
}

/// What a list counts from, if it is lettered.
fn counting(content: &Content) -> Option<&str> {
    let Numbering::Pattern(pattern) = content.to_packed::<EnumElem>()?.numbering.as_option().as_ref()? else {
        return None;
    };
    pattern.pieces[0].1.shorthand().filter(|first| "aAiI".contains(first))
}

/// Typst takes the apostrophes of `f()`'s and the '90s for opening quotes, and that of the 90's for a prime.
fn requote<'a>(before: &[Node], text: &'a str, after: Option<&HtmlNode>) -> &'a str {
    let after = if let Some(HtmlNode::Text(after, _)) = after { after } else { "" };
    match text {
        "‘" if matches!(after.as_bytes(), [b'0'..=b'9', b'0', b's', ..]) => "’",
        "‘" if plain(before).ends_with(|c: char| !c.is_whitespace() && !"([{“-–—/".contains(c)) => "’",
        "’" if after.starts_with(char::is_alphabetic) && plain(before).ends_with('-') => "‘",
        "′" if after.starts_with(char::is_alphabetic) => "’",
        _ => text,
    }
}

impl Tree {
    pub fn new(document: &HtmlDocument) -> Self {
        let ids = Ids(document.introspector().query_labelled().iter().filter_map(label).collect());
        let mut tree = Self { nodes: Vec::new(), ids, renamed: Vec::new(), named: HashSet::new() };
        tree.nodes = tree.grow(document, &document.root().children, Span::detached()).pop().unwrap().children;
        tree
    }

    fn grow(&mut self, document: &HtmlDocument, nodes: &[HtmlNode], around: Span) -> Vec<Node> {
        let links = LateLinkResolver::new(None, document.introspector().as_ref());
        let missing = |frame: &HtmlFrame| Some(("missing".into(), formula::missing(&frame.inner)?.into()));
        let mut grown: Vec<Node> = Vec::new();
        // What the next node was made from.
        let mut origins: Vec<&Content> = Vec::new();
        // How much there was at the last blank line.
        let mut blank = 0;
        for (at, node) in nodes.iter().enumerate() {
            if let HtmlNode::Tag(Tag::Start(origin, _)) = node {
                blank = if data(origin) == Some(&Value::None) { grown.len() } else { blank };
                origins.push(origin);
                continue;
            }
            let ended =
                |origin: &&Content| matches!(node, HtmlNode::Tag(end) if origin.location() == Some(end.location()));
            let origins = origins.split_off(origins.iter().position(ended).unwrap_or(0));
            let own = origins
                .iter()
                .rev()
                .map(|origin| origin.span())
                .chain([node.span()])
                .find(|span| is_written(span.id()));
            let mut labels: Vec<String> = origins.iter().copied().filter_map(label).collect();
            labels.retain(|label| self.named.insert(label.clone()));

            let mut new = Node::new("", own.unwrap_or(around));
            match node {
                HtmlNode::Tag(_) => {}
                HtmlNode::Text(text, _) => new.text = requote(&grown, text, nodes.get(at + 1)).into(),
                HtmlNode::Element(element) if element.pre_span => {
                    new.text = plain(&self.grow(document, &element.children, around));
                }
                HtmlNode::Element(element) => {
                    new.tag = element.tag.resolve().to_string();
                    new.attributes =
                        element.attrs.0.iter().map(|(name, value)| (name.to_string(), value.into())).collect();
                    new.attributes
                        .extend(origins.into_iter().find_map(counting).map(|first| ("type".into(), first.into())));
                    let mut inside = &element.children[..];
                    if new.tag == "math"
                        && let Some(drawn) = inside.iter().rposition(|node| matches!(node, HtmlNode::Frame(_)))
                        && let (spoken, [HtmlNode::Frame(frame), ..]) = inside.split_at(drawn)
                    {
                        new.text = formula::draw(&frame.inner, frame.text_size, new.is_displayed(), links.track());
                        new.attributes.extend(missing(frame));
                        inside = spoken;
                    }
                    new.children = self.grow(document, inside, new.span);
                    new.span = own.or(new.children.first().map(|first| first.span)).unwrap_or(around);
                    if let Some(HtmlNode::Tag(Tag::Start(first, _))) = element.children.first()
                        && let Some(Value::Dict(options)) = data(first)
                    {
                        new.options = options.clone();
                    }
                }
                HtmlNode::Frame(frame) => {
                    new.tag = "x-frame".into();
                    new.attributes.extend(missing(frame));
                    new.attributes.push(("style".into(), frame.css.to_inline().to_string()));
                    new.attributes.extend(frame.id.as_ref().map(|id| ("id".into(), id.into())));
                    new.text = typst_svg::svg_in_html(
                        &frame.inner,
                        frame.text_size,
                        false,
                        None,
                        "height: auto",
                        &frame.anchors,
                        links.track(),
                    );
                }
            }

            let anchor = |id| Node { attributes: vec![("id".into(), id)], ..Node::new("span", around) };
            let slug = |ids: &mut Ids| new.level().map(|_| ids.like(&slugify(&plain(&new.children))));
            if let Some(id) = labels.pop_if(|_| !new.tag.is_empty()).or_else(|| slug(&mut self.ids)) {
                self.renamed
                    .extend(new.attributes.extract_if(.., |(name, _)| name == "id").map(|(_, old)| (old, id.clone())));
                // A paragraph may yet be joined to another.
                match new.children.first_mut() {
                    _ if new.tag != "p" => new.attributes.push(("id".into(), id)),
                    Some(formula) if formula.is_displayed() => formula.attributes.push(("id".into(), id)),
                    _ => new.children.insert(0, anchor(id)),
                }
            }
            grown.extend(labels.into_iter().map(anchor));
            let next = if new.tag == "p" { new.children.first() } else { Some(&new).filter(|new| new.is_inline()) };
            let cut =
                |last: &Node| next.is_some() && last.children.last().into_iter().chain(next).any(Node::is_displayed);
            match grown[blank..].last_mut() {
                _ if new.tag.is_empty() && new.text.is_empty() => {}
                Some(last) if last.tag.is_empty() && new.tag.is_empty() => last.text += &new.text,
                Some(last) if last.tag == "p" && cut(last) => {
                    last.children.push(Node { text: " ".into(), ..Node::new("", around) });
                    if new.tag == "p" {
                        last.children.extend(new.children);
                    } else {
                        let paragraph = grown.pop().unwrap();
                        grown.extend(paragraph.children.into_iter().chain([new]));
                    }
                }
                _ => grown.push(new),
            }
        }
        grown
    }
}

impl Node {
    fn new(tag: &str, span: Span) -> Self {
        Self { tag: tag.into(), text: "".into(), attributes: vec![], children: vec![], span, options: Dict::new() }
    }

    pub fn get(&self, attribute: &str) -> Option<&str> {
        self.attributes.iter().find(|(name, _)| name == attribute).map(|(_, value)| value.as_str())
    }

    pub fn option<T: FromValue>(&self, name: &str) -> Result<T> {
        let value = self.options.get(name).cloned().unwrap_or_default();
        value.cast().map_err(|error| anyhow!("`{name}`: {}", error.message()))
    }

    pub fn is_inline(&self) -> bool {
        let kind = HtmlTag::intern(&self.tag).ok().filter(|_| !self.tag.starts_with("x-"));
        kind.is_none_or(tag::should_group_into_pars) && self.get("style") != Some("display: block")
    }

    /// Whether it is a formula on a line of its own.
    pub fn is_displayed(&self) -> bool {
        self.tag == "math" && self.get("display").is_some()
    }

    /// Of a heading.
    pub fn level(&self) -> Option<usize> {
        ["h2", "h3", "h4", "h5", "h6"].iter().position(|tag| *tag == self.tag)
    }

    /// It and everything in it, but for what is in its notes and formulas.
    pub fn walk(&self) -> Box<dyn Iterator<Item = &Node> + '_> {
        let children = if matches!(&*self.tag, "x-note" | "math") { &[][..] } else { &self.children };
        Box::new([self].into_iter().chain(children.iter().flat_map(Node::walk)))
    }

    pub fn count(&self, tags: &[&str]) -> usize {
        self.walk().filter(|node| tags.contains(&&*node.tag)).count()
    }
}

/// The words alone, a block to a line.
pub fn plain(nodes: &[Node]) -> String {
    let mut out = String::new();
    for node in nodes {
        match node.tag.as_str() {
            "" => out += &node.text,
            "math" => out.extend(node.get("text").unwrap_or_default().split_whitespace()),
            "br" => out.push(' '),
            "x-note" | "x-video" | "x-explainer" | "script" | "style" => {}
            "p" | "li" | "dt" | "dd" | "th" | "td" | "pre" | "figcaption" => {
                out += &format!("\n{}\n", plain(&node.children));
            }
            _ => out += &plain(&node.children),
        }
    }
    out
}

pub fn words(nodes: &[Node]) -> usize {
    plain(nodes).split_whitespace().count()
}

pub fn summary(blocks: &[Node]) -> String {
    let text = blocks.iter().find(|node| node.tag == "p").map_or(String::new(), |node| plain(&node.children));
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
        let kind = match block.tag.as_str() {
            "p" | "ul" | "ol" | "dl" | "table" | "div" | "details" | "x-quote" | "x-aside" => Kind::Text,
            "pre" | "x-code" => Kind::Code,
            other if other == "figure" || other.starts_with("x-") => Kind::Figure,
            _ => continue,
        };
        let mut notes = block.count(&["x-note"]) as u32;
        paragraphs.push(Piece { kind, words: words(&block.children) as u32, notes });
        if kind != Kind::Text {
            sentences.push(Piece { kind, words: if kind == Kind::Code { 20 } else { 8 }, notes: 0 });
            continue;
        }
        let text = [". ", "! ", "? "].iter().fold(plain(&block.children), |text, end| text.replace(end, "\n"));
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
    use crate::typeset::tests::set;

    #[test]
    fn formulas_stay_in_their_paragraph() {
        let (blocks, _) = set("#show parbreak: it => it + metadata(none)\na $ x $ <x> b\n\n$ y $\n\n*c $ z $*");
        assert_eq!(plain(&blocks), "\na x b\n\ny\n\nc z\n");
        assert_eq!(blocks[0].children[2].get("id"), Some("x"));
    }

    #[test]
    fn headings_are_named_once() {
        let (blocks, _) = set("= A `b()`\n= A b\n= C <a-b-1>");
        assert!(blocks.iter().map(|heading| heading.get("id").unwrap()).eq(["a-b", "a-b-2", "a-b-1"]));
    }

    #[test]
    fn apostrophes_are_not_quotes() {
        assert_eq!(plain(&set("`f()`'s 90's 'a' (5') $x$'s '90s").0), "\nf()’s 90’s ‘a’ (5′) x’s ’90s\n");
    }
}
