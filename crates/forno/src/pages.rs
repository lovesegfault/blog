//! The pages around the text.

use anyhow::Result;
use collision::{Kind, Source};
use jiff::ZonedDateTime;
use maud::{DOCTYPE, Markup, PreEscaped, html};
use mural::{HOME, LABELS, POST};

use crate::article::{Article, Section};
use crate::content::{Comments, NOT_FOUND, Page, address};
use crate::figure::{card, figure};
use crate::formula::gather;
use crate::links::encode;
use crate::site::{Build, Taxonomy, Term};

struct Head<'a> {
    /// The home page has only the site's.
    title: Option<&'a str>,
    url: &'a str,
    description: &'a str,
    card: &'a str,
    feed: &'a str,
    published: Option<&'a ZonedDateTime>,
}

const BEFORE_PAINT: &str = r#"document.documentElement.classList.add("js");
try { if (localStorage.theme) document.documentElement.dataset.theme = localStorage.theme } catch {}"#;

fn layout(build: &Build, head: &Head, body: Markup) -> Markup {
    let (config, style) = (build.config, build.style);
    html! {
        (DOCTYPE)
        html lang=(config.language) {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1, viewport-fit=cover";
                // Comments are found by this title.
                title { @if let Some(title) = head.title { (title) " :: " } (config.title) }
                meta name="description" content=(head.description);
                meta name="author" content=(config.author);
                @if head.url != NOT_FOUND {
                    link rel="canonical" href={ (config.url) (head.url) };
                    meta property="og:url" content={ (config.url) (head.url) };
                }
                meta property="og:site_name" content=(config.title);
                meta property="og:title" content=(head.title.unwrap_or(&config.title));
                meta property="og:description" content=(head.description);
                meta property="og:image" content={ (config.url) (head.card) };
                meta property="og:type" content=(if head.published.is_some() { "article" } else { "website" });
                @if let Some(date) = head.published {
                    meta property="article:published_time" content=(date.timestamp());
                }
                meta name="twitter:card" content="summary_large_image";
                meta name="twitter:creator" content=(config.twitter);
                link rel="alternate" type="application/rss+xml" href=(head.feed) title=(config.title);
                link rel="icon" href="/favicon.png";
                link rel="apple-touch-icon" href="/apple-touch-icon.png";
                @for font in &style.fonts[..if head.title.is_some() { 3 } else { 2 }] {
                    link rel="preload" as="font" type="font/woff2" href=(font) crossorigin;
                }
                link rel="stylesheet" href=(style.sheet);
                script { (PreEscaped(BEFORE_PAINT)) }
                script type="module" src=(style.script) {}
            }
            body.titled[head.title.is_some()] { (PreEscaped(gather(&body.0))) }
        }
    }
}

fn links() -> Markup {
    html! {
        nav.links.voice aria-label="Site" {
            a href="/about/" { "About" }
            a href="/tags/" { "Tags" }
            a href="/index.xml" { "RSS" }
            button.dusk type="button" aria-label="Switch between day and night" {}
        }
    }
}

fn day(date: &ZonedDateTime) -> Markup {
    html! { time datetime=(date.date()) { (date.strftime("%-d %B %Y")) } }
}

fn list(posts: &[&Article]) -> Markup {
    let year = |post: &Article| post.page.date.as_ref().map(ZonedDateTime::year);
    html! {
        ol.posts {
            @for group in posts.chunk_by(|a, b| year(a) == year(b)) {
                @for (row, post) in group.iter().enumerate() {
                    li.first[row == 0].draft[post.page.draft] {
                        a href=(post.page.url) data-lights=(post.page.slug) {
                            @if let Some(date) = &post.page.date { time.voice datetime=(date.date()) { (date.year()) } }
                            " " span { (post.title) }
                        }
                    }
                }
            }
        }
    }
}

fn label(url: &str, name: &str, posts: &[&Article]) -> Markup {
    let (glaze, _) = LABELS[blake3::hash(url.as_bytes()).as_bytes()[0] as usize % LABELS.len()];
    html! { li {
        a class=(glaze) href=(url) data-lights=[(!posts.is_empty()).then(|| names(posts).join(" "))] {
            (name) @if posts.len() > 1 { " " small { (posts.len()) } }
        }
    } }
}

fn names<'a>(posts: &[&'a Article]) -> Vec<&'a str> {
    posts.iter().map(|post| post.page.slug.as_str()).collect()
}

