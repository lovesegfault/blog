//! What is under `content/`.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail, ensure};
use jiff::tz::{Offset, TimeZone};
use jiff::{Zoned, civil};
use serde::Deserialize;
use toml::value::Datetime;

use crate::output::read_text;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Front {
    pub title: String,
    date: Option<Datetime>,
    updated: Option<Datetime>,
    #[serde(default)]
    pub draft: bool,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub comments: Comments,
    #[serde(default)]
    pub macros: BTreeMap<String, String>,
}

#[derive(Deserialize, Clone, Copy)]
#[serde(untagged)]
pub enum Comments {
    /// In the issue named after the page.
    Open(bool),
    /// In this issue, whatever the page is called now.
    Issue(u32),
}

pub struct Page {
    pub front: Front,
    pub slug: String,
    pub url: String,
    /// Only posts have one.
    pub date: Option<Zoned>,
    pub updated: Option<Zoned>,
    pub path: PathBuf,
    /// Where its pictures and files are, if it has a folder to itself.
    pub folder: Option<PathBuf>,
    source: String,
    body: usize,
}

pub const NOT_FOUND: &str = "/404.html";

impl Default for Comments {
    fn default() -> Self {
        Self::Open(true)
    }
}

/// The front matter, and where the body starts.
fn split(source: &str) -> Result<(&str, usize)> {
    let front = source.strip_prefix("+++").and_then(|rest| rest.split_once("\n+++\n"));
    let (front, body) = front.context("the file has to start with front matter between lines of `+++`")?;
    Ok((front, source.len() - body.len()))
}

