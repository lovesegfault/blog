//! Runs Typst.

use std::path::Path;

use anyhow::Result;
use typst::diag::{FileError, FileResult, PackageError, SourceDiagnostic, Tracepoint, bail, error};
use typst::foundations::{Bytes, Content, Datetime, Duration, NativeElement, ShowFn, Smart, Target};
use typst::layout::{BlockElem, Rel, Sizing};
use typst::loading::LoadSource;
use typst::model::FootnoteContainer;
use typst::syntax::{DiagSpan, FileId, LinkedNode, RootedPath, Source, SyntaxKind, VirtualPath, VirtualRoot, ast};
use typst::text::{Font, FontBook, RawElem, TextElem};
use typst::utils::LazyHash;
use typst::visualize::ImageElem;
use typst::{Feature, Library, LibraryExt, World, WorldExt};
use typst_html::{HtmlDocument, HtmlElem, HtmlTag, attr};
use typst_kit::files::{FileLoader, FileStore};
use typst_kit::fonts::{FontStore, embedded};
use typst_kit::packages::FsPackages;

use crate::content::is_source;
use crate::output::{read, walk};
use crate::scrub::scrub;
use crate::site::SOURCES;
use crate::{formula, photo};

pub struct Typst {
    library: LazyHash<Library>,
    fonts: FontStore,
    packages: FsPackages,
}

/// The files as they are now.
pub struct Desk<'a>(FileStore<&'a Typst>);

struct Job<'a> {
    desk: &'a Desk<'a>,
    main: FileId,
}

/// Pictures that browsers can show are left where they are.
const PICTURE: ShowFn<ImageElem> = |image, engine, styles| {
    let (source, kind, [across, down]) = if let LoadSource::Path(file) = image.source.derived.source.v
        && *file.root() == VirtualRoot::Project
        && let Some((kind, size)) = photo::measure(file.vpath().extension(), &image.source.derived.data)
    {
        (file.vpath().get_without_slash().into(), kind, size)
    } else if image.source.derived.data.starts_with(b"\x1F\x8B") {
        bail!(image.span(), "this has to be unpacked with `gunzip`");
    } else {
        let image = image.decode(engine, styles)?;
        (typst_svg::WebImage::new(&image).to_base64_url(), "embedded", [image.width(), image.height()])
    };
    let (em, fit) = (styles.resolve(TextElem::size), ["cover", "contain", "fill"][image.fit.get(styles) as usize]);
    let width = |it: Rel<_>| format!("width:calc({}% + {:.3}em);", it.rel.get() * 100.0, it.abs / em);
    let [width, crop] = match (image.width.resolve(styles).custom().map(width), image.height.get(styles)) {
        (width, Sizing::Auto) => [width.unwrap_or_default(), String::new()],
        (None, Sizing::Rel(it)) if it.rel.is_zero() => {
            [format!("width:{:.3}em;", it.abs.at(em) / em * across / down), String::new()]
        }
        (Some(width), Sizing::Rel(it)) if it.rel.is_zero() => {
            [width + "max-width:100%;", format!("height:{:.3}em;object-fit:{fit};", it.abs.at(em) / em)]
        }
        _ => bail!(image.span(), "a page has no height to take a part of"),
    };
    let pixels = |count: f64| matches!(kind, "jpeg" | "png" | "pixels").then(|| count.to_string());
    let picture = HtmlElem::new(HtmlTag::intern("x-picture").unwrap())
        .with_attr(attr::src, source)
        .with_attr(attr::kind, kind)
        .with_attr(attr::size, format!("--ratio:{:.3};{width}", across / down))
        .with_attr(attr::shape, crop)
        .with_optional_attr(attr::alt, image.alt.get_cloned(styles))
        .with_optional_attr(attr::width, pixels(across))
        .with_optional_attr(attr::height, pixels(down));
    Ok(BlockElem::packed(picture.pack()))
};

impl Typst {
    pub fn new(faces: &Path, packages: &Path) -> Result<Self> {
        let mut library = Library::builder().with_features([Feature::Html].into_iter().collect()).build();
        library.rules.replace(Target::Html, formula::RULE);
        library.rules.replace(Target::Html, PICTURE);
        library.rules.replace::<FootnoteContainer>(Target::Html, |_, _, _| Ok(Content::empty()));
        library.styles.set(RawElem::theme, Smart::Custom(None));
        let mut fonts = FontStore::new();
        for path in walk(faces)? {
            fonts.extend(Font::iter(Bytes::new(read(&path)?)).map(|font| (font.clone(), font.info().clone())));
        }
        fonts.extend(embedded());
        Ok(Self { library: library.into(), fonts, packages: FsPackages::new(packages) })
    }