/// All the posts at once, in about as many tracks as there is room for.
fn everything(posts: &[&Article], tracks: usize) -> Source {
    let each = (tracks / posts.len().max(1)).clamp(2, 15);
    Source::Site(posts.iter().rev().map(|post| post.paragraphs.thinned(each)).collect())
}

fn contents(sections: &[Section]) -> Markup {
    html! { ol { @for section in sections { li {
        a href={ "#" (section.id) } { (section.title) }
        @if !section.parts.is_empty() { (contents(&section.parts)) }
    } } } }
}

fn comments(build: &Build, page: &Page) -> Markup {
    let (issue, thread) = match page.comments {
        Comments::Open(true) if !page.draft => {
            (None, format!("?q={}", encode(&format!("is:issue in:title \"{}\"", page.title))))
        }
        Comments::Open(_) => return html!(),
        Comments::Issue(number) => (Some(number), format!("/{number}")),
    };
    let repository = &build.config.comments;
    html! {
        section.comments aria-label="Comments" data-repo=(repository) data-issue=[issue] {
            a.voice href={ "https://github.com/" (repository) "/issues" (thread) } { "Comments are on GitHub" }
        }
    }
}

/// Any page but the first: a title that the arrow points at, a column, and the shore.
fn sheet(build: &Build, heading: Markup, figure: Markup, column: Markup, end: Markup) -> Markup {
    let config = build.config;
    html! {
        // The names that headings get have no capitals.
        a.skip href="#Text" { "Skip to the text" }
        header.top { a href="/" { (config.title) } (links()) }
        main.sheet {
            hgroup { (heading) }
            (figure)
            (column)
        }
        footer {
            div.coast {}
            div.deep {
                (end)
                p.where.voice {
                    span { (config.farm.at) }
                    span { "© " (config.author) " · " a href="/colophon/" { "Colophon" } }
                }
            }
        }
    }
}

/// Drafts are left out of cards, which are too slow to draw at every save.
pub fn site_card(build: &Build, posts: &[&Article]) -> Result<String> {
    let published: Vec<_> = posts.iter().copied().filter(|post| !post.page.draft).collect();
    card(build, &everything(&published, 170), None, "/")
}

pub fn home(build: &Build, posts: &[&Article], card: &str) -> Result<Markup> {
    let config = build.config;
    let head = Head { title: None, url: "/", description: &config.subtitle, card, feed: "/index.xml", published: None };
    let body = html! {
        div.home {
            header { h1 { (config.title) } p { (config.subtitle) } (links()) }
            main { (list(posts)) (figure(build, &everything(posts, 170), &[], &HOME)?) }
        }
    };
    Ok(layout(build, &head, body))
}

pub fn article(build: &Build, article: &Article, newer: Option<&Article>, older: Option<&Article>) -> Result<Markup> {
    let page = article.page;
    let mut shape = article.sentences.thinned(70);
    if shape.pieces.iter().all(|piece| piece.kind != Kind::Text) {
        shape.pieces.clear();
    }
    let event = Source::Post(shape);
    let head = Head {
        title: Some(&page.title),
        url: &page.url,
        description: if article.summary.is_empty() { &build.config.subtitle } else { &article.summary },
        card: &if page.draft { String::new() } else { card(build, &event, Some(&article.name), &page.url)? },
        feed: "/index.xml",
        published: page.date.as_ref(),
    };
    let heading = html! {
        h1 lang=(page.language) { (article.title) }
        @if let Some(date) = &page.date {
            p.voice {
                @if page.draft { span { "draft" } " " }
                (day(date)) " "
                @for name in &page.categories { a href={ "/categories/" (address(name)) "/" } { (name) } " " }
                span { (article.minutes) " min" }
                @if let Some(date) = &page.updated { " " span { "updated " (day(date)) } }
            }
        }
    };
    let column = html! {
        @if article.contents.iter().map(|section| 1 + section.parts.len()).sum::<usize>() >= 3 {
            nav.contents.voice aria-label="Contents" {
                details { summary { "Contents" } (contents(&article.contents)) }
            }
        }
        article.prose #Text lang=(page.language) data-page=(page.slug) { (article.body) }
        @if !page.tags.is_empty() {
            ul.labels aria-label="Tags" {
                @for tag in &page.tags { (label(&format!("/tags/{}/", address(tag)), tag, &[])) }
            }
        }
    };
    let end = html! {
        @if newer.or(older).is_some() {
            nav.flip aria-label="Other posts" {
                @for (post, rel, word) in [(newer, "next", "Newer"), (older, "prev", "Older")] {
                    @if let Some(post) = post {
                        a href=(post.page.url) rel=(rel) { span.voice { (word) } " " (post.title) }
                    }
                }
            }
        }
        (comments(build, page))
    };
    Ok(layout(build, &head, sheet(build, heading, figure(build, &event, &[], &POST)?, column, end)))
}

