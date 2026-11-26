//! A page's text, as HTML for the site and for feeds.

use std::collections::HashSet;
use std::mem::replace;
use std::path::Path;

use anyhow::{Context, Result, bail, ensure};
use collision::Post;
use maud::{Markup, PreEscaped, html};
use typst_html::tag::mathml::{is_mathml, is_token};
use typst_html::{HtmlTag, tag};

use crate::code::{Listing, Options};
use crate::content::{Page, published, slugify};
use crate::links::{absolute, resolve, values};
use crate::output::{File, read, read_text};
use crate::site::Build;
use crate::style::themed;
use crate::tree::{self, Ids, Node, plain};
use crate::{formula, photo, programs};

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
    /// Feed readers have none of the site's styles or scripts.
    feed: bool,
    /// For titles, which are shown in more than one place.
    copy: bool,
    in_link: bool,
    verse: bool,
    in_figure: bool,
    in_formula: bool,
    /// For pictures, in rem.
    room: f32,
    /// Each note, and how many times it was referred to.
    notes: Vec<(&'a Node, usize, String)>,
    ids: Ids,
    contents: Vec<Section>,
}

const COLUMN: f32 = 32.5;
const WIDE: f32 = 46.2;
const MARGIN: f32 = 11.5;

/// The stylesheet does not reach these.
const THEMED: &str = "display: inherit; min-width: var(--fill); max-width: 100%; height: auto; margin: auto;";

impl<'a> Article<'a> {
    pub fn new(build: &Build, page: &'a Page) -> Self {
        let blocks = &page.body.nodes;
        let writer = |feed| Writer {
            build,
            page,
            feed,
            copy: false,
            in_link: false,
            verse: false,
            in_figure: false,
            in_formula: false,
            room: COLUMN,
            notes: Vec::new(),
            ids: page.body.ids.clone(),
            contents: Vec::new(),
        };
        let mut site = writer(false);
        let [paragraphs, sentences] = tree::shape(blocks).map(|pieces| Post { name: page.slug.clone(), pieces });
        Self {
            page,
            title: site.title(&page.heading),
            name: plain(&page.heading.children),
            body: PreEscaped(site.document()),
            feed: absolute(&writer(true).document(), &build.config.url, &page.url),
            contents: site.contents,
            summary: page.description.clone().unwrap_or_else(|| tree::summary(blocks)),
            minutes: (paragraphs.pieces.iter().map(|piece| piece.words as usize).sum::<usize>() + 110).max(220) / 220,
            paragraphs,
            sentences,
        }
    }
}

/// The note that has a label.
fn labelled<'a>(nodes: &'a [Node], label: &str) -> Option<&'a Node> {
    let found =
        |node: &'a Node| node.children.first().filter(|_| node.tag == "x-note" && node.get("id") == Some(label));
    nodes.iter().find_map(|node| found(node).or_else(|| labelled(&node.children, label)))
}

/// What Typst drew.
fn drawn(node: &Node) -> Result<&str> {
    if let Some(text) = node.get("missing") {
        bail!("there is no font to draw `{text}` with");
    }
    Ok(&node.text)
}

fn given(text: &str) -> Option<&str> {
    Some(text).filter(|text| !text.is_empty())
}

/// The links in a picture cannot be reached.
fn role(svg: &str, alt: &str) -> Option<&'static str> {
    given(alt).map(|_| if svg.contains("</a>") { "group" } else { "img" })
}

/// The video, and the second it starts at.
fn youtube(url: &str) -> Option<(&str, u32)> {
    let after = |signs: &str| signs.split(' ').find_map(|sign| url.split_once(sign)?.1.split(['&', '?', '#']).next());
    let id = after("?v= &v= youtu.be/ /shorts/ /live/ /embed/").filter(|_| url.contains("youtu"))?;
    let start = after("?t= &t= #t= start=").unwrap_or_default().split_inclusive(['h', 'm']);
    let unit = |part: &str| 60_u32.pow(part.chars().find_map(|unit| "smh".find(unit)).unwrap_or(0) as u32);
    Some((id, start.map(|part| part.trim_end_matches(['h', 'm', 's']).parse().unwrap_or(0) * unit(part)).sum()))
}

