//! Puts the site together.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result};
use rayon::prelude::*;
use serde::Deserialize;

use crate::article::Article;
use crate::content::{self, Front, NOT_FOUND, address};
use crate::math::{Macros, Math};
use crate::output::{Cache, File, Files, Output, read, read_text, walk};
use crate::style::Style;
use crate::{links, pages, photo};

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
    pub macros: Macros,
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
    pub vendor: PathBuf,
    pub cache: Cache,
    pub math: Math,
}

pub struct Build<'a> {
    pub config: &'a Config,
    pub tools: &'a Tools,
    pub style: &'a Style,
    pub programs: &'a Path,
    pub output: Output,
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

impl Tools {
    pub fn new() -> Result<Self> {
        let vendor: PathBuf = env::var_os("FORNO_VENDOR").context("run this inside `nix develop`")?.into();
        let math = Math::new(read_text(&vendor.join("katex/katex.min.js"))?);
        Ok(Self { vendor, cache: Cache(".cache/forno".into()), math })
    }
}

impl<'a> Taxonomy<'a> {
    fn new([url, title, one]: [&'static str; 3], posts: &[&'a Article], of: fn(&Front) -> &[String]) -> Self {
        let mut terms = BTreeMap::new();
        for (post, name) in posts.iter().flat_map(|post| of(&post.page.front).iter().map(move |name| (*post, name))) {
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

    /// Publishes the files in `folder` as they are, but for what photos give away.
    fn copy(&self, folder: &Path, url: &str) -> Result<()> {
        for path in walk(folder)? {
            let name = path.strip_prefix(folder)?.to_string_lossy().into_owned();
            let file = if name == "index.md" {
                continue;
            } else if photo::is_photo(&path) {
                File::Made(photo::scrub(&read(&path)?).with_context(|| path.display().to_string())?)
            } else {
                if photo::gives_away(&path)? {
                    let how = "`exiftool -all=` or `ffmpeg -map_metadata -1`";
                    self.report(format!("{}: this may say where it was made. Clean it with {how}", path.display()));
                }
                File::Copied(path)
            };
            self.output.add(&(links::resolve(url, "") + &name), file)?;
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
    let build = Build { config: &config, tools, style: &style, programs, output, problems: Mutex::default() };
    let add = |url: &str, page: maud::Markup| build.output.add(url, File::Made(page.0.into_bytes()));

    let sources = content::load(drafts)?;
    let articles: Vec<Article> = sources.par_iter().map(|page| Article::new(&build, page)).collect();
    let posts: Vec<&Article> = articles.iter().take_while(|article| article.page.date.is_some()).collect();
    let card = pages::site_card(&build, &posts)?;

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
    let tags = Taxonomy::new(["/tags/", "Tags", "tag"], &posts, |front| &front.tags);
    let categories = Taxonomy::new(["/categories/", "Categories", "category"], &posts, |front| &front.categories);
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
        add(&from, pages::redirect(&build, &to))?;
    }

    let files = build.output.into_files();
    let mut problems = build.problems.into_inner().unwrap();
    problems.extend(links::check(&files));
    Ok(Site { files, problems })
}
