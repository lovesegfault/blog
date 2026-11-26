//! A page's text, as HTML for the site and for feeds.

use std::collections::HashSet;
use std::mem::replace;
use std::path::Path;

use anyhow::{Context, Result, bail};
use collision::Post;
use maud::{Markup, PreEscaped, html};
use typst_html::{HtmlTag, tag};

use crate::code::{Listing, Options};
use crate::content::{Page, published, slugify};
use crate::links::{absolute, resolve, values};
use crate::output::{File, read, read_text};
use crate::site::Build;
use crate::tree::{self, Ids, Node, plain};
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
    /// Feed readers have none of the site's styles or scripts.
    feed: bool,
    /// For titles, which are shown in more than one place.
    copy: bool,
    in_link: bool,
    verse: bool,
    in_figure: bool,
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

/// Has the colours that `blog.typ` names follow the theme.
fn themed(html: &str, end: char) -> String {
    let colours =
        [("000000", "ink"), ("ffffff", "paper"), ("a83a10", "ember"), ("17389c", "cobalt"), ("6b5c50", "dim")];
    colours
        .iter()
        .fold(html.into(), |html, (day, name)| html.replace(&format!("#{day}{end}"), &format!("var(--{name}){end}")))
}

fn given(text: &str) -> Option<&str> {
    Some(text).filter(|text| !text.is_empty())
}