fn zoned(when: Datetime) -> Result<Zoned> {
    let day = when.date.context("a date needs a day")?;
    let day = civil::Date::new(day.year as i16, day.month as i8, day.day as i8)?;
    let time = match when.time {
        Some(time) => civil::Time::new(time.hour as i8, time.minute as i8, time.second.unwrap_or(0) as i8, 0)?,
        None => civil::Time::midnight(),
    };
    let minutes = match when.offset {
        Some(toml::value::Offset::Custom { minutes }) => minutes,
        _ => 0,
    };
    Ok(day.to_datetime(time).to_zoned(TimeZone::fixed(Offset::from_seconds(i32::from(minutes) * 60)?))?)
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

impl Page {
    pub fn parse(path: &Path, source: String, is_post: bool) -> Result<Self> {
        let folder = path.ends_with("index.md").then(|| path.parent().unwrap().to_owned());
        let name = folder.as_deref().map_or(path.file_stem(), Path::file_name);
        let slug = name.context("a page needs a name")?.to_string_lossy().into_owned();
        ensure!(slug == slugify(&slug), "`{slug}` would not do in an address: try `{}`", slugify(&slug));
        let (front, body) = split(&source)?;
        let front: Front = toml::from_str(front)?;
        ensure!(front.date.is_some() == is_post, "posts have a `date`, and other pages do not");
        Ok(Self {
            url: match slug.as_str() {
                _ if is_post => format!("/posts/{slug}/"),
                "404" => NOT_FOUND.to_owned(),
                _ => format!("/{slug}/"),
            },
            date: front.date.map(zoned).transpose()?,
            updated: front.updated.map(zoned).transpose()?,
            path: path.to_owned(),
            front,
            slug,
            folder,
            source,
            body,
        })
    }

    pub fn body(&self) -> &str {
        &self.source[self.body..]
    }

    /// Names a place in the body, for errors.
    pub fn at(&self, offset: usize) -> String {
        format!("{}:{}", self.path.display(), self.source[..self.body + offset].matches('\n').count() + 1)
    }

    /// One of its own files.
    pub fn file(&self, name: &str) -> Result<PathBuf> {
        let folder = self.folder.as_ref().with_context(|| format!("move this page to `{}/index.md`", self.slug));
        Ok(folder.with_context(|| format!("`{name}` has to be in the page's own folder"))?.join(name))
    }
}

/// Each `name.md` and `name/index.md` in `folder`.
fn sources(folder: &Path) -> Vec<PathBuf> {
    let entries = fs::read_dir(folder).into_iter().flatten().flatten().map(|entry| entry.path());
    let source = |path: PathBuf| if path.is_dir() { path.join("index.md") } else { path };
    entries.map(source).filter(|path| path.extension().is_some_and(|e| e == "md") && path.is_file()).collect()
}

/// Posts, newest first, and then the other pages.
pub fn load(drafts: bool) -> Result<Vec<Page>> {
    let mut pages = Vec::new();
    for (folder, is_post) in [("content", false), ("content/posts", true), ("content/drafts", true)] {
        let unfinished = folder == "content/drafts";
        if unfinished && !drafts {
            continue;
        }
        for path in sources(Path::new(folder)) {
            let mut page =
                Page::parse(&path, read_text(&path)?, is_post).with_context(|| path.display().to_string())?;
            page.front.draft |= unfinished;
            if drafts || !page.front.draft {
                pages.push(page);
            }
        }
    }
    pages.sort_by(|a, b| (&b.date, &a.slug).cmp(&(&a.date, &b.slug)));
    Ok(pages)
}

fn now() -> String {
    format!("date = {}", Zoned::now().strftime("%Y-%m-%dT%H:%M:%S%:z"))
}

fn named(folder: &str, slug: &str) -> Option<PathBuf> {
    [slug.to_owned(), format!("{slug}.md")].iter().map(|name| Path::new(folder).join(name)).find(|path| path.exists())
}

/// Starts a draft.
pub fn draft(title: &str) -> Result<PathBuf> {
    let slug = address(title);
    ensure!(!slug.is_empty(), "there is nothing in that title to make an address from");
    if let Some(taken) = named("content/drafts", &slug).or(named("content/posts", &slug)) {
        bail!("{} already exists", taken.display());
    }
    let path = Path::new("content/drafts").join(slug).join("index.md");
    let title = toml::Value::from(title);
    fs::create_dir_all(path.parent().unwrap())?;
    fs::write(&path, format!("+++\ntitle = {title}\n{}\ncategories = [\"essay\"]\ntags = []\n+++\n", now()))?;
    Ok(path)
}

/// Moves a draft in with the posts, dated now.
pub fn publish(name: &str) -> Result<PathBuf> {
    let name = Path::new(name.trim_end_matches("/index.md"));
    let slug = name.file_stem().context("which draft?")?.to_string_lossy();
    let from = named("content/drafts", &slug).with_context(|| format!("there is no draft called `{slug}`"))?;
    if let Some(taken) = named("content/posts", &slug) {
        bail!("{} already exists", taken.display());
    }
    let markdown = |path: &Path| if from.is_dir() { path.join("index.md") } else { path.to_owned() };
    let to = Path::new("content/posts").join(from.file_name().unwrap());
    let published = markdown(&to);
    fs::write(markdown(&from), stamp(&read_text(&markdown(&from))?, &now())?)?;
    fs::rename(&from, &to)?;
    Ok(published)
}

fn stamp(source: &str, date: &str) -> Result<String> {
    let (front, body) = split(source)?;
    let sets = |line: &str, key: &str| line.split('=').next().is_some_and(|name| name.trim() == key);
    let lines = front.lines().filter(|line| !sets(line, "draft")).map(|l| if sets(l, "date") { date } else { l });
    Ok(format!("+++{}\n+++\n{}", lines.collect::<Vec<_>>().join("\n"), &source[body..]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchors_match_the_ones_hugo_made() {
        assert_eq!(slugify("1. sizeof()"), "1-sizeof");
        assert_eq!(slugify("Standard Cognition & Rust in 2020"), "standard-cognition--rust-in-2020");
        assert_eq!(slugify(r"An orthogonal basis \mathcal B"), "an-orthogonal-basis-mathcal-b");
        assert_eq!(address("Então: C, C++ & C#"), "entao-c-cpp-csharp");
    }

    #[test]
    fn dates_keep_their_own_offset() {
        let source = "+++\ntitle = 'x'\ndate = 2021-04-01T22:04:19-07:00\n+++\n\nbody\n";
        let page = Page::parse(Path::new("x/index.md"), source.into(), true).unwrap();
        assert_eq!(page.date.as_ref().unwrap().strftime("%F %:z").to_string(), "2021-04-01 -07:00");
        assert_eq!((page.url.as_str(), page.at(1).as_str()), ("/posts/x/", "x/index.md:6"));
    }

    #[test]
    fn publishing_only_touches_the_date_and_the_flag() {
        let draft = "+++\ntitle = 'x'\ndate = 2020-01-01\ndraft = true\ntags = []\n+++\n\ndate = draft\n";
        assert_eq!(stamp(draft, "date = 1").unwrap(), "+++\ntitle = 'x'\ndate = 1\ntags = []\n+++\n\ndate = draft\n");
    }
}