    pub fn desk(&self) -> Desk<'_> {
        typst::comemo::evict(4);
        Desk(FileStore::new(self))
    }
}

impl FileLoader for &Typst {
    fn load(&self, file: FileId) -> FileResult<Bytes> {
        if let VirtualRoot::Package(package) = file.root() {
            let missing = || PackageError::Other(Some("add it to flake.nix".into()));
            return self.packages.obtain(package).ok_or_else(missing)?.load(file.vpath());
        }
        // Drawings have the pictures in them whole.
        let path = Path::new(file.vpath().get_without_slash());
        let bytes = if is_source(path) { read(path) } else { scrub(path).and_then(|file| Ok(file.bytes()?.into())) };
        bytes.map(Bytes::new).map_err(|error| FileError::Other(Some(format!("{error:#}").into())))
    }
}

impl Job<'_> {
    /// Only what `flake.nix` builds the site from.
    fn allowed(&self, file: FileId) -> FileResult<FileId> {
        let under = |file: FileId, root| Path::new(file.vpath().get_without_slash()).starts_with(root);
        let draft = |file| under(file, "content/drafts");
        let known = SOURCES.iter().any(|root| under(file, root)) && draft(file) <= draft(self.main);
        let allowed = known || file == self.main || *file.root() != VirtualRoot::Project;
        allowed.then_some(file).ok_or(FileError::Other(Some("the site is published without it".into())))
    }
}

impl World for Job<'_> {
    fn library(&self) -> &LazyHash<Library> {
        &self.desk.0.loader().library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.desk.0.loader().fonts.book()
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, file: FileId) -> FileResult<Source> {
        self.desk.0.source(self.allowed(file)?)
    }

    fn file(&self, file: FileId) -> FileResult<Bytes> {
        self.desk.0.file(self.allowed(file)?)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.desk.0.loader().fonts.font(index)
    }

    fn today(&self, _: Option<Duration>) -> Option<Datetime> {
        None
    }
}

/// Whether a file is part of the writing.
pub fn is_written(file: Option<FileId>) -> bool {
    file.is_some_and(|file| *file.root() == VirtualRoot::Project && file.vpath().get_with_slash() != "/blog.typ")
}

/// Text that Typst would quietly take for something else.
fn misread(node: &LinkedNode, source: &str, found: &mut Vec<SourceDiagnostic>) {
    use SyntaxKind::*;
    node.children().for_each(|child| misread(&child, source, found));
    let (before, written, after) = (&source[..node.offset()], &source[node.range()], &source[node.range().end..]);
    let (previous, word) = (node.prev_sibling_kind().unwrap_or(End), char::is_alphanumeric);
    let (inside, last) = (written.trim_matches(['*', '_']), node.prev_sibling());
    let linked = || source.split(written).any(|before| before.trim_end().ends_with("link("));
    let block = |it: LinkedNode| it.cast().is_some_and(ast::Raw::block) || it.cast().is_some_and(ast::Equation::block);
    let prose = matches!(previous, Text | Strong | Emph | SmartQuote | Shorthand | Escape | Link | Raw | Equation);
    let wrapped = before[before.trim_end().len()..].matches('\n').count() == 1 && !matches!(previous, EnumItem | End);
    let opens = previous == End || before.ends_with([' ', '\n', '(']) || before.ends_with(" \"");
    let (it, sign) = match node.kind() {
        _ if node.parent_kind() != Some(Markup) => return,
        Equation if before.ends_with(char::is_alphabetic) || after.starts_with(char::is_numeric) => ("a formula", '$'),
        Label if prose && !last.is_some_and(block) && !linked() => ("a label", '<'),
        Strong if inside.trim() != inside => ("strong emphasis", '*'),
        Emph if inside.trim() != inside => ("emphasis", '_'),
        Link if after.starts_with(|c| word(c) || c == '–') => {
            return found.push(error!(node.span(), "this link is cut short"; hint: "write it as `#link(\"…\")`"));
        }
        SetRule if written.contains("lang:") && node.parent().and_then(LinkedNode::parent).is_none() => {
            return found.push(error!(node.span(), "the language of a page is `lang`, in `post.with`"));
        }
        LineComment | BlockComment if before.ends_with(|c: char| !c.is_whitespace()) => ("a comment", '/'),
        Semicolon if !previous.is_stmt() && !after.starts_with(|c| word(c) || "([.-_".contains(c)) => ("code", ';'),
        Hash if after.starts_with(char::is_numeric) => ("code", '#'),
        Escape if written.ends_with(word) => ("an escape", '\\'),
        EnumItem if written.starts_with(word) && !written.starts_with("1.") && wrapped => ("a list", '.'),
        Shorthand if written == "~" && (opens || after.starts_with('/')) => ("a space", '~'),
        _ => return,
    };
    found.push(error!(node.span(), "this reads as {it}"; hint: "the sign alone is `\\{sign}`"));
}

