//! A page's text, as HTML for the site and for feeds.

use std::collections::{BTreeMap, HashSet};
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail, ensure};
use collision::Post;
use maud::{Markup, PreEscaped, html};
use pulldown_cmark::{Alignment, BlockQuoteKind, CodeBlockKind, Event, LinkType, Tag};

use crate::code::{Listing, Options};
use crate::content::{Page, slugify};
use crate::links::{absolute, decode, is_external, resolve};
use crate::markdown::{self, Node, plain};
use crate::math::{self, Macros};
use crate::output::{File, read, read_text};
use crate::site::Build;
use crate::{photo, programs};

pub struct Article<'a> {
    pub page: &'a Page,
    pub title: Markup,
    /// The title in plain text.
    pub name: String,
    pub body: Markup,
    pub feed: String,
    pub contents: Vec<Section>,
    pub summary: String,
    pub minutes: usize,
    pub paragraphs: Post,
    pub sentences: Post,
}

pub struct Section {
    pub id: String,
    pub title: Markup,
    pub parts: Vec<Section>,
    level: usize,
}

struct Writer<'a> {
    build: &'a Build<'a>,
    page: &'a Page,
    /// What the spans of the nodes refer to.
    source: &'a str,
    macros: &'a Macros,
    notes: &'a BTreeMap<String, &'a Node<'a>>,
    /// Feed readers have none of the site's styles or scripts.
    feed: bool,
    /// For text that goes inside a link.
    linkless: bool,
    verse: bool,
    cited: Vec<(String, usize)>,
    ids: HashSet<String>,
    contents: Vec<Section>,
}

/// The widths a figure can have.
const COLUMN: f32 = 650.0;
const WIDE: f32 = 924.0;
const MARGIN: f32 = 230.0;

impl<'a> Article<'a> {
    pub fn new(build: &Build, page: &'a Page) -> Self {
        let blocks = markdown::parse(page.body());
        let note = |node| match node {
            &Node { event: Event::Start(Tag::FootnoteDefinition(ref label)), .. } => Some((label.to_lowercase(), node)),
            _ => None,
        };
        let notes = blocks.iter().filter_map(note).collect();
        let macros = build.config.macros.clone().into_iter().chain(page.front.macros.clone()).collect();
        let writer = |source, feed| Writer {
            build,
            page,
            source,
            macros: &macros,
            notes: &notes,
            feed,
            linkless: false,
            verse: false,
            cited: Vec::new(),
            ids: HashSet::new(),
            contents: Vec::new(),
        };
        let heading = format!("# {}", page.front.title);
        let title = markdown::parse_heading(&heading);
        let mut site = writer(page.body(), false);
        let [paragraphs, sentences] = markdown::shape(&blocks).map(|pieces| Post { name: page.slug.clone(), pieces });
        Self {
            page,
            title: Writer { linkless: true, ..writer(&heading, false) }.inlines(&title),
            name: verbatim(&title),
            body: site.document(&blocks),
            feed: absolute(&writer(page.body(), true).document(&blocks).0, &build.config.url, &page.url),
            contents: site.contents,
            summary: page.front.description.clone().unwrap_or_else(|| markdown::summary(&blocks)),
            minutes: (paragraphs.pieces.iter().map(|piece| piece.words as usize).sum::<usize>() + 110).max(220) / 220,
            paragraphs,
            sentences,
        }
    }
}

fn verbatim(nodes: &[Node]) -> String {
    plain(nodes, |tex| tex)
}

fn youtube(url: &str) -> Option<&str> {
    let id = ["https://www.youtube.com/watch?v=", "https://youtu.be/"].iter().find_map(|p| url.strip_prefix(p))?;
    id.split(['&', '?']).next()
}

