//! What every page loads: the fired tiles, the fonts, the stylesheet and the script.

use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use mural::{HOME, LABELS, NARROW, POST, RECIPE, Scene, Valley};
use typst::visualize::{Color, Oklab};

use crate::figure::FIRE;
use crate::output::{Cache, File, Output, read_text};
use crate::photo::webp;
use crate::site::Tools;

pub struct Style {
    pub sheet: String,
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
    let fire = FIRE.concat().join(" ");
    let mut day = format!("--scheme: only light; --day: initial; --night: ; --styled: 100%; --fire: {fire};");
    let mut night = String::from(" --scheme: dark; --day: ; --night: initial; --dark: 100%;");
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
         @media (prefers-color-scheme: dark) {{ :root:not([data-theme=light]) {{{night} }} }}\n\
         @container style(--sky: black) {{ body {{{night} }} }}\n"
    );
    for (chip, (glaze, pale)) in LABELS.iter().enumerate() {
        let [on, under] = if *pale { ["pale", "dark"] } else { ["dark", "pale"] }.map(|it| format!("var(--on-{it})"));
        writeln!(css, ".{glaze} {{ --chip: {chip}; --label: var(--label-{glaze}); --on: {on}; --under: {under}; }}")?;
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

/// A colour meant for paper, as one that follows the theme and stands out as much from the others.
fn paint(day: &str, thin: bool) -> Option<String> {
    let day = if let Some(mixed) = day.strip_prefix("oklab(") {
        let mut parts = mixed.split([' ', '%', ')']).flat_map(str::parse);
        Oklab::new(parts.next()? / 100.0, parts.next()?, parts.next()?, parts.next().unwrap_or(1.0)).into()
    } else {
        day.parse().map(|it: svgtypes::Color| Color::from_u8(it.red, it.green, it.blue, it.alpha)).ok()?
    };
    let mut night = day.to_oklch();
    // Nothing is written over a red line, which is better left bright.
    let least = if thin && night.chroma > 0.05 && night.l < 0.8 { night.l } else { 0.0 };
    night.l = (0.06 / (night.l.powi(3) + 0.05) - 0.05).cbrt().clamp(least, 0.92);
    while Color::from(Color::from(night).to_rgb()).to_oklch().chroma < night.chroma - 0.005 {
        night.chroma -= 0.005;
    }
    // Reader views have none of these properties, and older browsers no `light-dark()`.
    let night = Color::from(night).to_hex();
    let or = |it: &str, when, other: &str| format!("color-mix(in srgb, {it} var(--{when}, 0%), {other})");
    Some(match &*day.to_hex() {
        "#000000" => "var(--ink, currentColor)".into(),
        "#ffffff" => "var(--paper, Canvas)".into(),
        "#a83a10" => "var(--ember, chocolate)".into(),
        "#17389c" => "var(--cobalt, royalblue)".into(),
        "#6b5c50" => "var(--dim, gray)".into(),
        "#d3d3d3" if !thin => "var(--inline, #8884)".into(),
        day => or(&night, "dark", &or(day, "styled", &format!("var(--ink, light-dark({day}, {night}))"))),
    })
}

/// Whatever uses `light-dark()` again, for browsers from before it.
fn lowered(css: &str) -> String {
    let (mut rules, mut selector) = (String::new(), "");
    for part in css.split_inclusive(['{', ';', '}']) {
        if let Some((before, rest)) = part.split_once("light-dark(")
            && let [day, night, after] = rest.splitn(3, [',', ')']).collect::<Vec<_>>()[..]
        {
            let after = after.trim_end_matches('}');
            write!(rules, "{selector}{before}var(--day, {day}) var(--night, {night}){after}}}").unwrap();
        } else if part.ends_with('{') {
            selector = part;
        }
    }
    format!("@supports not (color: light-dark(red, red)) {{{rules}}}")
}

pub fn themed(svg: &str) -> String {
    let mut themed = String::new();
    for part in svg.split_inclusive(['"', ':', ';']) {
        let (value, end) = part.split_at(part.len() - 1);
        let tag = themed.rsplit('<').next().unwrap();
        let thin = tag.ends_with("stroke=\"") || tag.starts_with("use") || tag.starts_with("text");
        let painted = ["fill=\"", "stroke=\"", "-color=\"", "stop-color:"].iter().any(|name| tag.ends_with(name));
        themed.extend([painted.then(|| paint(value, thin)).flatten().as_deref().unwrap_or(value), end]);
    }
    themed
}

fn asset(output: &Output, path: PathBuf) -> Result<String> {
    output.asset(&path.file_name().unwrap().to_string_lossy(), File::Copied(path.clone()))
}

/// A stylesheet, pointed at where the files it uses from `folder` are published.
fn sheet(css: String, folder: &Path, output: &Output) -> Result<String> {
    let mut parts = css.split("url(");
    let mut linked = parts.next().unwrap().to_owned();
    for part in parts {
        let (file, rest) = part.split_once(')').context("a `url(` in the stylesheet is not closed")?;
        write!(linked, "url({}){rest}", asset(output, folder.join(file.trim_matches('"')))?)?;
    }
    Ok(linked)
}

impl Style {
    pub fn new(tools: &Tools, programs: &Path, output: &Output) -> Result<Self> {
        let fired = fire(&tools.cache)?;
        let mut own = sheet(read_text(Path::new("assets/site.css"))?, Path::new("assets"), output)?;
        for scene in [HOME, POST, NARROW] {
            own = own.replace(&format!("--scene: {};", scene.name), &measurements(&scene));
        }
        let fonts = sheet(read_text(&tools.fonts.join("fonts.css"))?, &tools.fonts, output)?;
        let css = fonts + &tiles(&fired, output)? + &own + &lowered(&own);
        let font = |name| asset(output, tools.fonts.join(format!("{name}-common.woff2")));
        Ok(Self {
            sheet: output.asset("site.css", File::Made(css.into_bytes()))?,
            script: asset(output, "assets/site.js".into())?,
            fonts: [font("Spectral-Regular")?, font("BonaNova-Italic")?, font("Spectral-Light")?],
            collider: asset(output, programs.join("collision.wasm"))?,
            fired,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colours_follow_the_theme() {
        assert_eq!(themed(r##"<use fill="#fff">fill:red;"##), r#"<use fill="var(--paper, Canvas)">fill:red;"#);
        assert!(themed(r#"<g stroke="none" fill="pink">"#).contains(r#""none" fill="color-mix(in srgb, #5"#));
        assert!(paint("red", false) < paint("red", true));
        assert!(lowered("a { b: c(light-dark(d,e) f) }").ends_with("{a { b: c(var(--day, d) var(--night, e) f) }}"));
    }
}
