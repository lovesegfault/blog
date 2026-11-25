//! The built site, as files by address, and the cache of whatever is slow to make.

use std::borrow::Cow;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{Context, Result, bail, ensure};

pub enum File {
    Made(Vec<u8>),
    Copied(PathBuf),
}

pub type Files = BTreeMap<String, File>;

#[derive(Default)]
pub struct Output(Mutex<Files>);

pub struct Cache(pub PathBuf);

pub fn read(path: &Path) -> Result<Vec<u8>> {
    let bytes = fs::read(path).with_context(|| format!("cannot read {}", path.display()))?;
    ensure!(
        !bytes.starts_with(b"version https://git-lfs.github.com/spec/"),
        "{} is only a pointer: run `git lfs pull`",
        path.display()
    );
    Ok(bytes)
}

pub fn read_text(path: &Path) -> Result<String> {
    String::from_utf8(read(path)?).with_context(|| format!("{} is not UTF-8", path.display()))
}

/// Every file under `folder` but the hidden ones, in order.
pub fn walk(folder: &Path) -> Result<Vec<PathBuf>> {
    let mut found = Vec::new();
    for entry in fs::read_dir(folder).with_context(|| format!("cannot read {}", folder.display()))? {
        let path = entry?.path();
        if path.file_name().is_some_and(|name| name.as_encoded_bytes().starts_with(b".")) {
            continue;
        } else if path.is_dir() {
            found.extend(walk(&path)?);
        } else {
            found.push(path);
        }
    }
    found.sort();
    Ok(found)
}

fn digest(inputs: &[&[u8]]) -> String {
    let mut hasher = blake3::Hasher::new();
    for input in inputs {
        hasher.update(&input.len().to_le_bytes()).update(input);
    }
    hasher.finalize().to_hex()[..16].to_owned()
}

impl File {
    pub fn bytes(&self) -> Result<Cow<'_, [u8]>> {
        match self {
            Self::Made(bytes) => Ok(bytes.into()),
            Self::Copied(path) => read(path).map(Cow::from),
        }
    }
}

impl Output {
    /// An address that ends in `/` is a page.
    pub fn add(&self, url: &str, file: File) -> Result<()> {
        let path = if url.ends_with('/') { format!("{url}index.html") } else { url.to_owned() };
        if self.0.lock().unwrap().insert(path, file).is_some() {
            bail!("two things want to be at {url}");
        }
        Ok(())
    }

    /// Adds a file at an address that changes when it does, and returns the address.
    pub fn asset(&self, name: &str, file: File) -> Result<String> {
        let (stem, extension) = name.rsplit_once('.').context("an asset needs an extension")?;
        let url = format!("/assets/{stem}.{}.{extension}", &digest(&[&file.bytes()?])[..10]);
        self.0.lock().unwrap().insert(url.clone(), file);
        Ok(url)
    }

    pub fn into_files(self) -> Files {
        self.0.into_inner().unwrap()
    }
}

/// Replaces whatever an earlier build left in `folder`.
pub fn write(files: &Files, folder: &Path) -> Result<()> {
    if folder.exists() {
        let replaceable = folder.join("sitemap.xml").exists() || fs::read_dir(folder)?.next().is_none();
        ensure!(replaceable, "{} does not look like a built site, so it was left alone", folder.display());
        fs::remove_dir_all(folder)?;
    }
    for (url, file) in files {
        let path = folder.join(url.trim_start_matches('/'));
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(&path, file.bytes()?).with_context(|| format!("cannot write {}", path.display()))?;
    }
    Ok(())
}

impl Cache {
    /// The folder that `make` fills, which is made once for any given `inputs`.
    pub fn entry(&self, inputs: &[&[u8]], make: impl FnOnce(&Path) -> Result<()>) -> Result<PathBuf> {
        let folder = self.0.join(digest(inputs));
        if !folder.exists() {
            let draft = folder.with_extension(format!("{}-{:?}", std::process::id(), std::thread::current().id()));
            fs::create_dir_all(&draft)?;
            let made = make(&draft).and_then(|()| Ok(fs::rename(&draft, &folder)?));
            if made.is_err() {
                fs::remove_dir_all(&draft)?;
            }
            // Someone else may have got there first.
            if !folder.exists() {
                made?;
            }
        }
        Ok(folder)
    }
}