impl Writer<'_> {
    /// Reports what went wrong, and shows it where it did.
    fn attempt(&self, node: &Node, result: Result<Markup>) -> Markup {
        result.unwrap_or_else(|error| {
            let offset = if self.source == self.page.body() { node.span.start } else { 0 };
            self.build.report(format!("{}: {error:#}", self.page.at(offset)));
            html! { span.problem { (format!("{error:#}")) } }
        })
    }

    fn link(&self, url: &str) -> String {
        resolve(&self.page.url, url)
    }

    fn id(&mut self, wanted: &str) -> String {
        let mut candidates = [wanted.to_owned()].into_iter().chain((1..).map(|n| format!("{wanted}-{n}")));
        candidates.find(|id| !id.is_empty() && self.ids.insert(id.clone())).unwrap()
    }

    fn numbered(&mut self, kind: &str) -> String {
        (1..).map(|n| format!("{kind}{n}")).find(|id| self.ids.insert(id.clone())).unwrap()
    }

    fn cite(&mut self, label: &str) -> Markup {
        if self.linkless {
            return html!();
        }
        let label = label.to_lowercase();
        let index = self.cited.iter().position(|(cited, _)| *cited == label).unwrap_or_else(|| {
            self.cited.push((label, 0));
            self.cited.len() - 1
        });
        let (number, earlier) = (index + 1, self.cited[index].1);
        self.cited[index].1 += 1;
        html! {
            sup.ref id={ "fnref" @if earlier > 0 { (earlier) } ":" (number) } {
                a href={ "#fn:" (number) } role="doc-noteref" { (number) }
            }
        }
    }

    fn document(&mut self, blocks: &[Node]) -> Markup {
        let body = self.blocks(blocks);
        let mut notes = Vec::new();
        // Notes can have notes.
        while let Some((label, _)) = self.cited.get(notes.len()) {
            let number = notes.len() + 1;
            let text = self.notes.get(label).map_or(String::new(), |note| self.blocks(&note.children).0);
            let (text, end) = text.split_at(text.strip_suffix("</p>").unwrap_or(&text).len());
            notes.push(html! {
                li id={ "fn:" (number) } {
                    (PreEscaped(text)) "\u{a0}"
                    a.back href={ "#fnref:" (number) } role="doc-backlink" aria-label="Back to the text" { "↑" }
                    (PreEscaped(end))
                }
            });
        }
        for (label, note) in self.notes.iter().filter(|(label, _)| self.cited.iter().all(|(cited, _)| cited != *label))
        {
            self.attempt(note, Err(anyhow!("nothing refers to the note `{label}`")));
        }
        html! {
            (body)
            @if !notes.is_empty() {
                section.notes role="doc-endnotes" aria-label="Notes" {
                    @if self.feed { hr; }
                    ol { @for note in notes { (note) } }
                }
            }
        }
    }

    fn blocks(&mut self, nodes: &[Node]) -> Markup {
        html! {
            @for run in nodes.chunk_by(|a, b| a.is_inline() && b.is_inline()) {
                @if run[0].is_inline() {
                    (self.figure(run).unwrap_or_else(|| self.inlines(run)))
                } @else {
                    (self.block(&run[0]))
                }
            }
        }
    }

    fn block(&mut self, node: &Node) -> Markup {
        let children = node.children.as_slice();
        let tag = match &node.event {
            Event::Start(tag) => tag,
            Event::Html(html) => return PreEscaped(html.to_string()),
            _ => return html! { hr; },
        };
        match tag {
            Tag::Paragraph if self.verse => html! { p {
                @for line in children.split(|node| node.event == Event::HardBreak) {
                    span.line { (self.inlines(line)) br; }
                }
            } },
            Tag::Paragraph => self.figure(children).unwrap_or_else(|| html! { p { (self.inlines(children)) } }),
            Tag::Heading { level, id, .. } => self.heading(*level as usize, id.as_deref(), children),
            Tag::BlockQuote(None) => self.quotation(node),
            Tag::BlockQuote(Some(kind)) => self.aside(*kind, children),
            Tag::CodeBlock(kind) => {
                let info = if let CodeBlockKind::Fenced(info) = kind { info } else { "" };
                let code = self.code(info, verbatim(children));
                self.attempt(node, code)
            }
            Tag::List(None) => html! { ul { (self.blocks(children)) } },
            Tag::List(Some(start)) => html! { ol start=[(*start != 1).then_some(start)] { (self.blocks(children)) } },
            Tag::Item => html! { li { (self.blocks(children)) } },
            Tag::Table(columns) => self.table(columns, children),
            Tag::DefinitionList => html! { dl { (self.blocks(children)) } },
            Tag::DefinitionListTitle => html! { dt { (self.blocks(children)) } },
            Tag::DefinitionListDefinition => html! { dd { (self.blocks(children)) } },
            Tag::FootnoteDefinition(label) => match self.notes.get(&label.to_lowercase()) {
                Some(note) if note.span == node.span => html!(),
                Some(_) => self.attempt(node, Err(anyhow!("there is another note called `{label}`"))),
                None => self.attempt(node, Err(anyhow!("a note cannot be written inside a block"))),
            },
            _ => self.blocks(children),
        }
    }

    fn heading(&mut self, level: usize, id: Option<&str>, children: &[Node]) -> Markup {
        let id = self.id(&id.map_or_else(|| slugify(&verbatim(children)), str::to_owned));
        if matches!(level, 2 | 3) {
            self.linkless = true;
            let section = Section { id: id.clone(), title: self.inlines(children), parts: Vec::new(), level };
            self.linkless = false;
            match self.contents.last_mut() {
                Some(parent) if parent.level < level => parent.parts.push(section),
                _ => self.contents.push(section),
            }
        }
        let text = html! {
            (self.inlines(children))
            @if !self.feed { a.pin href={ "#" (id) } aria-label="Link to this section" {} }
        };
        PreEscaped(format!(r#"<h{level} id="{}">{}</h{level}>"#, html!((id)).0, text.0))
    }

    fn quotation(&mut self, node: &Node) -> Markup {
        let (quoted, source) = markdown::attribution(&node.children);
        let verse = node.count(|event| *event == Event::HardBreak) > 0;
        let technical = node.count(|event| matches!(event, Event::Code(_) | Event::InlineMath(_))) > 0;
        let short = !technical && markdown::words(quoted) <= 30;
        let around = std::mem::replace(&mut self.verse, verse);
        let quoted = self.blocks(quoted);
        self.verse = around;
        html! {
            figure.quote.large[verse || short] {
                blockquote { (quoted) }
                @if let Some(source) = source { figcaption { "—\u{a0}" (self.inlines(&source)) } }
            }
        }
    }

    fn aside(&mut self, kind: BlockQuoteKind, children: &[Node]) -> Markup {
        let label = format!("{kind:?}");
        let inside = html! {
            @if kind != BlockQuoteKind::Important { p.label { b { (label) } } }
            (self.blocks(children))
        };
        html! {
            @if self.feed { blockquote { (inside) } } @else { div.aside.(label.to_lowercase()) role="note" { (inside) } }
        }
    }

    fn table(&mut self, columns: &[Alignment], rows: &[Node]) -> Markup {
        let align = |column: &Alignment| match column {
            Alignment::Center => Some("text-align:center"),
            Alignment::Right => Some("text-align:right"),
            _ => None,
        };
        html! { div.table { table {
            thead { tr { @for (cell, column) in rows[0].children.iter().zip(columns) {
                th style=[align(column)] { (self.inlines(&cell.children)) }
            } } }
            tbody { @for row in &rows[1..] { tr { @for (cell, column) in row.children.iter().zip(columns) {
                td style=[align(column)] { (self.inlines(&cell.children)) }
            } } } }
        } } }
    }

    fn code(&mut self, info: &str, written: String) -> Result<Markup> {
        let options = Options::parse(info)?;
        let code = match options.file {
            Some(file) => {
                ensure!(written.is_empty(), "a block that takes its code from a `file` has to be empty");
                read_text(&self.page.file(file)?)?
            }
            None => written,
        };
        match options.language {
            "math" => return Ok(html! { p { (self.math(&code, true)?) } }),
            "dot" => {
                let drawing = programs::diagram(self.build.tools, &code, &self.numbered("diagram"))?;
                return Ok(html! { figure {
                    div.drawing.diagram { (PreEscaped(drawing)) }
                    @if let Some(title) = options.title { figcaption { (title) } }
                } });
            }
            _ => {}
        }
        let (whole, notes) = (options.file.is_some(), self.notes);
        let listing = Listing::new(options, &code, &|label| notes.contains_key(&label.to_lowercase()))?;
        if self.feed {
            return Ok(listing.portable(&mut |label| self.cite(label)));
        }
        let id = match listing.title() {
            Some(title) => self.id(&slugify(&title.replace(['.', '/'], "-"))),
            None => self.numbered("code"),
        };
        // Servers take `.rs` for XML and `.ts` for video.
        let source = whole.then(|| self.build.output.asset(&format!("{id}.txt"), File::Made(code.clone().into())));
        Ok(listing.render(&id, source.transpose()?.as_deref(), &mut |label| self.cite(label)))
    }

    fn math(&self, tex: &str, display: bool) -> Result<Markup> {
        let html = self.build.tools.math.render(tex, display, self.macros)?;
        Ok(PreEscaped(if self.feed { math::portable(&html) } else { html }))
    }

    fn figure(&mut self, inlines: &[Node]) -> Option<Markup> {
        let (pictures, caption) = markdown::figure(inlines)?;
        let Event::Start(Tag::Image { dest_url, .. }) = &pictures[0].event else { unreachable!() };
        let place = ["wide", "margin"].into_iter().find(|place| dest_url.rsplit('#').next() == Some(place));
        let width = match place {
            Some("wide") => WIDE,
            Some(_) => MARGIN,
            None => COLUMN,
        };
        Some(html! {
            figure class=[place] {
                @if let [picture] = pictures[..] {
                    (self.embed(picture, width, true))
                } @else {
                    div.row { @for picture in &pictures { (self.embed(picture, width / 1.5, true)) } }
                }
                @if !caption.is_empty() { figcaption { (self.inlines(caption)) } }
            }
        })
    }

    /// Whatever is written like an image.
    fn embed(&self, node: &Node, width: f32, framed: bool) -> Markup {
        let Event::Start(Tag::Image { dest_url, title, .. }) = &node.event else { unreachable!() };
        let url = dest_url.trim_end_matches("#wide").trim_end_matches("#margin");
        let (alt, title) = (verbatim(&node.children), Some(&**title).filter(|title| !title.is_empty()));
        // Feeds get the files as they are, whose addresses last.
        let own = !is_external(url) && !url.starts_with('/') && !self.feed;
        let embedded = if let Some(name) = url.strip_prefix("explainer:") {
            self.explainer(name, &alt)
        } else if let Some(id) = youtube(url) {
            Ok(html! {
                @if self.feed {
                    a href=(url) { (alt) }
                } @else {
                    iframe.video src={ "https://www.youtube-nocookie.com/embed/" (id) } title=(alt) loading="lazy"
                        allow="encrypted-media; fullscreen; picture-in-picture" {}
                }
            })
        } else if photo::is(Path::new(url), &["mp4", "webm"]) {
            Ok(html! { video src=(self.link(url)) aria-label=(alt) preload="metadata" controls playsinline {} })
        } else if own && photo::is_photo(Path::new(url)) {
            self.photo(url, &alt, title, width, framed)
        } else if own && url.ends_with(".svg") {
            self.drawing(url, &alt)
        } else {
            Ok(html! { img src=(self.link(url)) alt=(alt) title=[title] loading="lazy"; })
        };
        self.attempt(node, embedded)
    }

    fn photo(&self, url: &str, alt: &str, title: Option<&str>, width: f32, framed: bool) -> Result<Markup> {
        let path = self.page.file(&decode(url))?;
        let photo = photo::print(&self.build.tools.cache, &path)?;
        let name = slugify(&path.file_stem().unwrap().to_string_lossy());
        let mut prints = Vec::new();
        for (width, file) in photo.prints {
            prints.push((width, self.build.output.asset(&format!("{name}-{width}.webp"), File::Copied(file))?));
        }
        let (_, usual) = prints.iter().rfind(|(width, _)| *width <= 1300).unwrap();
        let choices: Vec<_> = prints.iter().map(|(width, url)| format!("{url} {width}w")).collect();
        let sizes = format!("(min-width: 1800px) {:.0}px, (min-width: 700px) {width:.0}px, 90vw", width * 1.28);
        let picture = html! {
            img src=(usual) srcset=(choices.join(", ")) sizes=(sizes)
                width=(photo.width) height=(photo.height) alt=(alt) title=[title] loading="lazy" decoding="async";
        };
        let shape = format!("--ratio:{:.3}", photo.width as f32 / photo.height as f32);
        Ok(if framed {
            html! { a.photo href=(self.link(url)) style=(shape) { (picture) } }
        } else {
            picture
        })
    }

    /// A drawing that takes its colour from the text has to be part of the page.
    fn drawing(&self, url: &str, alt: &str) -> Result<Markup> {
        let svg = read_text(&self.page.file(&decode(url))?)?;
        Ok(match svg.find("<svg").filter(|_| svg.contains("currentColor")) {
            Some(start) => html! { span.drawing role="img" aria-label=(alt) { (PreEscaped(&svg[start..])) } },
            None => html! { img src=(self.link(url)) alt=(alt) loading="lazy"; },
        })
    }

    fn explainer(&self, name: &str, alt: &str) -> Result<Markup> {
        if self.feed {
            return Ok(html! { a href=(self.page.url) { (alt) } });
        }
        let (name, options) = name.split_once('?').unwrap_or((name, ""));
        let program = read(&self.build.programs.join(format!("{name}.wasm")));
        let program = program.ok().with_context(|| format!("there is no crates/explainers/src/bin/{name}.rs"))?;
        let view = programs::first_view(&program, options).with_context(|| format!("`{name}` did not start"))?;
        let program = self.build.output.asset(&format!("{name}.wasm"), File::Made(program))?;
        Ok(html! {
            div.drawing.explainer role="group" aria-label=(alt) data-program=(program) data-options=(options) {
                (PreEscaped(view))
            }
        })
    }

    fn inlines(&mut self, nodes: &[Node]) -> Markup {
        html! { @for node in nodes { (self.inline(node)) } }
    }

    fn inline(&mut self, node: &Node) -> Markup {
        let children = node.children.as_slice();
        match &node.event {
            Event::Text(text) => self.attempt(node, self.text(node, text)),
            Event::Code(code) => html! { code { (code) } },
            Event::InlineMath(_) if self.is_money(node) => {
                self.attempt(node, Err(anyhow!("this reads as a formula. A dollar sign is `\\$`")))
            }
            Event::InlineMath(tex) => self.attempt(node, self.math(tex, false)),
            Event::DisplayMath(tex) => self.attempt(node, self.math(tex, true)),
            Event::InlineHtml(html) => PreEscaped(html.to_string()),
            Event::FootnoteReference(label) => self.cite(label),
            Event::SoftBreak => html! { "\n" },
            Event::HardBreak => html! { br; },
            Event::TaskListMarker(done) => html! { input type="checkbox" checked[*done] disabled aria-label="Done"; },
            Event::Start(Tag::Emphasis) => html! { em { (self.inlines(children)) } },
            Event::Start(Tag::Strong) => html! { strong { (self.inlines(children)) } },
            Event::Start(Tag::Strikethrough) => html! { del { (self.inlines(children)) } },
            Event::Start(Tag::Image { .. }) => self.embed(node, COLUMN, false),
            Event::Start(Tag::Link {
                link_type: LinkType::ReferenceUnknown | LinkType::CollapsedUnknown, id, ..
            }) => self.attempt(node, Err(anyhow!("the link `[{id}]` is not defined"))),
            Event::Start(Tag::Link { link_type, dest_url, title, .. }) if !self.linkless => match link_type {
                LinkType::Autolink => html! { a href=(dest_url) { (dest_url) } },
                LinkType::Email => html! { a href={ "mailto:" (dest_url) } { (dest_url) } },
                _ => html! {
                    a href=(self.link(dest_url)) title=[Some(title).filter(|t| !t.is_empty())] { (self.inlines(children)) }
                },
            },
            _ => self.inlines(children),
        }
    }

    /// As in `R$5 or R$10`.
    fn is_money(&self, formula: &Node) -> bool {
        self.source[..formula.span.start].ends_with(char::is_alphabetic)
            || self.source[formula.span.end..].starts_with(char::is_numeric)
    }

    /// Markdown leaves what it cannot make sense of as it is.
    fn text(&self, node: &Node, text: &str) -> Result<Markup> {
        let written = self.source.as_bytes();
        for at in node.span.clone().filter(|&at| at == 0 || written[at - 1] != b'\\') {
            match written[at..node.span.end] {
                [b'[', b'^', ..] => bail!("a note is referred to here, but never written"),
                [b'$', b'$', ..] => bail!("a line of this formula may read as a list or a heading: use a `math` block"),
                [b'$', b'a'..=b'z' | b'A'..=b'Z' | b'\\' | b'(' | b'{', ..] => {
                    bail!("this formula is never closed, or its braces are not. A dollar sign is `\\$`")
                }
                _ => {}
            }
        }
        Ok(html! { (text) })
    }
}
