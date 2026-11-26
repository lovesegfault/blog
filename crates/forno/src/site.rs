//! Puts the site together.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result, ensure};
use jiff::Timestamp;
use rayon::prelude::*;
use serde::Deserialize;

use crate::article::Article;
use crate::content::{self, NOT_FOUND, Page, address};
use crate::output::{Cache, File, Files, Output, read_text, walk};
use crate::scrub::scrub;
use crate::style::Style;
use crate::typeset::{Desk, Typst};
use crate::{links, pages};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub url: String,
    pub title: String,
    pub subtitle: String,
    pub author: String,
    pub language: String,
    pub twitter: String,
    pub comments: String,
    pub farm: Farm,
    redirects: BTreeMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Farm {
    pub at: String,
    pub story: String,
}

/// What lasts from one build to the next.
pub struct Tools {
    pub dot: PathBuf,
    pub fonts: PathBuf,
    pub cache: Cache,
    typst: Typst,
    /// Each page, as it was when it last made sense.
    pages: Mutex<HashMap<PathBuf, Page>>,
}

pub struct Build<'a> {
    pub config: &'a Config,
    pub tools: &'a Tools,
    pub style: &'a Style,
    pub programs: &'a Path,
    pub output: Output,
    pub desk: Desk<'a>,
    problems: Mutex<BTreeSet<String>>,
}

pub struct Site {
    pub files: Files,
    /// Mistakes in the writing. The site can be looked at, but not published.
    pub problems: BTreeSet<String>,
}

pub struct Taxonomy<'a> {
    pub url: &'static str,
    pub title: &'static str,
    pub one: &'static str,
    pub terms: BTreeMap<String, Term<'a>>,
}

pub struct Term<'a> {
    pub name: &'a str,
    pub url: String,
    pub posts: Vec<&'a Article<'a>>,
}

pub const SOURCES: [&str; 5] = ["assets", "blog.typ", "content", "site.toml", "static"];

impl Tools {
    pub fn new() -> Result<Self> {
        let path = |name| env::var_os(name).map(PathBuf::from).context("run this inside `nix develop`");
        let (dot, fonts) = (path("GRAPHVIZ_DOT")?, path("TYPST_FONT_PATHS")?);
        let typst = Typst::new(&fonts, &path("TYPST_PACKAGE_CACHE_PATH")?)?;
        Ok(Self { dot, fonts, typst, cache: Cache(".cache/forno".into()), pages: Mutex::default() })
    }
}

impl<'a> Taxonomy<'a> {
    fn new([url, title, one]: [&'static str; 3], posts: &[&'a Article], of: fn(&Page) -> &[String]) -> Self {
        let mut terms = BTreeMap::new();
        for (post, name) in posts.iter().flat_map(|post| of(post.page).iter().map(move |name| (*post, name))) {
            let new = |slug: &String| Term { name, url: format!("{url}{slug}/"), posts: Vec::new() };
            terms.entry(address(name)).or_insert_with_key(new).posts.push(post);
        }
        Self { url, title, one, terms }
    }
}

impl Build<'_> {
    pub fn report(&self, problem: String) {
        self.problems.lock().unwrap().insert(problem);
    }

    fn page(&self, path: &Path) -> Option<Page> {
        let page = self.desk.set(path).and_then(|(document, problems)| {
            self.problems.lock().unwrap().extend(problems);
            document.map(|document| Page::new(path, &document)).transpose()
        });
        let page = page.inspect_err(|error| self.report(format!("{}: {error:#}", path.display())));
        let mut pages = self.tools.pages.lock().unwrap();
        pages.extend(page.ok().flatten().map(|page| (path.into(), page)));
        pages.get(path).cloned()
    }

    /// Publishes the files in `folder` as they are, but for what photos give away.
    fn copy(&self, folder: &Path, url: &str) -> Result<()> {
        for path in walk(folder)?.into_iter().filter(|path| !content::is_source(path)) {
            let url = links::resolve(url, "") + &path.strip_prefix(folder)?.to_string_lossy();
            let file = scrub(&path).inspect_err(|error| self.report(format!("{error:#}")));
            self.output.add(&url, file.unwrap_or(File::Copied(path)))?;
        }
        Ok(())
    }
}