pub fn terms(build: &Build, of: &Taxonomy, other: &Taxonomy, posts: &[&Article], card: &str) -> Result<Markup> {
    let description = format!("Posts by {}", of.one);
    let (title, url) = (Some(of.title), of.url);
    let head = Head { title, url, description: &description, card, feed: "/index.xml", published: None };
    let heading = html! { h1 { (of.title) } p.voice { a href=(other.url) { "or by " (other.one) } } };
    let column = html! {
        ul.labels.all #Text { @for term in of.terms.values() { (label(&term.url, term.name, &term.posts)) } }
    };
    let figure = figure(build, &everything(posts, 55), &[], &POST)?;
    Ok(layout(build, &head, sheet(build, heading, figure, column, html!())))
}

pub fn term(build: &Build, of: &Taxonomy, term: &Term, posts: &[&Article], card: &str) -> Result<Markup> {
    let (description, feed) = (format!("Posts under {}", term.name), format!("{}index.xml", term.url));
    let (title, url) = (Some(term.name), &term.url);
    let head = Head { title, url, description: &description, card, feed: &feed, published: None };
    let heading = html! { h1 { (term.name) } p.voice { a href=(of.url) { (of.one) } " " a href=(feed) { "RSS" } } };
    let column = html! { div #Text { (list(&term.posts)) } };
    let figure = figure(build, &everything(posts, 110), &names(&term.posts), &POST)?;
    Ok(layout(build, &head, sheet(build, heading, figure, column, html!())))
}

pub fn redirect(build: &Build, to: &str) -> Markup {
    html! {
        (DOCTYPE)
        html lang=(build.config.language) {
            meta charset="utf-8";
            title { (build.config.title) }
            link rel="canonical" href={ (build.config.url) (to) };
            meta http-equiv="refresh" content={ "0; url=" (to) };
            a href=(to) { (to) }
        }
    }
}

const XML: PreEscaped<&str> = PreEscaped(r#"<?xml version="1.0" encoding="utf-8"?>"#);

/// The feed for the page at `url`.
pub fn feed(build: &Build, name: Option<&str>, url: &str, posts: &[&Article]) -> Markup {
    let config = build.config;
    let date =
        |post: &Article| post.page.date.as_ref().map(|date| date.strftime("%a, %d %b %Y %H:%M:%S %z").to_string());
    let feed = html! {
        (XML)
        rss version="2.0" xmlns:atom="http://www.w3.org/2005/Atom" xmlns:content="http://purl.org/rss/1.0/modules/content/" {
            channel {
                title { @if let Some(name) = name { (name) " on " } (config.title) }
                link { (config.url) (url) }
                description { (config.subtitle) }
                language { (config.language) }
                copyright { "© " (config.author) }
                atom:link href={ (config.url) (url) "index.xml" } rel="self" type="application/rss+xml" {}
                @for post in posts { item {
                    title { (post.page.title) }
                    link { (config.url) (post.page.url) }
                    pubDate { (date(post).unwrap_or_default()) }
                    // Readers tell new posts from old ones by this.
                    guid isPermaLink="false" { (post.page.url) }
                    description { (html!((post.summary)).0) }
                    content:encoded { (post.feed) }
                } }
            }
        }
    };
    // XML has no way of writing these.
    PreEscaped(feed.0.replace(|it| it < ' ' && !"\t\n\r".contains(it) || "\u{FFFE}\u{FFFF}".contains(it), ""))
}

pub fn sitemap(build: &Build, urls: &[(&str, Option<&ZonedDateTime>)]) -> Markup {
    html! {
        (XML)
        urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9" {
            @for (url, changed) in urls { url {
                loc { (build.config.url) (url) }
                @if let Some(date) = changed { lastmod { (date.timestamp()) } }
            } }
        }
    }
}