fn youtube(url: &str) -> Option<&str> {
    let id = ["https://www.youtube.com/watch?v=", "https://youtu.be/"].iter().find_map(|p| url.strip_prefix(p))?;
    id.split(['&', '?']).next()
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
        self.all(&node.children)
    }

    fn title(&mut self, node: &'a Node) -> Markup {
        (self.copy, self.in_link) = (true, true);
        let title = self.inside(node);
        (self.copy, self.in_link) = (false, false);
        title
    }

    /// As Typst made it, with `more` at the end.
    fn element(&mut self, node: &'a Node, more: Markup) -> Markup {
        let (name, kind) = (&node.tag, HtmlTag::intern(&node.tag).unwrap());
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
        let around = (self.in_link, self.verse, self.in_figure, self.room);
        let written = self.write(node).unwrap_or_else(|error| {
            self.report(node, &format!("{error:#}"));
            html! { span.problem { (format!("{error:#}")) } }
        });
        (self.in_link, self.verse, self.in_figure, self.room) = around;
        written
    }

    fn write(&mut self, node: &'a Node) -> Result<Markup> {
        let id = node.get("id").filter(|_| !self.copy);
        Ok(match node.tag.as_str() {
            "" => html! { (node.text) },
            "a" if replace(&mut self.in_link, true) => self.inside(node),
            "p" if self.verse => html! { p id=[id] {
                @for line in node.children.split(|node| node.tag == "br") { span.line { (self.all(line)) br; } }
            } },
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
                    div.aside.(kind.to_lowercase()) id=[id] role="note" { (inside) }
                } }
            }
            "x-wide" | "x-margin" => {
                self.room = if node.tag == "x-wide" { WIDE } else { MARGIN };
                html! { div class=(node.tag[2..]) id=[id] { (self.inside(node)) } }
            }
            "x-picture" if self.in_link || node.is_inline() => self.picture(node, false)?,
            "x-picture" | "x-row" | "x-video" | "x-explainer" | "x-drawing" if !replace(&mut self.in_figure, true) => {
                html! { figure id=[id] { (self.write(node)?) } }
            }
            "x-picture" => self.picture(node, true)?,
            "x-row" => html! { div.row { (self.inside(node)) } },
            "x-video" => self.video(&node.option::<String>("source")?, plain(&node.children).trim()),
            "x-explainer" => self.explainer(node, plain(&node.children).trim())?,
            "x-drawing" => {
                let alt: Option<String> = node.option("alt")?;
                self.frame(&node.children[0], self.alt(node, alt.as_deref()))?
            }
            "x-frame" => self.frame(node, "")?,
            _ => match node.level() {
                Some(level) => self.heading(node, level),
                None => self.element(node, html!()),
            },
        })
    }

    fn cite(&mut self, note: &'a Node) -> Markup {
        let index = self.notes.iter().position(|(cited, ..)| std::ptr::eq(*cited, note)).unwrap_or(self.notes.len());
        if index == self.notes.len() {
            // Notes can have notes.
            self.notes.push((note, 0, String::new()));
            (self.verse, self.in_figure, self.room) = (false, false, COLUMN);
            let text = self.inside(note);
            self.notes[index].2 =
                if note.children.iter().all(Node::is_inline) { html! { p { (text) } }.0 } else { text.0 };
        }
        let (number, earlier) = (index + 1, self.notes[index].1);
        self.notes[index].1 += 1;
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
        let tick = html! { input type="checkbox" checked[mark != "[ ] "] disabled aria-label="Done"; };
        PreEscaped(item.0.replacen(mark, &tick.0, 1))
    }

    fn formula(&self, node: &Node, id: Option<&str>) -> Result<Markup> {
        let formula = self.build.tools.math.render(node.get("mathml").context("formulas are written between `$`")?)?;
        let number = node.get("number").and_then(given);
        if self.feed {
            let number = html! { @if let Some(number) = number { mspace width="2em" {} mtext id=[id] { (number) } } };
            return Ok(PreEscaped(formula.mathml.replace("</math>", &(number.0 + "</math>"))));
        }
        self.build.math.lock().unwrap().extend(formula.rules.iter().cloned());
        let set = html! { (PreEscaped(themed(&formula.html, ';'))) span.spoken { (PreEscaped(&formula.mathml)) } };
        Ok(html! {
            @if node.is_displayed() {
                span.display id=[id] { span.formula { (set) } @if let Some(number) = number { span.number { (number) } } }
            } @else {
                span.formula id=[id] { (set) }
            }
        })
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
        if language == "dot" {
            let drawing = programs::diagram(self.build.tools, &code, &self.ids.numbered("diagram"))?;
            let drawing = html! { div.drawing.diagram { (PreEscaped(drawing)) } };
            return Ok(html! { @if self.in_figure { (drawing) } @else { figure id=[label] { (drawing) } } });
        }
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
        let (source, alt) = (node.get("src").unwrap(), self.alt(node, node.get("alt")));
        let size = ["height", "width"].map(|side| node.get(side).unwrap_or_default());
        // Feeds get the files as they are, whose addresses last.
        if photo::is_photo(Path::new(source)) && !self.feed {
            return self.photo(source, alt, size, framed);
        } else if photo::is(Path::new(source), &["svg"]) && !self.feed {
            // One that takes its colour from the text has to be part of the page.
            let svg = read_text(Path::new(source))?;
            if let Some((_, svg)) = svg.split_once("<svg").filter(|_| svg.contains("currentColor")) {
                return Ok(
                    self.drawing(&format!(r#"<svg style="{}"{svg}"#, size.concat()).replace(r#" style="""#, ""), alt)
                );
            }
        }
        Ok(html! { img src=(published(source)) alt=(alt) style=[given(&size.concat())] loading="lazy"; })
    }

    fn alt<'t>(&self, node: &Node, alt: Option<&'t str>) -> &'t str {
        alt.unwrap_or_else(|| {
            self.report(node, "this needs an `alt`, for those who cannot see it");
            ""
        })
    }

    fn photo(&self, source: &str, alt: &str, [height, width]: [&str; 2], framed: bool) -> Result<Markup> {
        let photo = photo::print(&self.build.tools.cache, Path::new(source))?;
        let name = slugify(&Path::new(source).file_stem().unwrap().to_string_lossy());
        let mut prints = Vec::new();
        for (width, file) in photo.prints {
            prints.push((width, self.build.output.asset(&format!("{name}-{width}.webp"), File::Copied(file))?));
        }
        let (_, usual) = prints.iter().rfind(|(width, _)| *width <= 1300).unwrap();
        let choices: Vec<_> = prints.iter().map(|(width, url)| format!("{url} {width}w")).collect();
        let sizes = format!("(min-width: 112em) {:.1}em, min({:.1}em, 90vw)", self.room * 1.6, self.room * 1.25);
        let picture = html! {
            img src=(usual) srcset=(choices.join(", ")) sizes=(sizes) width=(photo.width) height=(photo.height)
                alt=(alt) style=[given(&[height, if framed { "" } else { width }].concat())] loading="lazy" decoding="async";
        };
        let shape = format!("--ratio:{:.3};{width}", photo.width as f32 / photo.height as f32);
        Ok(html! { @if framed { a.photo href=(published(source)) style=(shape) { (picture) } } @else { (picture) } })
    }

    fn frame(&mut self, frame: &Node, alt: &str) -> Result<Markup> {
        if self.feed {
            let drawing = self.build.output.asset("drawing.svg", File::Made(frame.text.clone().into()))?;
            return Ok(html! { img src=(drawing) alt=(alt); });
        }
        Ok(self.drawing(&themed(&frame.text, '"'), alt))
    }

    fn drawing(&mut self, svg: &str, alt: &str) -> Markup {
        // Another drawing may have the same names for its parts.
        let (ids, own): (HashSet<_>, _) = (values(svg, " id=\"").collect(), self.ids.numbered("drawing"));
        let mut parts = svg.split('#');
        let (mut drawn, mut used) = (parts.next().unwrap().to_owned(), HashSet::new());
        for part in parts {
            let id = ids.get(part.split(|c: char| !c.is_alphanumeric() && !"-_".contains(c)).next().unwrap());
            drawn.extend(["#", id.map_or("", |_| &own), part]);
            used.extend(id);
        }
        let drawn =
            used.iter().fold(drawn, |svg, id| svg.replace(&format!(" id=\"{id}\""), &format!(" id=\"{own}{id}\"")));
        html! { span.drawing role=[given(alt).map(|_| "img")] aria-label=[given(alt)] { (PreEscaped(drawn)) } }
    }

    fn video(&self, source: &str, alt: &str) -> Markup {
        html! { @match youtube(source) {
            Some(_) if self.feed => a href=(source) { (alt) },
            Some(id) => iframe.video src={ "https://www.youtube-nocookie.com/embed/" (id) } title=(alt) loading="lazy"
                allow="encrypted-media; fullscreen; picture-in-picture" {},
            None => video src=(resolve(&self.page.url, source)) aria-label=(alt) preload="metadata" controls playsinline {},
        } }
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