impl Desk<'_> {
    /// The page at `path`, unless it is too broken, and what is wrong with it.
    pub fn set(&self, path: &Path) -> Result<(Option<HtmlDocument>, Vec<String>)> {
        let id = |path| {
            anyhow::Ok(RootedPath::new(VirtualRoot::Project, VirtualPath::virtualize(Path::new(""), path)?).intern())
        };
        let set = typst::compile::<HtmlDocument>(&Job { desk: self, main: id(path)? });
        let mut found: Vec<_> = set.output.as_ref().err().into_iter().flatten().cloned().collect();
        let ours = |file: FileId| *file.root() == VirtualRoot::Project;
        found.extend(
            set.warnings.into_iter().filter(|it| it.span.id().is_some_and(ours) || it.message.contains("ignored")),
        );
        let parts = if path.ends_with("index.typ") { walk(path.parent().unwrap())? } else { vec![path.into()] };
        for source in parts.iter().filter(|part| is_source(part)).flat_map(|part| self.0.source(id(part).ok()?).ok()) {
            misread(&LinkedNode::new(source.root()), source.text(), &mut found);
        }
        let describe = |problem: SourceDiagnostic| {
            let mut trace = [problem.span].into_iter().chain(problem.trace.iter().map(|point| point.span.into()));
            let place = match problem.trace.first().map(|point| &point.v) {
                Some(Tracepoint::Import(name)) if name == "/blog.typ" => {
                    self.place(problem.span, "every page".as_ref())
                }
                _ => self.place(trace.find(|span| is_written(span.id())).unwrap_or(problem.span), path),
            };
            problem.hints.iter().fold(format!("{place}: {}", problem.message), |text, hint| text + "\n  " + &hint.v)
        };
        Ok((set.output.ok(), found.into_iter().map(describe).collect()))
    }

    /// Names where something is written, or else the page that it is on.
    pub fn place(&self, span: impl Into<DiagSpan>, page: &Path) -> String {
        let find = |span: DiagSpan| {
            let job = Job { desk: self, main: span.id()? };
            let line = job.file(job.main).ok()?.lines().ok()?.byte_to_line(job.range(span)?.start)?;
            let place = format!("{}:{}", job.main.vpath().get_without_slash(), line + 1);
            Some(if is_written(span.id()) { place } else { format!("{}, by way of {place}", page.display()) })
        };
        find(span.into()).unwrap_or_else(|| page.display().to_string())
    }
}

#[cfg(test)]
pub mod tests {
    use std::{env, fs};

    use super::*;
    use crate::tree::{Node, Tree};

    pub fn set(source: &str) -> (Vec<Node>, Vec<String>) {
        let (folder, name) = (env::temp_dir().join("forno-tests"), format!("{}.typ", blake3::hash(source.as_bytes())));
        fs::create_dir_all(folder.join("fonts"))
            .and(env::set_current_dir(&folder))
            .and(fs::write(&name, source))
            .unwrap();
        let (document, problems) = Typst::new("fonts".as_ref(), &folder).unwrap().desk().set(name.as_ref()).unwrap();
        (document.map_or(Vec::new(), |document| Tree::new(&document).nodes), problems)
    }

    #[test]
    fn problems_have_a_line() {
        assert!(set("a\n\n#b").1[0].contains(".typ:3: unknown variable: b"));
        assert!(set("a\n#v(1em)").1[0].contains(".typ:2: v was ignored"));
    }

    #[test]
    fn prose_is_not_lost_to_markup() {
        let lost = r#"$5 or $6|R$x$|Vec<T>|a <b>|#1|at ~5|"~/a"|*#emph[a];*|ftp://a|a/*b*/|C:\a|`a` <b>|"<a>""#;
        for lost in lost.split('|').chain(["$a$ <b>", "_a _b", "2 * 3 * 4", "http://a/São", "#set text(lang: \"pt\")"])
        {
            assert_eq!(set(lost).1.len(), 1, "{lost}");
        }
        assert_eq!(set("in\n2012. a\n\nb:\n1. c\n2. d\n\ne\n\n5. f").1.len(), 1);
        let fine = r"$n$th|= A <a>|a <b> #link(<b>)[c]|a~b|#let a = 1; b|#emph[a];(b)|http://a|a // b|\u{41}";
        for fine in fine.split('|').chain(["$ a $ <b>", "```\na\n``` <b>", "*> _b_*", "#[#set text(lang: \"pt\")]"]) {
            assert_eq!(set(fine).1, [""; 0]);
        }
    }
}
