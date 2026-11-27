//! The collision in the lake: on a page, and on the card that goes with a link to it.

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use collision::{Camera, Event, START, Source};
use image::codecs::jpeg::JpegEncoder;
use image::imageops::{FilterType, resize};
use image::{DynamicImage, RgbaImage};
use maud::{Markup, html};
use mural::{HOME, Scene};
use resvg::tiny_skia::{IntSize, Mask, MaskType, Pixmap, PixmapPaint, Transform};
use resvg::usvg::{Options, Tree};
use ttf_parser::Face;
use typst_kit::fonts::embedded;

use crate::output::{File, read};
use crate::photo::weigh;
use crate::site::Build;

/// For each class of track, from the vertex outwards.
pub const FIRE: [[&str; 3]; 3] =
    [["#fffbe6", "#ffdf7e", "#ffc85e"], ["#ffeab0", "#ffb45c", "#ff9248"], ["#ffc08a", "#ff8458", "#ff6e50"]];

const SVG: &str = "http://www.w3.org/2000/svg";

/// The event as every page first shows it, about the vertex and along the arrow. `site.js` draws it the same way.
fn tracks(source: &Source, lit: &[&str], radius: f32, weight: f32) -> Markup {
    let (event, camera) = (Event::new(source), Camera::new(START));
    let shown = |jet: usize| match source {
        Source::Site(posts) if !lit.is_empty() && !lit.contains(&&*posts[jet].name) => 0.1,
        _ => 1.0,
    };
    let at = |[x, y]: [f32; 2]| format!("{:.1} {:.1}", x * radius, y * radius);
    html! {
        @for (class, [near, middle, far]) in FIRE.iter().enumerate() {
            radialGradient id={ "fire" (class) } gradientUnits="userSpaceOnUse" cx="0" cy="0" r=(radius * 1.9) {
                stop stop-color=(near) {}
                stop stop-color=(middle) offset=(if class == 2 { 0.14 } else { 0.3 }) {}
                stop stop-color=(far) offset="1" {}
            }
        }
        g fill="none" stroke="#e4ecff" stroke-linecap="round" stroke-linejoin="round" {
            @for hit in &event.hits {
                @let (width, opacity) = [(1.6, 0.55), (2.3, 0.75), (3.3, 0.92)][hit.size as usize];
                path d={ "M" (at(hit.seen(camera))) "h.01" } stroke-width=(width * weight)
                    opacity=(opacity * shown(hit.jet)) {}
            }
            @for track in &event.tracks {
                path d={ "M" (track.seen(camera, 1.0).map(at).collect::<Vec<_>>().join(" ")) }
                    stroke={ "url(#fire" (track.class) ")" } stroke-width=(track.width * weight)
                    opacity=(track.opacity * shown(track.jet)) {}
            }
        }
    }
}

/// `lit` names the posts that stand out, if not all of them.
pub fn figure(build: &Build, source: &Source, lit: &[&str], scene: &Scene, water: Option<&str>) -> Result<Markup> {
    let r = scene.radius;
    let still = html! {
        svg xmlns=(SVG) viewBox={ (-2.5 * r) " " (-4.0 * r) " " (5.0 * r) " " (8.0 * r) } {
            (tracks(source, lit, r, 1.1))
        }
    };
    let still = build.output.asset("event.svg", File::Made(still.0.into_bytes()))?;
    let shaft = html! { svg.shaft aria-hidden="true" { line x1="7" x2="100%" {} } };
    let lit = (!lit.is_empty()).then(|| lit.join(" "));
    Ok(html! {
        div.figure data-program=(build.style.collider) data-event=(source) data-lit=[lit] data-water=[water] {
            div.mural {}
            div.arrow {
                (shaft)
                a.farm href=(build.config.farm.story) { span { (build.config.farm.at) } }
            }
            div.water {
                canvas {}
                div.arrow {
                    noscript { img src=(still) alt=""; }
                    (shaft)
                }
            }
            canvas.bed {}
        }
    })
}

const CARD: [u32; 2] = [1200, 630];
/// The size of the mural on a card, and where its corner goes.
const SCALE: f32 = 0.62;
const CORNER: [i32; 2] = [-80, -12];

fn picture(path: &Path, [width, height]: [u32; 2]) -> Result<Pixmap> {
    let image = weigh(image::load_from_memory(&read(path)?)?.into_rgba8(), |alpha| alpha);
    let pixels = resize(&image, width, height, FilterType::Triangle).into_raw();
    IntSize::from_wh(width, height).and_then(|size| Pixmap::from_vec(pixels, size)).context("an empty picture")
}

fn drawing(fonts: &Options, content: Markup) -> Result<Tree> {
    Ok(Tree::from_str(&html! { svg xmlns=(SVG) width="2000" height="2000" { (content) } }.0, fonts)?)
}

fn roman(size: usize) -> String {
    format!("font: 300 {size}px Spectral; letter-spacing: -.025em")
}

/// Each letter in the first font that has it. None has emoji.
fn letters(fonts: &Options, line: &str) -> Markup {
    let glyph = |c, font| fonts.fontdb.with_face_data(font, |it, at| Face::parse(it, at).ok()?.glyph_index(c))?;
    html! { @for c in line.chars() {
        @match fonts.fontdb.faces().enumerate().find(|(_, font)| glyph(c, font.id).is_some()) {
            Some((0, _)) => (c),
            Some((_, font)) => tspan font-family=(font.families[0].0) { (c) },
            None => {},
        }
    } }
}

