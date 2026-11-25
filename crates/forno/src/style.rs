//! What every page loads: the fired tiles, the fonts, the stylesheet and the script.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use mural::{HOME, LABELS, NARROW, POST, RECIPE, Scene, Valley};

use crate::figure::FIRE;
use crate::output::{Cache, File, Output, read_text};
use crate::photo::webp;
use crate::site::Tools;

pub struct Style {
    pub sheet: String,
    pub math_sheet: String,
    pub script: String,
    /// For the text, the notes and the title.
    pub fonts: [String; 3],
    pub collider: String,
    /// The folder that each firing is in.
    pub fired: BTreeMap<String, PathBuf>,
}

fn fire(cache: &Cache) -> Result<BTreeMap<String, PathBuf>> {
    let drawing = read_text(Path::new("assets/valley.svg"))?;
    let valley = Valley::parse(&drawing).context("assets/valley.svg")?;
    let recipe = RECIPE.concat() + include_str!("photo.rs");
    let mut fired = BTreeMap::new();
    for firing in mural::firings(&valley) {
        let folder = cache.entry(&[recipe.as_bytes(), drawing.as_bytes(), firing.name.as_bytes()], |folder| {
            eprintln!("firing {}", firing.name);
            for (take, image) in firing.takes.iter().zip((firing.fire)()?) {
                let encoded = webp(&image.rgba, image.width, image.height, firing.quality)?;
                fs::write(folder.join(format!("{take}.webp")), encoded)?;
            }
            Ok(())
        })?;
        fired.insert(firing.name, folder);
    }
    Ok(fired)
}

/// A custom property for each firing, which changes at night, and a class for each glaze.
fn tiles(fired: &BTreeMap<String, PathBuf>, output: &Output) -> Result<String> {
    let (mut day, mut night) = (format!("--fire: {};", FIRE.concat().join(" ")), String::new());
    for (name, folder) in fired {
        let url =
            |take| output.asset(&format!("{name}-{take}.webp"), File::Copied(folder.join(format!("{take}.webp"))));
        write!(day, " --{name}: url({});", url("day")?)?;
        write!(night, " --{name}: url({});", url("night")?)?;
        if folder.join("water.webp").exists() {
            write!(day, " --{name}-water: url({});", url("water")?)?;
        }
    }
    let mut css = format!(
        ":root {{ {day} }}\n:root[data-theme=dark] {{{night} }}\n\
         @media (prefers-color-scheme: dark) {{ :root:not([data-theme=light]) {{{night} }} }}\n"
    );
    for (chip, (glaze, pale)) in LABELS.iter().enumerate() {
        let on = if *pale { "pale" } else { "dark" };
        writeln!(css, ".{glaze} {{ --chip: {chip}; --label: var(--label-{glaze}); --on-label: var(--on-{on}); }}")?;
    }
    Ok(css)
}

/// What `--scene: name;` stands for.
fn measurements(scene: &Scene) -> String {
    let Scene { name, size: [w, h], tip: [tx, ty], length, radius, .. } = scene;
    let ([vx, vy], turn, farm) = (scene.vertex(), scene.turn(), scene.farm());
    format!(
        "--mural: var(--{name}); --water: var(--{name}-water); --w: {w}; --h: {h}; --tx: {tx}; --ty: {ty}; \
         --vx: {vx}; --vy: {vy}; --turn: {turn}rad; --length: {length}; --farm: {farm}; --radius: {radius};"
    )
}

fn asset(output: &Output, path: PathBuf) -> Result<String> {
    output.asset(&path.file_name().unwrap().to_string_lossy(), File::Copied(path.clone()))
}

/// A stylesheet, pointed at where the files it uses are published.
fn sheet(path: &Path, output: &Output) -> Result<String> {
    let css = read_text(path)?;
    let mut parts = css.split("url(");
    let mut linked = parts.next().unwrap().to_owned();
    for part in parts {
        let (file, rest) = part.split_once(')').with_context(|| format!("{}: `url(` is not closed", path.display()))?;
        write!(linked, "url({}){rest}", asset(output, path.with_file_name(file))?)?;
    }
    Ok(linked)
}

impl Style {
    pub fn new(tools: &Tools, programs: &Path, output: &Output) -> Result<Self> {
        let fired = fire(&tools.cache)?;
        let mut own = sheet(Path::new("assets/site.css"), output)?;
        for scene in [HOME, POST, NARROW] {
            own = own.replace(&format!("--scene: {};", scene.name), &measurements(&scene));
        }
        let css = sheet(&tools.vendor.join("fonts.css"), output)? + &tiles(&fired, output)? + &own;
        let math = sheet(&tools.vendor.join("katex/katex.min.css"), output)?;
        let font = |name| asset(output, tools.vendor.join(format!("fonts/{name}-common.woff2")));
        Ok(Self {
            sheet: output.asset("site.css", File::Made(css.into_bytes()))?,
            math_sheet: output.asset("katex.css", File::Made(math.into_bytes()))?,
            script: asset(output, "assets/site.js".into())?,
            fonts: [font("Spectral-Regular")?, font("BonaNova-Italic")?, font("Spectral-Light")?],
            collider: asset(output, programs.join("collision.wasm"))?,
            fired,
        })
    }
}
