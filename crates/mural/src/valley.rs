//! The drawing: `valley.svg`, in metres east and south of the farm.

use std::collections::HashMap;

use anyhow::{Context, Result, ensure};
use roxmltree::Document;
use svgtypes::{SimplePathSegment as Segment, SimplifyingPathParser};
use tiny_skia::{BlendMode, FillRule, Mask, Paint as Brush, Path, PathBuilder, Pixmap, Stroke, Transform};

use crate::field::Field;
use crate::glaze::{self, named};
use crate::{Paint, step};

pub struct Valley {
    /// Groups of shapes, each a way of cutting the mural out.
    cuts: HashMap<String, Vec<(String, Path)>>,
    river: Path,
}

/// Where the valley sits on a wall.
#[derive(Clone, Copy)]
pub struct View {
    pub cut: &'static str,
    /// A point of the valley, and where on the wall it goes.
    pub anchor: [f32; 2],
    pub at: [f32; 2],
    /// Design pixels per metre.
    pub scale: f32,
    /// Clockwise, in radians.
    pub turn: f32,
    /// How far around `at` the water runs dark.
    pub dark: f32,
    /// Between these heights the lake gives way to plain deep tiles.
    pub sink: Option<[f32; 2]>,
}

fn outline(d: &str) -> Result<Path> {
    let mut path = PathBuilder::new();
    for segment in SimplifyingPathParser::from(d) {
        match segment? {
            Segment::MoveTo { x, y } => path.move_to(x as f32, y as f32),
            Segment::LineTo { x, y } => path.line_to(x as f32, y as f32),
            Segment::Quadratic { x1, y1, x, y } => path.quad_to(x1 as f32, y1 as f32, x as f32, y as f32),
            Segment::CurveTo { x1, y1, x2, y2, x, y } => {
                path.cubic_to(x1 as f32, y1 as f32, x2 as f32, y2 as f32, x as f32, y as f32);
            }
            Segment::ClosePath => path.close(),
        }
    }
    path.finish().context("a path is empty")
}

impl Valley {
    pub fn parse(svg: &str) -> Result<Self> {
        let (mut cuts, mut river) = (HashMap::<_, Vec<_>>::new(), None);
        for node in Document::parse(svg)?.descendants().filter(|node| node.has_tag_name("path")) {
            ensure!(node.ancestors().all(|node| !node.has_attribute("transform")), "transforms have to be applied");
            let (class, path) = (node.attribute("class").unwrap_or(""), outline(node.attribute("d").unwrap_or(""))?);
            if class == "river" {
                river = Some(path);
            } else {
                ensure!(class == "lake" || glaze::find(class).is_some(), "there is no glaze called `{class}`");
                let cut = node.parent().and_then(|group| group.attribute("id")).unwrap_or("");
                cuts.entry(cut.to_owned()).or_default().push((class.to_owned(), path));
            }
        }
        Ok(Self { cuts, river: river.context("there is no river")? })
    }
}

/// The valley traced onto a wall at full size, ready to be glazed.
pub struct Cartoon {
    view: View,
    density: f32,
    width: usize,
    /// For each pixel, two glazes and the share of the second.
    coats: Vec<[u8; 3]>,
    edge: Field,
    inside: Field,
    depth: Field,
    river: Field,
    cobalt: usize,
    deep: usize,
}

/// How calm the plain deep tiles are, which carry the end of every page.
pub const DEEP_CALM: f32 = 0.72;

/// Distances near the border depend on what lies past it.
const MARGIN: f32 = 24.0;

/// A loose shape, to cut clay or lay glaze by.
pub struct Stencil {
    inside: Field,
    density: f32,
}

impl Stencil {
    pub fn trace(d: &str, size: [f32; 2], density: f32) -> Self {
        let [width, height] = size.map(|side| ((side + 2.0 * MARGIN) * density).ceil() as u32);
        let place = Transform::from_translate(MARGIN, MARGIN).post_scale(density, density);
        let mut mask = Mask::new(width, height).expect("the wall has a size");
        mask.fill_path(&outline(d).expect("a valid outline"), FillRule::Winding, false, place);
        let inside = Field::signed_distance(width as usize, height as usize, |i| mask.data()[i] > 0).soften(4);
        Self { inside, density }
    }

    /// The distance into the shape.
    pub fn inside(&self, x: f32, y: f32) -> f32 {
        self.inside.at((x + MARGIN) * self.density - 0.5, (y + MARGIN) * self.density - 0.5) / self.density
    }
}

