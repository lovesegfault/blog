//! The drawing, `valley.svg`, in metres east and south of the farm, and its chart, `depth.pgm`.

use std::collections::HashMap;

use anyhow::{Context, Result, bail, ensure};
use roxmltree::Document;
use svgtypes::{SimplePathSegment as Segment, SimplifyingPathParser, ViewBox};
use tiny_skia::{BlendMode, FillRule, Mask, Paint as Brush, Path, PathBuilder, Pixmap, Point, Stroke, Transform};

use crate::field::Field;
use crate::glaze::{self, named};
use crate::{Paint, step};

pub struct Valley {
    /// Groups of shapes: the ways of cutting the mural out, and the bed of the lake.
    cuts: HashMap<String, Vec<(String, Path)>>,
    river: Path,
    /// How deep the lake is, and the way from metres to its cells.
    chart: Field,
    cells: Transform,
}

/// What the chart has for no depth.
const SHORE: f32 = 25.0;

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
    /// As it was before the dam, and only what the lake covers.
    pub dry: bool,
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
    pub fn parse(svg: &str, chart: &[u8]) -> Result<Self> {
        let (mut cuts, mut river, svg) = (HashMap::<_, Vec<_>>::new(), None, Document::parse(svg)?);
        for node in svg.descendants().filter(|node| node.has_tag_name("path")) {
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
        let parts: Vec<_> = chart.splitn(5, u8::is_ascii_whitespace).collect();
        let &[b"P5", width, height, b"255", depths] = &parts[..] else { bail!("depth.pgm is not a chart") };
        let (width, height) = (str::from_utf8(width)?.parse()?, str::from_utf8(height)?.parse()?);
        ensure!(depths.len() == width * height, "depth.pgm is cut short");
        let ViewBox { x, y, w, h } = svg.root_element().attribute("viewBox").unwrap_or("").parse()?;
        Ok(Self {
            cuts,
            river: river.context("there is no river")?,
            chart: Field { width, height, data: depths.iter().copied().map(f32::from).collect() },
            cells: Transform::from_translate(-x as f32, -y as f32)
                .post_scale(width as f32 / w as f32, height as f32 / h as f32)
                .post_translate(-0.5, -0.5),
        })
    }
}

/// The valley traced onto a wall at full size, ready to be glazed.
pub struct Cartoon<'a> {
    view: View,
    density: f32,
    width: usize,
    /// For each pixel, two glazes and the share of the second.
    coats: Vec<[u8; 3]>,
    edge: Field,
    inside: Field,
    depth: Field,
    river: Field,
    /// Where it is dry: the distance into the lake to come, its chart, and the way from the wall to that.
    flood: Option<(Field, &'a Field, Transform)>,
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

impl<'a> Cartoon<'a> {
    pub fn trace(valley: &'a Valley, view: View, size: [f32; 2], density: f32) -> Result<Self> {
        let group = |name| valley.cuts.get(name).with_context(|| format!("the valley has no group `{name}`"));
        let (cut, bed) = (group(view.cut)?, if view.dry { group("bed")? } else { &[][..] });
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
        let bare = ["sand", if view.dry { "lake" } else { "" }];
        for (class, path) in cut.iter().filter(|(class, _)| !bare.contains(&&**class)).chain(bed) {
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

        let (w, h) = (width as usize, height as usize);
        let within = |shapes: &[(String, Path)], wanted: &[&str]| {
            let mask = mask(&mut shapes.iter().filter(|(class, _)| wanted.contains(&&**class)).map(|(_, path)| path));
            Field::signed_distance(w, h, |i| mask.data()[i] > 0)
        };
        let river = mask(&mut std::iter::once(&valley.river));
        let cells = place.invert().expect("the valley has a size").post_concat(valley.cells).pre_translate(0.5, 0.5);
        Ok(Self {
            view,
            density,
            width: w,
            coats,
            edge: Field::distance(w, h, |i| lines.data()[i * 4 + 3] > 0).soften(2),
            inside: within(cut, &["sand", "lake"]).soften(3),
            depth: within(if view.dry { bed } else { cut }, &["lake"]),
            river: Field::signed_distance(w, h, |i| river.data()[i] > 0).soften(3),
            flood: view.dry.then(|| (within(cut, &["lake"]), &valley.chart, cells)),
            cobalt,
            deep: named("deep"),
        })
    }

    pub fn paint(&self, x: f32, y: f32, tile: [i32; 2]) -> Paint {
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
            shown: true,
            sounding: 0.0,
        };
        if let Some((lake, chart, cells)) = &self.flood {
            let (into, mut cell) = (lake.at(px, py) / self.density, Point::from_xy(px, py));
            cells.map_point(&mut cell);
            // From the line of the river to past that of the lake, short of the cut and of the farm.
            paint.shown = paint.offshore < 2.0 && into > -15.0 && paint.inside > 4.0 && tile != [0; 2];
            // The lesser of the chart and a shelf that starts at the shore, with no corner between them.
            let (charted, shelf) = (chart.smooth(cell.x, cell.y), SHORE + 5.0 * into / self.view.scale);
            paint.sounding = charted.min(shelf) - 8.0 * ((charted - shelf).abs() / -8.0).exp().ln_1p();
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chart_lies_over_the_drawing() {
        let svg = r#"<svg viewBox="-40 0 80 40"><path class="river" d="M0 0h1v1z"/></svg>"#;
        let chart = [b"P5 8 4 255\n".as_slice(), &[0, 10, 20, 30, 40, 50, 60, 70].repeat(4)].concat();
        let (valley, mut at) = (Valley::parse(svg, &chart).unwrap(), Point::from_xy(-3.0, 20.0));
        valley.cells.map_point(&mut at);
        assert!((valley.chart.smooth(at.x, at.y) - 32.0).abs() < 0.01);
        assert!(Valley::parse(svg, &chart[..40]).is_err());
    }
}