/// `*` stands for the name of any page there is.
fn redirects(from: &str, to: &str, pages: &[&str]) -> Vec<(String, String)> {
    let Some((before, after)) = to.split_once('*') else { return vec![(from.to_owned(), to.to_owned())] };
    let names = pages.iter().filter_map(|url| url.strip_prefix(before)?.strip_suffix(after));
    names.map(|name| (from.replace('*', name), to.replace('*', name))).collect()
}

pub fn build(tools: &Tools, programs: &Path, drafts: bool) -> Result<Site> {
    let config: Config = toml::from_str(&read_text(Path::new("site.toml"))?).context("site.toml")?;
    let output = Output::default();
    let style = Style::new(tools, programs, &output)?;
    let (desk, problems) = (tools.typst.desk(), Mutex::default());
    let build = Build { config: &config, tools, style: &style, programs, output, desk, problems };
    let add = |url: &str, page: maud::Markup| build.output.add(url, File::Made(page.0.into_bytes()));

    // Posts, newest first, and then the other pages.
    let paths = content::sources(drafts);
    let mut sources: Vec<Page> = paths.par_iter().filter_map(|path| build.page(path)).collect();
    sources.sort_by(|a, b| (&b.date, &a.slug).cmp(&(&a.date, &b.slug)));
    let articles: Vec<Article> = sources.par_iter().map(|page| Article::new(&build, page)).collect();
    let posts: Vec<&Article> = articles.iter().take_while(|article| article.page.date.is_some()).collect();
    let card = pages::site_card(&build, &posts)?;
    for Article { page, body, .. } in articles.iter().filter(|article| !article.page.draft) {
        if page.date.iter().chain(&page.updated).any(|date| date.timestamp() > Timestamp::now()) {
            build.report(format!("{}: this is dated in the future", page.path.display()));
        }
        let linked = |draft: &&Page| draft.draft && body.0.contains(&format!("=\"{}", draft.url));
        for draft in sources.iter().filter(linked) {
            build.report(format!("{}: `{}` is only a draft", page.path.display(), draft.url));
        }
    }

    build.copy(Path::new("static"), "/")?;
    add("/", pages::home(&build, &posts, &card)?)?;
    add("/index.xml", pages::feed(&build, None, "/", &posts))?;
    add("/posts/index.xml", pages::feed(&build, Some("Posts"), "/posts/", &posts))?;
    articles.par_iter().enumerate().try_for_each(|(at, article)| {
        let beside = |other: usize| posts.get(at).and(posts.get(other)).copied();
        add(&article.page.url, pages::article(&build, article, beside(at.wrapping_sub(1)), beside(at + 1))?)?;
        article.page.folder.as_ref().map_or(Ok(()), |folder| build.copy(folder, &article.page.url))
    })?;

    let mut listed = vec![("/", None)];
    let found = sources.iter().filter(|page| page.url != NOT_FOUND);
    listed.extend(found.map(|page| (page.url.as_str(), page.updated.as_ref().or(page.date.as_ref()))));
    let tags = Taxonomy::new(["/tags/", "Tags", "tag"], &posts, |page| &page.tags);
    let categories = Taxonomy::new(["/categories/", "Categories", "category"], &posts, |page| &page.categories);
    for (taxonomy, other) in [(&tags, &categories), (&categories, &tags)] {
        add(taxonomy.url, pages::terms(&build, taxonomy, other, &posts, &card)?)?;
        listed.push((taxonomy.url, None));
        for term in taxonomy.terms.values() {
            add(&term.url, pages::term(&build, taxonomy, term, &posts, &card)?)?;
            add(&format!("{}index.xml", term.url), pages::feed(&build, Some(term.name), &term.url, &term.posts))?;
            listed.push((&term.url, None));
        }
    }
    add("/sitemap.xml", pages::sitemap(&build, &listed))?;

    let urls: Vec<&str> = listed.iter().map(|(url, _)| *url).collect();
    for (from, to) in config.redirects.iter().flat_map(|(from, to)| redirects(from, to, &urls)) {
        ensure!(from.ends_with('/') || from.ends_with(".html"), "site.toml: `{from}` has to end in `/` or `.html`");
        add(&from, pages::redirect(&build, &to))?;
    }

    let files = build.output.into_files();
    let mut problems = build.problems.into_inner().unwrap();
    if sources.len() == paths.len() {
        problems.extend(links::check(&files, &config.url));
    }
    Ok(Site { files, problems })
}