impl<'a> Writer<'a> {
    fn document(&mut self) -> String {
        let html = html! {
            (self.all(&self.page.body.nodes))
            @if !self.notes.is_empty() {
                section.notes role="doc-endnotes" aria-label="Notes" {
                    @if self.feed { hr; }
                    ol { @for (number, (.., text)) in (1..).zip(&self.notes) {
                        @let (text, end) = text.split_at(text.strip_suffix("</p>").unwrap_or(text).len());
                        li id={ "fn:" (number) } {
                            (PreEscaped(text)) "\u{a0}"
                            a.back href={ "#fnref:" (number) } role="doc-backlink" aria-label="Back to the text" { "↑" }
                            (PreEscaped(end))
                        }
                    } }
                }
            }
        };
        let link = |id| format!("href=\"#{}\"", html!((id)).0);
        self.page.body.renamed.iter().fold(html.0, |html, (old, new)| html.replace(&link(old), &link(new)))
    }

    fn all(&mut self, nodes: &'a [Node]) -> Markup {
        html! { @for node in nodes { (self.attempt(node)) } }
    }

    fn inside(&mut self, node: &'a Node) -> Markup {
        let is = |node: &Node, kind: fn(_) -> _| HtmlTag::intern(&node.tag).is_ok_and(kind);
        let strict = is(node, is_mathml) && !is(node, is_token);
        // For readers that keep nothing but the text.
        let unseen = |signs: &String| {
            html! { @if signs.trim().is_empty() { (signs) } @else {
                mpadded width="0" height="0" depth="0" { mphantom { mtext { (signs) } } }
            } }
        };
        html! { @for node in &node.children {
            @let part = self.attempt(node);
            @let part = if strict && !is(node, is_mathml) { html! { mtext { (part) } } } else { part };
            @let [before, after] = node.signs.each_ref().map(unseen);
            @match (self.feed, node.signs.concat().trim()) {
                (false, _) => (part),
                (_, "") => { (before) (part) (after) }
                _ => mrow { (before) (part) (after) },
            }
        } }
    }

    fn title(&mut self, node: &'a Node) -> Markup {
        (self.copy, self.in_link) = (true, true);
        let title = self.inside(node);
        (self.copy, self.in_link) = (false, false);
        title
    }

