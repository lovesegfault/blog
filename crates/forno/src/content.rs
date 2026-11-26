//! What is under `content/`.

use std::fs;
use std::ops::Range;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use jiff::Zoned;
use jiff::fmt::temporal::Pieces;
use jiff::tz::Offset;
use typst::foundations::OneOrMultiple;
use typst::model::Document;
use typst::syntax::{LinkedNode, ast};
use typst_html::HtmlDocument;

use crate::links::encode;
use crate::output::{read_text, walk};
use crate::tree::{Node, Tree, plain};

#[derive(Clone, Copy)]
pub enum Comments {
    /// In the issue named after the page.
    Open(bool),
    /// In this issue, whatever the page is called now.
    Issue(u32),
}

#[derive(Clone)]
pub struct Page {
    /// As it is typed, which is what comments are found by.
    pub title: String,
    pub heading: Node,
    pub body: Tree,
    pub slug: String,
    pub url: String,
    /// Only posts have one.
    pub date: Option<Zoned>,
    pub updated: Option<Zoned>,
    pub draft: bool,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
    pub description: Option<String>,
    pub comments: Comments,
    pub language: String,
    pub path: PathBuf,
    /// Where its pictures and files are, if it has a folder to itself.
    pub folder: Option<PathBuf>,
}

pub const NOT_FOUND: &str = "/404.html";

fn zoned(when: String) -> Result<Zoned> {
    let when = Pieces::parse(&when)?;
    ensure!(when.time_zone_annotation().is_none(), "a date takes an offset, as in `-03:00`");
    let zone = when.to_numeric_offset().unwrap_or(Offset::UTC).to_time_zone();
    Ok(when.date().to_datetime(when.time().unwrap_or_default()).to_zoned(zone)?)
}

/// As GitHub and Hugo name their anchors.
pub fn slugify(text: &str) -> String {
    let keep = |c: char| match c {
        ' ' | '-' => Some('-'),
        c if c.is_alphanumeric() || c == '_' => Some(c),
        _ => None,
    };
    text.trim().chars().filter_map(keep).flat_map(char::to_lowercase).collect()
}

/// What a new post or a tag is called in its address.
pub fn address(name: &str) -> String {
    let accents = || "àáâãäçèéêëìíîïñòóôõöùúûü".chars().zip("aaaaaceeeeiiiinooooouuuu".chars());
    let bare = |letter| accents().find(|(marked, _)| *marked == letter).map_or(letter, |(_, bare)| bare);
    let slug: String = slugify(&name.replace('+', "p").replace('#', "sharp")).chars().map(bare).collect();
    slug.split('-').filter(|word| !word.is_empty()).collect::<Vec<_>>().join("-")
}

/// Where a file is published.
pub fn published(path: &str) -> String {
    let folders =
        [("content/drafts/", "/posts/"), ("content/404/", "/"), ("content/", "/"), ("static/", "/"), ("", "")];
    let url = folders.iter().find_map(|(folder, url)| Some(format!("{url}{}", path.strip_prefix(folder)?))).unwrap();
    ["%", " ", "#", "?"].iter().fold(url, |url, sign| url.replace(sign, &encode(sign)))
}

impl Page {
    pub fn new(path: &Path, document: &HtmlDocument) -> Result<Self> {
        let folder = path.ends_with("index.typ").then(|| path.parent().unwrap().to_owned());
        let name = folder.as_deref().map_or(path.file_stem(), Path::file_name);
        let slug = name.context("a page needs a name")?.to_string_lossy().into_owned();
        ensure!(slug == slugify(&slug), "`{slug}` would not do in an address: try `{}`", slugify(&slug));
        let (mut body, info) = (Tree::new(document), document.info());
        let front = body.nodes.extract_if(.., |node| node.tag == "x-front").next();
        let mut front = front.context("a page starts with `#show: post.with(title: [A Title])`")?;
        let heading = front.children.pop_if(|node| node.tag == "h1").context("the title is not shown")?;
        let typed = info.title.as_deref().filter(|_| heading.count(&["math"]) == 0);
        let date = |name| front.option::<Option<String>>(name)?.map(zoned).transpose();
        let names = |name| {
            let OneOrMultiple::<String>(names) = front.option(name)?;
            ensure!(names.iter().all(|it| !address(it).is_empty()), "`{name}` need letters for their addresses");
            Ok(names)
        };
        let draft = path.starts_with("content/drafts");
        let is_post = draft || path.starts_with("content/posts");
        ensure!(date("date")?.is_some() == is_post, "posts have a `date`, and other pages do not");
        Ok(Self {
            title: typed.unwrap_or(&plain(&heading.children)).trim().into(),
            url: match slug.as_str() {
                _ if is_post => format!("/posts/{slug}/"),
                "404" => NOT_FOUND.to_owned(),
                _ => format!("/{slug}/"),
            },
            date: date("date")?,
            updated: date("updated")?,
            categories: names("categories")?,
            tags: names("tags")?,
            description: info.description.as_ref().map(|_| plain(&front.children).trim().into()),
            comments: front.option("comments").map(Comments::Issue).or(front.option("comments").map(Comments::Open))?,
            language: info.locale.unwrap_or_default().rfc_3066().into(),
            path: path.to_owned(),
            heading,
            body,
            slug,
            draft,
            folder,
        })
    }
}