/// The title in lines, as large as there is room for.
fn wrap(fonts: &Options, title: &str) -> (usize, Vec<String>) {
    let fits = |size, line: &str| {
        let text = drawing(fonts, html! { text style=(roman(size)) { (letters(fonts, line)) } });
        text.is_ok_and(|text| text.root().abs_bounding_box().is_none_or(|it| it.width() <= 560.0))
    };
    let set = |size| {
        let mut lines: Vec<String> = Vec::new();
        for word in title.split_whitespace() {
            match lines.last_mut() {
                Some(line) if fits(size, &format!("{line} {word}")) => line.extend([" ", word]),
                _ => lines.push(word.to_owned()),
            }
        }
        (size, lines)
    };
    let roomy = |(size, lines): &(usize, Vec<String>)| size * lines.len() < 270 && lines.iter().all(|l| fits(*size, l));
    [84, 66, 52].into_iter().map(set).find(roomy).unwrap_or_else(|| set(40))
}

/// Returns the address of the card. The site's own has no `title`.
pub fn card(build: &Build, source: &Source, title: Option<&str>, page: &str) -> Result<String> {
    let (config, fired) = (build.config, &build.style.fired);
    let tracks = tracks(source, &[], HOME.radius, if title.is_some() { 2.4 } else { 1.8 });
    let words = format!("{title:?} {} {} {fired:?} {:?}", config.title, config.subtitle, build.tools.fonts);
    let inputs = [include_bytes!("figure.rs"), tracks.0.as_bytes(), words.as_bytes()];
    let folder = build.tools.cache.entry(&inputs, |folder| {
        let mut fonts = Options::default();
        for font in ["Spectral-Light.ttf", "BonaNova-Italic.ttf"] {
            fonts.fontdb_mut().load_font_data(read(&build.tools.fonts.join(font))?);
        }
        embedded().for_each(|(font, _)| fonts.fontdb_mut().load_font_data(font.data().to_vec()));
        fonts.font_resolver.select_fallback = Box::new(|_, _, _| None);
        let paste = |onto: &mut Pixmap, [x, y]: [i32; 2], piece: &Pixmap| {
            onto.draw_pixmap(x, y, piece.as_ref(), &PixmapPaint::default(), Transform::identity(), None);
        };

        let size = HOME.size.map(|side| (side * SCALE) as u32);
        let ([vx, vy], turn) = (HOME.vertex(), HOME.turn().to_degrees());
        let along = Transform::from_scale(SCALE, SCALE).pre_translate(vx, vy).pre_rotate(turn);
        let shaft = |colour| {
            html! { path d={ "M9 0H" (HOME.length - 14.0) } stroke=(colour) stroke-width="2.2" stroke-dasharray="11 9" {} }
        };
        let mut mural = picture(&fired["home"].join("day.webp"), size)?;
        let ashore = html! {
            (shaft("#241712"))
            path d={ "M" (HOME.length) " 0l-18 6.4v-12.8z" } fill="#241712" {}
            path d="M-10 0h20M0 -10v20" transform={ "translate(" (HOME.farm()) ") rotate(" (-turn) ")" }
                stroke="#8f2f10" stroke-width="3" stroke-linecap="round" {}
        };
        resvg::render(&drawing(&fonts, ashore)?, along, &mut mural.as_mut());
        let afloat = html! {
            (tracks) (shaft("#f3ead2"))
            circle r="8" fill="#050b38" stroke="#f3ead2" stroke-width="2" {}
        };
        let mut lake = Pixmap::new(size[0], size[1]).unwrap();
        resvg::render(&drawing(&fonts, afloat)?, along, &mut lake.as_mut());
        lake.apply_mask(&Mask::from_pixmap(picture(&fired["home"].join("water.webp"), size)?.as_ref(), MaskType::Alpha));
        paste(&mut mural, [0, 0], &lake);

        let mut card = Pixmap::new(CARD[0], CARD[1]).unwrap();
        let wall = picture(&fired["wall"].join("day.webp"), [512; 2])?;
        (0..6).for_each(|tile| paste(&mut card, [tile % 3 * 512, tile / 3 * 512], &wall));
        paste(&mut card, CORNER, &mural);

        let [x, y] = [0, 1].map(|i| HOME.tip[i] * SCALE + CORNER[i] as f32 + [16.0, 12.0][i]);
        let voice = |size| format!("font: italic {size}px 'Bona Nova'");
        let text = match title {
            Some(title) => html! {
                @let (size, lines) = wrap(&fonts, title);
                @for (row, line) in lines.iter().enumerate() {
                    text x=(x) y=(y + (row * size) as f32 * 1.06) style=(roman(size)) { (letters(&fonts, line)) }
                }
                text x="1140" y="572" text-anchor="end" style=(voice(40)) fill="#17389c" { (config.title) }
            },
            None => html! {
                text x=(x) y=(y + 6.0) style=(voice(92)) { (config.title) }
                text x=(x + 4.0) y=(y + 66.0) style=(roman(31)) { (config.subtitle) }
            },
        };
        let text = drawing(&fonts, html! { g fill="#241712" { (text) } })?;
        resvg::render(&text, Transform::identity(), &mut card.as_mut());

        let card = DynamicImage::from(RgbaImage::from_raw(CARD[0], CARD[1], card.take()).unwrap()).into_rgb8();
        let mut jpeg = Vec::new();
        JpegEncoder::new_with_quality(&mut jpeg, 88).encode_image(&card)?;
        Ok(fs::write(folder.join("card.jpg"), jpeg)?)
    })?;
    build.output.lasting(&format!("/cards{}.jpg", page.trim_end_matches('/')), File::Copied(folder.join("card.jpg")))
}