    /// As Typst made it, with `more` at the end.
    fn element(&mut self, node: &'a Node, more: Markup) -> Markup {
        let kind = HtmlTag::intern(&node.tag).unwrap();
        let name = if is_mathml(kind) && node.get("notation").is_some() { "menclose" } else { &node.tag };
        let mut written = format!("<{name}");
        for (attribute, value) in node.attributes.iter().filter(|(attribute, _)| attribute != "id" || !self.copy) {
            let value = if attribute == "href" { &resolve(&self.page.url, value) } else { value };
            written += &format!(r#" {attribute}="{}""#, html!((value)).0);
        }
        let inside = if tag::is_raw(kind) { plain(&node.children) } else { self.inside(node).0 };
        PreEscaped(if tag::is_void(kind) { written + ">" } else { format!("{written}>{inside}{}</{name}>", more.0) })
    }

    fn report(&self, node: &Node, problem: &str) {
        self.build.report(format!("{}: {problem}", self.build.desk.place(node.span, &self.page.path)));
    }

    /// Reports what went wrong, and shows it where it did.
    fn attempt(&mut self, node: &'a Node) -> Markup {
        let around = (self.in_link, self.verse, self.in_figure, self.in_formula, self.room);
        let written = self.write(node).unwrap_or_else(|error| {
            self.report(node, &format!("{error:#}"));
            html! { span.problem { (format!("{error:#}")) } }
        });
        (self.in_link, self.verse, self.in_figure, self.in_formula, self.room) = around;
        written
    }

    fn write(&mut self, node: &'a Node) -> Result<Markup> {
        let id = node.get("id").filter(|_| !self.copy);
        Ok(match node.tag.as_str() {
            "" if self.in_formula => html! { (formula::spoken(&node.text)) },
            "" => html! { (node.text) },
            "a" if replace(&mut self.in_link, true) => self.inside(node),
            "p" if self.verse => html! { p id=[id] {
                @for line in node.children.split(|node| node.tag == "br") { span.line { (self.all(line)) br; } }
            } },
            "mtable" if node.children.is_empty() => html! { mrow {} },
            "li" => self.item(node),
            "table" => html! { div.table { (self.element(node, html!())) } },
            "figure" => {
                self.in_figure = true;
                self.element(node, html!())
            }
            "math" => self.formula(node, id)?,
            "pre" => self.code(node, Options::default())?,
            "x-code" => self.code(node, Options::new(node)?)?,
            "x-note" if self.copy => html!(),
            "x-note" if self.in_link => bail!("a note cannot be inside a link"),
            "x-note" => match node.option::<Option<String>>("again")? {
                Some(label) => self.cite(labelled(&self.page.body.nodes, &label).context("there is no such note")?),
                None => self.cite(&node.children[0]),
            },
            "x-quote" => {
                let quoted = &node.children[0];
                self.verse = quoted.count(&["br"]) > 0;
                let short = quoted.count(&["code", "math"]) == 0 && tree::words(&quoted.children) <= 30;
                html! { figure.quote.large[self.verse || short] id=[id] { (self.inside(node)) } }
            }
            "x-aside" => {
                let kind: String = node.option("kind")?;
                let inside = html! { @if kind != "Important" { p.label { b { (kind) } } } (self.inside(node)) };
                html! { @if self.feed { blockquote { (inside) } } @else {
                    section.aside.(kind.to_lowercase()) id=[id] role="note" { (inside) }
                } }
            }
            "x-wide" | "x-margin" => {
                self.room = if node.tag == "x-wide" { WIDE } else { MARGIN };
                html! { section class=(node.tag[2..]) id=[id] { (self.inside(node)) } }
            }
            "x-picture" if self.in_link || node.is_inline() => self.picture(node, false)?,
            "x-picture" | "x-row" | "x-video" | "x-explainer" | "x-drawing"
                if !node.is_inline() && !replace(&mut self.in_figure, true) =>
            {
                html! { figure id=[id] { (self.write(node)?) } }
            }
            "x-picture" => self.picture(node, true)?,
            "x-row" => html! { div.row { (self.inside(node)) } },
            "x-video" => self.video(&node.option::<String>("source")?, plain(&node.children).trim())?,
            "x-explainer" => self.explainer(node, plain(&node.children).trim())?,
            "x-drawing" => {
                let alt: Option<String> = node.option("alt")?;
                let alt = self.alt(node, alt.as_deref());
                match &node.children[..] {
                    [frame, ..] if frame.tag == "x-frame" => self.drawing(drawn(frame)?, alt, false)?,
                    graph => self.drawing(&programs::diagram(self.build.tools, plain(graph).trim())?, alt, true)?,
                }
            }
            "x-frame" => self.drawing(drawn(node)?, "", false)?,
            _ => match node.level() {
                Some(level) => self.heading(node, level),
                None => self.element(node, html!()),
            },
        })
    }

    fn cite(&mut self, note: &'a Node) -> Markup {
        let index = self.notes.iter().position(|(cited, ..)| std::ptr::eq(*cited, note)).unwrap_or(self.notes.len());
        let (number, earlier) = (index + 1, self.notes.get(index).map_or(0, |(_, times, _)| *times));
        if earlier == 0 {
            // Notes can have notes.
            self.notes.push((note, 1, String::new()));
            (self.verse, self.in_figure, self.room) = (false, false, COLUMN);
            let text = self.inside(note);
            self.notes[index].2 =
                if note.children.iter().all(Node::is_inline) { html! { p { (text) } }.0 } else { text.0 };
        } else {
            self.notes[index].1 += 1;
        }
        html! {
            sup.ref id={ "fnref" @if earlier > 0 { (earlier) } ":" (number) } {
                a href={ "#fn:" (number) } role="doc-noteref" { (number) }
            }
        }
    }

    fn heading(&mut self, node: &'a Node, level: usize) -> Markup {
        let id = node.get("id").unwrap();
        if level < 2 {
            let section = Section { id: id.into(), title: self.title(node), parts: Vec::new(), level };
            match self.contents.last_mut() {
                Some(parent) if parent.level < level => parent.parts.push(section),
                _ => self.contents.push(section),
            }
        }
        let pin = html! { @if !self.feed { a.pin href={ "#" (id) } aria-label="Link to this section" {} } };
        self.element(node, pin)
    }

    /// One that starts with `[x]` or `[ ]` is a task.
    fn item(&mut self, node: &'a Node) -> Markup {
        let item = self.element(node, html!());
        let text = item.0.trim_start_matches("<li>").trim_start_matches("<p>");
        let Some(mark) = ["[x] ", "[X] ", "[ ] "].into_iter().find(|mark| text.starts_with(mark)) else { return item };
        let tick = html! { input.task type="checkbox" checked[mark != "[ ] "] disabled aria-label="Done"; };
        PreEscaped(item.0.replacen(mark, &tick.0, 1))
    }

    fn formula(&mut self, node: &'a Node, id: Option<&str>) -> Result<Markup> {
        let number = given(node.get("number").context("formulas are written between `$`")?);
        let alone = replace(&mut self.in_formula, true) || self.feed;
        let spoken = html! { math display=[node.get("display")] {
            (self.inside(node))
            @if let Some(number) = number.filter(|_| alone) { mspace width="2em" {} mtext id=[id] { (number) } }
        } };
        if alone {
            return Ok(spoken);
        }
        let mut drawn: String = themed(drawn(node)?).split_inclusive("</svg>").map(|piece| self.own(piece)).collect();
        if self.in_link {
            drawn = drawn.split("</a>").flat_map(|part| part.split("<a ").next()).collect();
        }
        let source = node.get("source").map_or_else(|| formula::flat(node), Into::into);
        let set = html! {
            span.formula id=[id.filter(|_| !node.is_displayed())] translate="no" data-source=(source) {
                (PreEscaped(drawn)) span.spoken { (spoken) }
            }
        };
        Ok(html! { @if !node.is_displayed() { (set) } @else {
            span.display id=[id] { (set) @if let Some(number) = number { span.number { (number) } } }
        } })
    }

    fn code(&mut self, node: &'a Node, options: Options) -> Result<Markup> {
        let (block, notes) = match node.children.as_slice() {
            _ if node.tag == "pre" => (node, &[][..]),
            [block] if block.tag == "pre" => (block, &[][..]),
            [block, notes] if block.tag == "pre" && notes.tag == "dl" => (block, notes.children.as_chunks().0),
            _ => bail!("`code` takes one block, and then its notes"),
        };
        let (language, code) = (block.get("lang").unwrap_or_default(), plain(&block.children));
        let label = node.get("id").or(block.get("id"));
        ensure!(language != "dot", "a graph goes in `#diagram(alt: \"…\")[…]`");
        ensure!(!self.in_link, "a block of code cannot be inside a link");
        let names: Vec<_> = notes.iter().map(|[name, _]| plain(&name.children)).collect();
        let listing = Listing::new(language, &options, &code, &names);
        let listing = listing.with_context(|| options.title.clone().unwrap_or("code".into()))?;
        if self.feed {
            return Ok(listing.portable(&mut |note| self.cite(&notes[note][1])));
        }
        let id = match (label, &options.title) {
            (Some(label), _) => label.into(),
            (_, Some(title)) => self.ids.like(&slugify(&title.replace(['.', '/'], "-"))),
            _ => self.ids.numbered("code"),
        };
        // Servers take `.rs` for XML and `.ts` for video.
        let source =
            options.whole.then(|| self.build.output.asset(&format!("{id}.txt"), File::Made(code.clone().into())));
        Ok(listing.render(&id, source.transpose()?.as_deref(), &mut |note| self.cite(&notes[note][1])))
    }

    fn picture(&mut self, node: &Node, framed: bool) -> Result<Markup> {
        let [source, kind, size, crop] = ["src", "kind", "size", "shape"].map(|name| node.get(name).unwrap());
        let (alt, whole) = (self.alt(node, node.get("alt")), published(source));
        // Feeds get the files as they are, whose addresses last.
        let prints = if matches!(kind, "jpeg" | "png") && !self.feed { self.prints(source.as_ref())? } else { vec![] };
        let usual = prints.iter().rfind(|(width, _)| *width <= 1300).map(|(_, url)| url);
        let choices: Vec<_> = prints.iter().map(|(width, url)| format!("{url} {width}w")).collect();
        let sizes = format!("(min-width: 112em) {:.1}em, min({:.1}em, 90vw)", self.room * 1.6, self.room * 1.25);
        let [width, height] = ["width", "height"].map(|side| node.get(side).filter(|_| !self.feed));
        let framed = framed && usual.is_some();
        let picture = html! {
            img class=[(kind != "jpeg").then_some("sheer")] src=(usual.unwrap_or(&whole)) alt=(alt)
                srcset=[given(&choices.join(", "))] sizes=[usual.and(Some(sizes))] width=[width] height=[height]
                style=[given(&[if framed { "" } else { size }, crop].concat())] loading="lazy" decoding="async";
        };
        if kind != "themed" || self.feed {
            return Ok(html! { @if framed { a.photo href=(whole) style=(size) { (picture) } } @else { (picture) } });
        }
        // To take its colour from the text it has to be in the page, but its names and styles are no one else's.
        let svg = read_text(Path::new(source))?.replace(" id=\"", " id =\"").replace("href=\"#", "href =\"#");
        let (ratio, size) = size.split_once(';').unwrap();
        Ok(html! { span.drawing role=[role(&svg, alt)] aria-label=[given(alt)] style=(ratio) {
            template shadowrootmode="open" {
                style { (PreEscaped(format!(":host > svg {{ {THEMED} {size}{crop} }}"))) }
                (PreEscaped(&svg[svg.find("<svg").unwrap_or(0)..]))
            }
            (picture)
        } })
    }

    fn alt<'t>(&self, node: &Node, alt: Option<&'t str>) -> &'t str {
        alt.unwrap_or_else(|| {
            self.report(node, "this needs an `alt`, for those who cannot see it");
            ""
        })
    }

    fn prints(&self, photo: &Path) -> Result<Vec<(u32, String)>> {
        let name = slugify(&photo.file_stem().unwrap().to_string_lossy());
        let publish = |width, file| self.build.output.asset(&format!("{name}-{width}.webp"), File::Copied(file));
        let prints = photo::print(&self.build.tools.cache, photo)?.into_iter();
        prints.map(|(width, file)| Ok((width, publish(width, file)?))).collect()
    }

    fn drawing(&mut self, svg: &str, alt: &str, graph: bool) -> Result<Markup> {
        if self.feed {
            let url = format!("/drawings{}{}.svg", self.page.url, self.ids.numbered("drawing"));
            let file = File::Made(svg.replacen('>', "><style>svg { background: #fff }</style>", 1).into());
            return Ok(html! { img src=(self.build.output.lasting(&url, file)?) alt=(alt); });
        }
        let drawn = PreEscaped(self.own(&themed(svg)));
        Ok(html! { span.drawing.diagram[graph] role=[role(svg, alt)] aria-label=[given(alt)] { (drawn) } })
    }

    /// Another drawing may have the same names for its parts.
    fn own(&mut self, svg: &str) -> String {
        let ids: HashSet<_> = values(svg, " id=\"").collect();
        let own = if ids.is_empty() { String::new() } else { self.ids.numbered("drawing") };
        let mut parts = svg.split('#');
        let (mut drawn, mut used) = (parts.next().unwrap().to_owned(), HashSet::new());
        for part in parts {
            let id = ids.get(part.split(|c: char| !c.is_alphanumeric() && !"-_".contains(c)).next().unwrap());
            drawn.extend(["#", id.map_or("", |_| &own), part]);
            used.extend(id);
        }
        used.iter().fold(drawn, |svg, id| svg.replace(&format!(" id=\"{id}\""), &format!(" id=\"{own}{id}\"")))
    }

    fn video(&self, source: &str, alt: &str) -> Result<Markup> {
        let file = source.split(['?', '#']).next().unwrap().to_lowercase();
        let plays = [".mp4", ".webm", ".m4v", ".mov"].iter().any(|kind| file.ends_with(kind));
        ensure!(plays || youtube(source).is_some(), "a video is an MP4 or a WebM, or is on YouTube");
        Ok(html! { @match youtube(source) {
            Some(_) if self.feed => a href=(source) { (alt) },
            Some((id, start)) => iframe.video src={ "https://www.youtube-nocookie.com/embed/" (id) "?start=" (start) }
                title=(alt) loading="lazy" allow="encrypted-media; fullscreen; picture-in-picture" {},
            None => video src=(resolve(&self.page.url, source)) aria-label=(alt) preload="metadata" controls playsinline {},
        } })
    }

    fn explainer(&self, node: &Node, alt: &str) -> Result<Markup> {
        if self.feed {
            return Ok(html! { a href=(self.page.url) { (alt) } });
        }
        let (name, options): (String, String) = (node.option("name")?, node.option("options")?);
        let program = read(&self.build.programs.join(format!("{name}.wasm")));
        let program = program.ok().with_context(|| format!("there is no crates/explainers/src/bin/{name}.rs"))?;
        let view = programs::first_view(&program, &options).with_context(|| format!("`{name}` did not start"))?;
        let program = self.build.output.asset(&format!("{name}.wasm"), File::Made(program))?;
        Ok(html! {
            div.drawing.explainer role="group" aria-label=(alt) data-program=(program) data-options=(options) {
                (PreEscaped(view))
            }
        })
    }
}