pub fn is_source(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "typ")
}

/// Each `name.typ` and `name/index.typ`, and what is among the posts and should have been one of those.
pub fn sources(drafts: bool) -> Vec<PathBuf> {
    let folders = ["content", "content/posts", "content/drafts"].into_iter().take(if drafts { 3 } else { 2 });
    let entries = folders.flat_map(fs::read_dir).flatten().flatten().map(|entry| entry.path());
    let source = |path: PathBuf| match (path.is_dir(), path.iter().count() > 2) {
        _ if path.file_name()?.as_encoded_bytes().starts_with(b".") => None,
        (true, true) => (!walk(&path).is_ok_and(|files| files.is_empty())).then(|| path.join("index.typ")),
        (true, false) => Some(path.join("index.typ")).filter(|path| path.is_file()),
        (false, post) => (is_source(&path) || post && path.extension()? == "md").then_some(path),
    };
    entries.filter_map(source).collect()
}

fn now() -> String {
    Zoned::now().strftime("\"%Y-%m-%dT%H:%M:%S%:z\"").to_string()
}

fn named(folder: &str, slug: &str) -> Option<PathBuf> {
    [slug.to_owned(), format!("{slug}.typ")].iter().map(|name| Path::new(folder).join(name)).find(|path| path.exists())
}

/// As markup that means nothing but itself.
fn literal(text: &str) -> String {
    let opening = text.find(|c: char| !c.is_ascii_digit());
    let dash = |at: usize| ["--", "-?"].iter().any(|dash| text[at..].starts_with(dash));
    let means = |at, c| "\\[]#$*_`<@~/".contains(c) || Some(at) == opening && "=+-.".contains(c) || dash(at);
    text.char_indices().flat_map(|(at, c)| means(at, c).then_some('\\').into_iter().chain([c])).collect()
}

/// Starts a draft.
pub fn draft(title: &str) -> Result<PathBuf> {
    let slug = address(title);
    ensure!(!slug.is_empty(), "there is nothing in that title to make an address from");
    if let Some(taken) = named("content/drafts", &slug).or(named("content/posts", &slug)) {
        bail!("{} already exists", taken.display());
    }
    let path = Path::new("content/drafts").join(slug).join("index.typ");
    let front =
        format!("  title: [{}],\n  date: {},\n  categories: \"essay\",\n  tags: (),", literal(title.trim()), now());
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, format!("#import \"/blog.typ\": *\n\n#show: post.with(\n{front}\n)\n"))?;
    Ok(path)
}

/// Moves a draft in with the posts, dated now.
pub fn publish(name: &str) -> Result<PathBuf> {
    let name = Path::new(name.trim_end_matches("/index.typ"));
    let slug = name.file_stem().context("which draft?")?.to_string_lossy();
    let from = named("content/drafts", &slug).with_context(|| format!("there is no draft called `{slug}`"))?;
    if let Some(taken) = named("content/posts", &slug) {
        bail!("{} already exists", taken.display());
    }
    let source = |path: &Path| if from.is_dir() { path.join("index.typ") } else { path.to_owned() };
    let to = Path::new("content/posts").join(from.file_name().unwrap());
    let published = source(&to);
    fs::write(source(&from), stamp(read_text(&source(&from))?, &now())?)?;
    fs::rename(&from, &to)?;
    Ok(published)
}

fn stamp(mut source: String, now: &str) -> Result<String> {
    fn date(node: LinkedNode) -> Option<Range<usize>> {
        match node.cast::<ast::Named>() {
            Some(named) if named.name().as_str() == "date" => node.children().next_back().map(|value| value.range()),
            _ => node.children().find_map(date),
        }
    }
    let markup = typst::syntax::parse(&source);
    let written = LinkedNode::new(&markup).children().filter(|node| node.is::<ast::ShowRule>()).find_map(date);
    source.replace_range(written.context("there is no `date` in `post.with`")?, now);
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_match_the_ones_hugo_made() {
        assert_eq!(slugify("1. sizeof()"), "1-sizeof");
        assert_eq!(slugify("Standard Cognition & Rust in 2020"), "standard-cognition--rust-in-2020");
        assert_eq!(address("Então: C, C++ & C#"), "entao-c-cpp-csharp");
    }

    #[test]
    fn dates_keep_their_own_offset() {
        let shown = |date: &str| zoned(date.into()).unwrap().strftime("%F %R %:z").to_string();
        assert_eq!(shown("2021-04-01T22:04:19-07:00"), "2021-04-01 22:04 -07:00");
        assert_eq!(shown("2021-04-01"), "2021-04-01 00:00 +00:00");
    }

    #[test]
    fn titles_are_not_markup() {
        assert_eq!(literal("1. It's Vec<T> - or --C#."), r"1\. It's Vec\<T> - or \--C\#.");
    }

    #[test]
    fn publishing_only_touches_the_date() {
        let draft = "#f(date: 0)\n#show: post.with(title: [date: x], date: \"0\")";
        assert_eq!(stamp(draft.into(), "1").unwrap(), "#f(date: 0)\n#show: post.with(title: [date: x], date: 1)");
    }
}