impl Cartoon {
    pub fn trace(valley: &Valley, view: View, size: [f32; 2], density: f32) -> Result<Self> {
        let shapes = valley.cuts.get(view.cut).with_context(|| format!("the valley has no group `{}`", view.cut))?;
        let [width, height] = size.map(|side| ((side + 2.0 * MARGIN) * density).ceil() as u32);
        let place = Transform::from_translate(-view.anchor[0], -view.anchor[1])
            .post_rotate(view.turn.to_degrees())
            .post_scale(view.scale, view.scale)
            .post_translate(view.at[0] + MARGIN, view.at[1] + MARGIN)
            .post_scale(density, density);
        let mask = |paths: &mut dyn Iterator<Item = &Path>| {
            let mut mask = Mask::new(width, height).expect("the wall has a size");
            paths.for_each(|path| mask.fill_path(path, FillRule::EvenOdd, false, place));
            mask
        };
        let (sand, cobalt) = (named("sand"), named("cobalt"));

        // Each shape covers what is under it, and leaves its own outline.
        let mut coats = vec![[sand as u8, sand as u8, 0]; (width * height) as usize];
        let mut lines = Pixmap::new(width, height).expect("the wall has a size");
        let eraser = Brush { blend_mode: BlendMode::Clear, anti_alias: false, ..Brush::default() };
        let pen = Brush { anti_alias: false, ..Brush::default() };
        let hair = Stroke { width: 1.2 / (view.scale * density), ..Stroke::default() };
        let mut coat = Mask::new(width, height).expect("the wall has a size");
        for (class, path) in shapes.iter().filter(|(class, _)| class != "sand") {
            let Some(bounds) = path.bounds().transform(place) else { continue };
            let span = |from: f32, to: f32, limit: u32| {
                (from.max(0.0) as usize).min(limit as usize)..(to.ceil().max(0.0) as usize).min(limit as usize)
            };
            let (columns, rows) =
                (span(bounds.left(), bounds.right(), width), span(bounds.top(), bounds.bottom(), height));
            if columns.is_empty() || rows.is_empty() {
                continue;
            }
            let glaze = if class == "lake" { cobalt } else { named(class) } as u8;
            coat.fill_path(path, FillRule::EvenOdd, true, place);
            for row in rows {
                let range = row * width as usize + columns.start..row * width as usize + columns.end;
                for (under, share) in coats[range.clone()].iter_mut().zip(&mut coat.data_mut()[range]) {
                    match std::mem::take(share) {
                        0 => {}
                        255 => *under = [glaze, glaze, 0],
                        share => *under = [if under[2] > 127 { under[1] } else { under[0] }, glaze, share],
                    }
                }
            }
            lines.fill_path(path, &eraser, FillRule::EvenOdd, place, None);
            lines.stroke_path(path, &pen, &hair, place, None);
        }

        let of = |wanted: &'static [&str]| {
            shapes.iter().filter(|(class, _)| wanted.contains(&class.as_str())).map(|(_, path)| path)
        };
        let (ground, lake) = (mask(&mut of(&["sand", "lake"])), mask(&mut of(&["lake"])));
        let river = mask(&mut std::iter::once(&valley.river));
        let (w, h) = (width as usize, height as usize);
        Ok(Self {
            view,
            density,
            width: w,
            coats,
            edge: Field::distance(w, h, |i| lines.data()[i * 4 + 3] > 0).soften(2),
            inside: Field::signed_distance(w, h, |i| ground.data()[i] > 0).soften(3),
            depth: Field::signed_distance(w, h, |i| lake.data()[i] > 0),
            river: Field::signed_distance(w, h, |i| river.data()[i] > 0).soften(3),
            cobalt,
            deep: named("deep"),
        })
    }

    pub fn paint(&self, x: f32, y: f32) -> Paint {
        let (px, py) = ((x + MARGIN) * self.density - 0.5, (y + MARGIN) * self.density - 0.5);
        let [first, second, share] = self.coats[(py + 0.5) as usize * self.width + (px + 0.5) as usize];
        let mut paint = Paint {
            glazes: [first as usize, second as usize],
            blend: f32::from(share) / 255.0,
            body: 0.0,
            edge: self.edge.at(px, py) / self.density,
            inside: self.inside.at(px, py) / self.density,
            offshore: self.depth.at(px, py) / self.density,
            calm: 0.0,
        };
        let depth = paint.offshore / self.view.scale;
        if depth > 0.0 {
            // Cobalt lies thicker over deep water, and thinner along the old river.
            let river = self.river.at(px, py) / self.density;
            let around = (x - self.view.at[0]).hypot(y - self.view.at[1]) / self.view.dark;
            paint.body = -0.16 + 0.56 * step(0.0, 170.0, depth) + 0.52 * (-around * around).exp()
                - 0.40 * step(-1.2, 2.2, river)
                + 0.20 * (-(river / 2.6).powi(2)).exp();
            if let (Some([from, to]), true) = (self.view.sink, paint.glazes == [self.cobalt; 2]) {
                paint.glazes[1] = self.deep;
                paint.blend = step(from, to, y);
                paint.body *= 1.0 - paint.blend;
                paint.calm = DEEP_CALM * paint.blend;
            }
        }
        paint
    }
}
