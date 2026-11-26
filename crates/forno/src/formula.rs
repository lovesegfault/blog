//! Formulas as Typst sets them on paper, cut where a line may end so that browsers can fit them to the screen.

use std::collections::HashSet;
use std::fmt::Write;
use std::iter::{once, successors};
use std::mem::{replace, take};
use std::ops::ControlFlow;
use std::sync::LazyLock;

use ttf_parser::{GlyphId, OutlineBuilder};
use typst::World;
use typst::comemo::Tracked;
use typst::engine::Engine;
use typst::foundations::{Content, NativeElement, NativeRuleMap, SequenceElem, ShowFn, StyledElem, SymbolElem, Target};
use typst::introspection::{Counter, Location, Locator, Tag, TagElem, TagFlags};
use typst::layout::{
    Abs, Axes, BlockBody, BlockElem, BoxElem, Em, Frame, FrameItem, HElem, InlineElem, InlineItem, Point, Rect, Region,
    Rel, Size,
};
use typst::math::{AlignPointElem, AttachElem, ClassElem, EquationElem, LrElem, MidElem};
use typst::model::LateLinkResolver;
use typst::syntax::{Span, SyntaxKind, SyntaxNode};
use typst::text::color::should_outline;
use typst::text::{Glyph, LinebreakElem, SpaceElem, TextElem, TextItem};
use typst::utils::{default_math_class, hash128};
use typst::visualize::{Color, Geometry, Paint, Shape};
use typst_html::{FrameElem, HtmlAttr, HtmlElem};
use unicode_math_class::MathClass;

use crate::tree::{Node, plain};
use crate::typeset::is_written;

/// Typst's MathML, followed by the drawing.
pub const RULE: ShowFn<EquationElem> = |equation, engine, styles| {
    static TYPST: LazyLock<NativeRuleMap> = LazyLock::new(|| {
        let mut rules = NativeRuleMap::new();
        typst_html::register(&mut rules);
        rules
    });
    let number = match equation.numbering.get_ref(styles) {
        Some(numbering) if equation.block.get(styles) => {
            let location = equation.location().unwrap();
            Counter::of(EquationElem::ELEM).display_at(engine, location, styles, numbering, equation.span())?
        }
        _ => Content::empty(),
    };
    let (mut spoken, mut drawn) = (equation.clone(), equation.clone());
    let body = equation.body.clone().set(LrElem::size, Rel::one());
    spoken.body = equation.alt.get_cloned(styles).map_or(body, TextElem::packed);
    let mut math = TYPST.get(Target::Html, spoken.pack_ref()).unwrap().apply(spoken.pack_ref(), engine, styles)?;
    if let Some(BlockBody::Content(inside)) = math.to_packed::<BlockElem>().and_then(|it| it.body.get_cloned(styles)) {
        math = inside;
    }
    let math = math.into_packed::<HtmlElem>().unwrap().unpack();

    drawn.body = marked(&equation.body, 0, &mut false) + mark(LINE, 0);
    drawn.block.set(true);
    drawn.numbering.set(None);
    let drawn = InlineElem::layouter(drawn, |drawn, engine, _, styles, room| {
        let paged = engine.library.rules.get(Target::Paged, drawn.pack_ref()).unwrap();
        let paged = paged.apply(drawn.pack_ref(), engine, styles)?.into_packed::<BlockElem>().unwrap();
        let Some(BlockBody::MultiLayouter(layout)) = paged.body.get_ref(styles) else { unreachable!() };
        // What is in both copies is counted once.
        let (locator, room) = (Locator::synthesize(drawn.location().unwrap()), Region::new(room, Axes::splat(false)));
        Ok(vec![InlineItem::Frame(layout.call(engine, locator, styles, room.into())?.into_frame())])
    });
    // An apostrophe after this is not taken for an opening quote.
    let after = FrameElem::new(drawn.pack()).pack() + TextElem::packed('\u{FFFC}');
    let body = math.body.get_cloned(styles).unwrap_or_default() + after;
    let source = [equation.span(), equation.body.span()].into_iter().find_map(|span| source(engine, span));
    Ok(math
        .with_attr(HtmlAttr::constant("number"), number.plain_text())
        .with_optional_attr(HtmlAttr::constant("source"), source)
        .with_body(Some(body))
        .pack()
        .spanned(equation.body.traverse(&mut written).break_value().unwrap_or(Span::detached())))
};

fn written(part: Content) -> ControlFlow<Span> {
    if is_written(part.span().id()) { ControlFlow::Break(part.span()) } else { ControlFlow::Continue(()) }
}

/// Without its comments, and only if it says all there is to say.
fn source(engine: &Engine, span: Span) -> Option<String> {
    use SyntaxKind::*;
    fn bare(node: &SyntaxNode, around: SyntaxKind) -> String {
        let blank = node.kind().is_trivia() && (around, node.kind()) != (Code, Space);
        let own = if blank { " " } else { node.leaf_text() };
        node.children().fold(own.into(), |text, child| text + &bare(child, node.kind()))
    }
    let file = engine.world.source(span.id()?).ok()?;
    let node = file.find(span).filter(|node| node.kind() == Equation)?;
    let mut around = successors(Some(&node), |node| node.parent());
    around.all(|node| !matches!(node.kind(), Closure | ForLoop | WhileLoop)).then(|| bare(&node, Markup))
}

// What comes after a mark. Lines end at the first of these that they can.
const LINE: u8 = 0;
const SPACE: u8 = 1;
const COMMA: u8 = 2;
const ARROW: u8 = 3;
const WORDS: u8 = 4;
const LOGIC: u8 = 5;
const RELATION: u8 = 6;
const SUM: u8 = 7;
const PRODUCT: u8 = 8;
const BRACKET: u8 = 9;
const OTHER: u8 = 10;

fn mark(kind: u8, depth: u8) -> Content {
    let flags = TagFlags { introspectable: false, tagged: false };
    TagElem::packed(Tag::End(Location::new(kind.into()), depth.into(), flags))
}

fn kind(part: &Content) -> u8 {
    let sign = part.to_packed::<SymbolElem>().and_then(|symbol| symbol.text.chars().next());
    let base = ["base", "body"].iter().find_map(|field| part.get_by_name(field).ok()?.cast().ok());
    match part.to_packed::<ClassElem>().map(|given| given.class).or(sign.and_then(default_math_class)) {
        _ if part.is::<LinebreakElem>() => LINE,
        _ if part.is::<AlignPointElem>() => SPACE,
        _ if part.is::<BoxElem>() => OTHER,
        _ if part.is::<MidElem>() || sign.is_some_and(|sign| "⇐⇒⇔⟸⟹⟺→↔∴".contains(sign)) => ARROW,
        _ if part.to_packed::<TextElem>().is_some_and(|it| it.text.contains(char::is_alphabetic)) => WORDS,
        _ if sign.is_some_and(|sign| "∧∨".contains(sign)) => LOGIC,
        Some(MathClass::Punctuation) => COMMA,
        Some(MathClass::Relation) => RELATION,
        Some(MathClass::Vary) => SUM,
        Some(MathClass::Binary) => PRODUCT,
        _ => base.map(|base| kind(&base)).filter(|kind| *kind > SPACE).unwrap_or(OTHER),
    }
}

fn ends_in_brackets(part: &Content) -> bool {
    let last = part.to_packed::<SequenceElem>().and_then(|parts| parts.children.last());
    last.or(part.to_packed::<EquationElem>().map(|it| &it.body)).map_or(part.is::<LrElem>(), ends_in_brackets)
}

/// With a mark before each part, which the layout carries along.
fn marked(part: &Content, depth: u8, opening: &mut bool) -> Content {
    let mut before = |kind| if take(opening) { Content::empty() } else { mark(kind, depth) };
    if part.label().is_some() {
        before(OTHER) + part.clone()
    } else if let Some(parts) = part.to_packed::<SequenceElem>() {
        Content::sequence(parts.children.iter().map(|part| marked(part, depth, opening)))
    } else if let Some(styled) = part.to_packed::<StyledElem>() {
        marked(&styled.child, depth, opening).styled_with_map(styled.styles.clone())
    } else if let Some(nested) = part.to_packed::<EquationElem>() {
        marked(&nested.body, depth, opening)
    } else if let Some(mut brackets) = part.to_packed::<LrElem>().cloned() {
        brackets.body = marked(&brackets.body, depth.saturating_add(1), &mut true);
        before(BRACKET) + brackets.pack()
    } else if let Some(mut raised) = part.to_packed::<AttachElem>().cloned()
        && ends_in_brackets(&raised.base)
        && ["tl", "bl", "tr", "br"].iter().all(|corner| part.get_by_name(corner).is_err())
    {
        raised.base = marked(&raised.base, depth, &mut true);
        before(BRACKET) + raised.pack()
    } else if part.is::<SpaceElem>() || part.to_packed::<HElem>().is_some_and(|space| space.weak.get(<_>::default())) {
        part.clone()
    } else {
        before(kind(part)) + part.clone()
    }
}

/// What is between two marks. Lengths are in ems.
#[derive(Default)]
struct Atom {
    kind: u8,
    depth: u8,
    items: Vec<(Point, FrameItem)>,
    baseline: f64,
    left: f64,
    right: f64,
    top: f64,
    bottom: f64,
    /// How bad it is for a line to start here, if one can.
    cut: Option<(u8, u8)>,
    aligned: bool,
}

struct Scribe<'a> {
    em: Abs,
    displayed: bool,
    links: Tracked<'a, LateLinkResolver<'a>>,
    indent: f64,
    /// How far the pieces go when every line that can end does.
    reach: f64,
    /// Of the line before.
    bottom: f64,
    html: String,
}

fn flatten(frame: &Frame, offset: Point, flat: &mut Vec<(Point, FrameItem)>) {
    for (at, item) in frame.items() {
        match item {
            FrameItem::Group(group) if group.transform.is_identity() && group.clip.is_none() => {
                flatten(&group.frame, offset + *at, flat);
            }
            _ => flat.push((offset + *at, item.clone())),
        }
    }
}

fn bounds(item: &FrameItem) -> Option<Rect> {
    match item {
        FrameItem::Text(text) => {
            let last = text.glyphs.iter().rposition(|glyph| !text.text[glyph.range()].trim().is_empty())?;
            let end = text.glyphs[..=last].iter().map(|glyph| glyph.x_advance.at(text.size)).sum();
            let Rect { min, max } = text.bbox();
            Some(Rect::new(Point::with_y(max.y.min(Abs::zero())), Point::new(max.x, min.y).max(Point::with_x(end))))
        }
        FrameItem::Shape(shape, _) => Some(shape.bbox(true)),
        FrameItem::Group(group) => {
            let inside = group.frame.items().filter_map(|(at, it)| bounds(it).map(|it| (*at + it.min, *at + it.max)));
            let clip = group.clip.iter().map(|_| (Point::zero(), group.frame.size().to_point()));
            let corners: Vec<_> = (inside.filter(|_| group.clip.is_none()).chain(clip))
                .flat_map(|(a, b)| [a, b, Point::new(a.x, b.y), Point::new(b.x, a.y)])
                .map(|corner| corner.transform(group.transform))
                .collect();
            let [min, max] = [Point::min, Point::max].map(|pick| corners.iter().copied().reduce(pick));
            // The line that cancels a term takes no room from its neighbours.
            let past = match group.frame.items().as_slice() {
                [(_, FrameItem::Shape(Shape { geometry: Geometry::Line(_), .. }, _))] => (max?.x - min?.x) / 2.0,
                _ => Abs::zero(),
            };
            Some(Rect::new(min? + Point::with_x(past), max? - Point::with_x(past)))
        }
        FrameItem::Image(_, size, _) | FrameItem::Link(_, size) => Some(Rect::from_pos_size(Point::zero(), *size)),
        FrameItem::Tag(_) => None,
    }
}

impl Atom {
    fn new(kind: u8, depth: u8, baseline: f64, aligned: bool) -> Self {
        let (left, right, top, bottom) = (f64::MAX, f64::MIN, baseline, baseline);
        Self { kind, depth, baseline, left, right, top, bottom, aligned, ..<_>::default() }
    }

    fn push(&mut self, at: Point, item: FrameItem, em: Abs) {
        let Some(Rect { min, max }) = bounds(&item) else { return };
        let (min, max) = (at + min, at + max);
        (self.left, self.top) = (self.left.min(min.x / em), self.top.min(min.y / em));
        (self.right, self.bottom) = (self.right.max(max.x / em), self.bottom.max(max.y / em));
        self.items.push((at, item));
    }

    /// By the word, if words are all that it has.
    fn words(self, em: Abs) -> Vec<Self> {
        let mut words = vec![Self::new(WORDS, self.depth, self.baseline, self.aligned)];
        for &(mut at, ref item) in &self.items {
            let text = match item {
                _ if self.kind != WORDS => return vec![self],
                FrameItem::Text(text) if (at.y / em - self.baseline).abs() < 0.01 => text,
                _ => return vec![Self { kind: OTHER, ..self }],
            };
            let apart = |last: &Glyph, next: &Glyph| {
                let (before, after) = text.text.split_at(next.range().start);
                last.range.end == next.range.start && before.ends_with(' ') && after.starts_with(char::is_alphabetic)
            };
            for (count, word) in text.glyphs.chunk_by(|last, next| !apart(last, next)).enumerate() {
                if count > 0 && !words[0].items.is_empty() {
                    words.push(Self::new(OTHER, self.depth, self.baseline, false));
                }
                let word = TextItem { glyphs: word.into(), ..text.clone() };
                let next = at + Point::with_x(word.width());
                words.last_mut().unwrap().push(replace(&mut at, next), FrameItem::Text(word), em);
            }
        }
        words.retain(|word| !word.items.is_empty());
        words
    }
}

fn lines(frame: &Frame, em: Abs, displayed: bool) -> Vec<Vec<Atom>> {
    let (mut flat, mut lines, mut atom) = (Vec::new(), vec![Vec::<Atom>::new()], Atom::default());
    let mut as_marked = true;
    flatten(frame, Point::zero(), &mut flat);
    for (at, item) in flat {
        if let FrameItem::Tag(Tag::End(kind, depth, flags)) = item
            && !flags.any()
        {
            let (line, kind) = (lines.last_mut().unwrap(), kind.hash() as u8);
            let aligned = atom.items.is_empty() && atom.aligned && kind != LINE || kind == SPACE;
            for mut atom in replace(&mut atom, Atom::new(kind, depth as u8, at.y / em, aligned)).words(em) {
                as_marked &=
                    (at.y / em - atom.baseline).abs() < 0.01 && line.last().is_none_or(|it| it.left < atom.left);
                atom.cut = line.last().and_then(|last| cut(last, &atom, displayed));
                line.push(atom);
            }
            lines.extend((kind == LINE).then(Vec::new));
        } else {
            atom.push(at, item, em);
        }
    }
    lines.retain(|line| !line.is_empty());
    let whole = !as_marked || lines.is_empty() || lines.len() > 1 && !displayed;
    if whole {
        let atoms = lines.into_iter().flatten().chain([atom]);
        lines = vec![atoms.map(|atom| Atom { cut: None, aligned: false, ..atom }).collect()];
    }
    if whole || !displayed {
        (lines[0][0].left, lines[0].last_mut().unwrap().right) = (0.0, frame.width() / em);
    }
    lines
}

fn cut(last: &Atom, next: &Atom, displayed: bool) -> Option<(u8, u8)> {
    let joins = |atom: &Atom| matches!(atom.kind, ARROW | LOGIC..=PRODUCT);
    let (gap, big) = (next.left - last.right, |it: &Atom| it.right - it.left > 3.0 || it.bottom - it.top > 2.0);
    let kind = match (joins(last), joins(next)) {
        _ if gap > 3.0 => LINE,
        _ if gap > 0.7 => SPACE,
        _ if last.kind == COMMA && gap > 0.1 => COMMA,
        (true, _) if gap > 0.2 && !displayed => last.kind,
        (false, true) if gap > 0.2 && displayed => next.kind,
        (false, false) if gap > 0.2 => next.kind,
        (false, false) if (last.depth > next.depth || big(last)) && (next.kind == BRACKET || big(next)) => BRACKET,
        _ => return None,
    };
    Some((next.depth, kind))
}

fn width(atoms: &[Atom]) -> f64 {
    atoms[atoms.len() - 1].right - atoms[0].left
}

/// A thousand to the em.
fn units(ems: f64) -> f64 {
    (ems * 1000.0).round() + 0.0
}

fn css(ems: f64) -> String {
    format!("{}em", units(ems) / 1000.0)
}

#[derive(Default)]
struct Outline {
    path: String,
    pen: [f32; 2],
}

impl Outline {
    fn shift(&mut self, [x, y]: [Em; 2]) {
        self.pen = [self.pen[0] - x.get() as f32, self.pen[1] - y.get() as f32];
    }

    fn draw(&mut self, command: char, points: &[f32]) {
        self.path.push(command);
        for (to, from) in points.iter().zip(self.pen.iter().cycle()) {
            write!(self.path, "{:+}", (to.round() - from.round()) as i32).unwrap();
        }
        self.pen = *points.last_chunk().unwrap();
    }
}

impl OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.draw('m', &[x, y]);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.draw('l', &[x, y]);
    }

    fn quad_to(&mut self, a: f32, b: f32, x: f32, y: f32) {
        self.draw('q', &[a, b, x, y]);
    }

    fn curve_to(&mut self, a: f32, b: f32, c: f32, d: f32, x: f32, y: f32) {
        self.draw('c', &[a, b, c, d, x, y]);
    }

    fn close(&mut self) {}
}

/// As in the stylesheet.
const INDENT: f64 = 1.1;

/// What Typst leaves between lines, unless it is told otherwise.
const LEADING: f64 = 0.65;

/// Around the shape of a glyph, until the page is put together.
const SHAPE: &str = "<x-shape>";

impl Scribe<'_> {
    fn glyphs(&mut self, text: &TextItem, ink: &Color, mut at: Point) {
        let ink = if *ink == Color::BLACK { String::new() } else { format!(r#" fill="{}""#, ink.to_hex()) };
        let (scale, grid) = (units(text.size / self.em) / text.font.units_per_em(), text.font.units_per_em());
        // The parts of a stretched one are drawn together, or the seams would show.
        for parts in text.glyphs.chunk_by(|a, b| a.range == b.range) {
            let (mut outline, x, y) = (Outline::default(), units(at.x / self.em), units(at.y / self.em));
            for part in parts {
                outline.shift([part.x_offset, part.y_offset].map(|by| by * grid));
                text.font.ttf().outline_glyph(GlyphId(part.id), &mut outline);
                outline.shift([part.x_advance - part.x_offset, part.y_advance - part.y_offset].map(|by| by * grid));
                at += Point::new(part.x_advance.at(text.size), -part.y_advance.at(text.size));
            }
            let shape = format!(r#"transform="scale({scale} -{scale})" d="{}""#, outline.path);
            let id = format!("g{:x}", hash128(&shape) as u32);
            if !outline.path.is_empty() {
                write!(self.html, r##"<use href="#{id}" x="{x}" y="{y}"{ink}/>{SHAPE}{id}" {shape}{SHAPE}"##).unwrap();
            }
        }
    }

    /// It starts and ends like an English word, so that it keeps its punctuation and its place.
    fn piece(&mut self, atoms: &[Atom]) {
        let (first, wide, lead) = (&atoms[0], width(atoms).max(0.1), if self.displayed { 0.3 } else { 0.0 });
        let up = atoms.iter().fold(0.0, |up: f64, atom| up.max(first.baseline - atom.top)) + lead;
        let down = atoms.iter().fold(0.0, |down: f64, atom| down.max(atom.bottom - first.baseline)) + lead;
        let style = format!("margin-block:{} {};vertical-align:{}", css(up - 0.5), css(down), css(-down));
        let size = format!(r#"width="{}" viewBox="0 -500 {} 500""#, css(wide), units(wide));
        write!(self.html, "<b>\u{2063}\u{200E}<svg {size} style=\"{style}\">").unwrap();
        let (origin, start) = (Point::new(self.em * first.left, self.em * first.baseline), self.html.len());
        let mut rest = Frame::soft(Size::new(self.em * wide, self.em));
        for (at, item) in atoms.iter().flat_map(|atom| &atom.items) {
            if let FrameItem::Text(text @ TextItem { fill: Paint::Solid(ink), stroke: None, .. }) = item
                && text.glyphs.iter().all(|glyph| should_outline(&text.font, GlyphId(glyph.id)))
            {
                self.glyphs(text, ink, *at - origin);
            } else {
                rest.push(*at - origin, item.clone());
            }
        }
        if !rest.is_empty() {
            let rest = typst_svg::svg_in_html(&rest, self.em, false, None, "", &[], self.links);
            let rest = rest.split_once('>').unwrap().1.replace("</svg>", "</g>").replace("#000000\"", "currentColor\"");
            let scale = format!(r#"<g transform="scale({:.3})">"#, Abs::pt(1000.0) / self.em);
            self.html.insert_str(start, &(scale + &rest.replace("<a ", r#"<a tabindex="-1" "#)));
        }
        self.html += "</svg>\u{2063}\u{200E}</b>";
        self.reach = self.reach.max(self.indent + wide);
    }

    fn groups<'a>(&mut self, parts: impl Iterator<Item = &'a mut [Atom]>, indent: f64) {
        let (mut end, around) = (None, self.indent);
        for part in parts {
            if let Some(end) = end.replace(part[part.len() - 1].right) {
                part[0].left = part[0].left.max(end);
                write!(self.html, r#"<i style="--apart:{}"> </i>"#, css(part[0].left - end)).unwrap();
            }
            self.group(part);
            self.indent = around + indent;
        }
        self.indent = around;
    }

    /// On display, a line only ends inside a group that is too long for one.
    fn group(&mut self, atoms: &mut [Atom]) {
        let best = atoms[1..].iter().filter_map(|atom| atom.cut).min();
        let short = if self.displayed || best < Some((1, 0)) { 6.5 } else { 11.0 };
        if best.is_none() || width(atoms) < short {
            return self.piece(atoms);
        }
        let first = atoms[1..].iter().position(|atom| atom.cut == best).unwrap() + 1;
        let apart = self.displayed && atoms[0].cut != best;
        if apart && width(&atoms[..first]) < 3.0 {
            atoms[first].cut = None;
            return self.group(atoms);
        }
        self.html += if apart { "<span>" } else { "" };
        self.groups(atoms.chunk_by_mut(|_, next| next.cut != best), if apart { INDENT } else { 0.0 });
        self.html += if apart { "</span>" } else { "" };
    }

    /// Returns how far in the rest of it hangs.
    fn line(&mut self, atoms: &mut [Atom]) -> f64 {
        let best = atoms.iter().filter_map(|atom| atom.cut).min().filter(|it| matches!(it, (0, ARROW | RELATION)));
        let point = atoms.iter().position(|atom| atom.aligned);
        let point = point.or(atoms.iter().position(|atom| atom.cut == best)).unwrap_or(0);
        let hang = atoms[point].left;
        let (top, bottom) = atoms.iter().fold((f64::MAX, f64::MIN), |(a, b), it| (a.min(it.top), b.max(it.bottom)));
        let above = (top - replace(&mut self.bottom, bottom) - LEADING).max(0.0);
        let style = format!("--hang:{};--lead:{};margin-top:{}", css(hang), css(hang - atoms[0].left), css(above));
        write!(self.html, r#"<span style="{style}">"#).unwrap();
        self.groups(<[_; 2]>::from(atoms.split_at_mut(point)).into_iter().filter(|side| !side.is_empty()), 0.0);
        self.html += "</span>";
        hang
    }
}

/// It takes the stylesheet to make sense of this, and to show it.
pub fn draw<'a>(frame: &Frame, em: Abs, displayed: bool, links: Tracked<'a, LateLinkResolver<'a>>) -> String {
    let mut scribe = Scribe { em, displayed, links, indent: 0.0, reach: 0.0, bottom: f64::MAX, html: String::new() };
    let (mut lines, wide) = (lines(frame, em, displayed), frame.width() / em);
    if !displayed {
        scribe.group(&mut lines[0]);
        return format!(r#"<span hidden aria-hidden="true">{}</span>"#, scribe.html);
    }
    let hang = lines.iter_mut().fold(0.0, |hang: f64, line| hang.max(scribe.line(line)));
    let aligned = lines.iter().all(|line| line[0].left < 0.2 || line.iter().any(|atom| atom.aligned));
    let fold = if aligned { wide.min((4.0 * hang).max(hang + scribe.reach)) } else { wide };
    let least = wide.min(hang.min(INDENT) + scribe.reach) + 0.1;
    let style = format!("--wide:{};--fold:{};--least:{least:.2}", css(wide), css(fold));
    format!(r#"<span hidden aria-hidden="true" style="{style}">{}</span>"#, scribe.html)
}

/// A token of MathML, in characters that screen readers know.
pub fn spoken(text: &str) -> String {
    let accents = "\u{300}\u{301}\u{302}\u{303}\u{304}\u{305}\u{306}\u{307}\u{308}\u{30A}\u{30C}\u{332}\u{20D7}";
    let accent = accents.chars().zip("`´^~¯‾˘˙¨˚ˇ_→".chars()).filter(|(accent, _)| text.starts_with(*accent));
    let italic = ('𝐴'..='𝑧').zip(('A'..='Z').chain('a'..='z'));
    let greek = ('𝛼'..='𝜛').zip(('α'..='ω').chain("∂ϵϑϰϕϱϖ".chars()));
    let plain = accent.chain([('ℎ', 'h'), (' ', '\u{A0}')]).chain(italic).chain(greek);
    text.chars().map(|letter| plain.clone().find(|(from, _)| *from == letter).map_or(letter, |(_, to)| to)).collect()
}

/// Spells out what Typst leaves to a stylesheet, which feeds cannot have.
pub fn arrange(node: &mut Node, displayed: bool) {
    let inside = node.is_displayed();
    node.children.iter_mut().for_each(|part| arrange(part, inside));
    let class = |it: &mut Node| Some(it.attributes.extract_if(.., |(name, _)| name == "class").next()?.1);
    if node.tag != "mtable" {
        return;
    }
    let table = class(node).unwrap_or_default();
    if displayed && table.starts_with("multiline") && node.get("displaystyle").is_none() {
        node.attributes.push(("displaystyle".into(), "true".into()));
    }
    for (column, cell) in node.children.iter_mut().flat_map(|row| row.children.iter_mut().enumerate()) {
        let side = match class(cell).as_ref().unwrap_or(&table).split('-').next() {
            _ if table.ends_with("aligned") => ["right", "left"][column % 2],
            Some("cases" | "left") => "left",
            Some("right") => "right",
            _ => continue,
        };
        let style = format!("text-align:{side};justify-items:{side};padding-{side}:0");
        cell.attributes.extend([("columnalign".into(), side.into()), ("style".into(), style)]);
    }
}

fn class(node: &Node) -> Option<MathClass> {
    default_math_class(node.children.first().filter(|_| node.tag == "mo")?.text.chars().next()?)
}

fn is_single(node: &Node) -> bool {
    let fence = |node| matches!(class(node), Some(MathClass::Opening | MathClass::Fence | MathClass::Closing));
    match &node.children[..] {
        [first, inside @ .., last] => fence(first) && fence(last) && !inside.iter().any(fence),
        only => only.iter().all(is_single),
    }
}

/// Adds the signs that it takes for a formula to mean the same once it is only text.
pub fn punctuate(node: &mut Node) {
    use MathClass::*;
    node.children.iter_mut().filter(|part| part.tag != "math").for_each(punctuate);
    let (signs, scripts): (&[&str], _) = match &*node.tag {
        _ if node.get("accent").or(node.get("accentunder")).is_some() => (&["", ""], false),
        "mfrac" if node.get("linethickness").is_some() => (&["", " choose "], false),
        "mfrac" => (&["", "/"], false),
        "mroot" => (&["", "^(1/"], false),
        "msub" | "munder" | "msubsup" | "munderover" | "mmultiscripts" => (&["", "_", "^", "", " pre_", " pre^"], true),
        "msup" | "mover" => (&["", "^"], true),
        _ => (&[], false),
    };
    let classes: Vec<_> = once(Some(Opening)).chain(node.children.iter().map(class)).chain([Some(Closing)]).collect();
    for ((at, part), [last, own, next]) in node.children.iter_mut().enumerate().zip(classes.array_windows()) {
        let [left, right] =
            [matches!(last, Some(Opening | Fence)), matches!(next, Some(Closing | Fence | Punctuation))]
                .map(|tight| if tight { "" } else { " " });
        let [before, open, close] = match (signs.get(at), own, &*part.tag) {
            (_, _, "mfrac" | "mroot" | "msqrt" | "msup") if scripts && at == 0 => ["", "(", ")"],
            (Some(sign), ..) if !is_single(part) && (at > 0 || !scripts) => [sign, "(", ")"],
            (Some(_), ..) if "′″‴⁗".contains(&plain(&part.children)) => continue,
            (Some(sign), ..) => [sign, "", ""],
            (_, Some(Alphabetic | Punctuation), _) | (.., "msqrt" | "mtd") => ["", "", right],
            _ if part.get("rspace") == Some("0em") || part.tag == "mtr" => continue,
            (_, Some(Binary | Vary), _) if last.is_some() => continue,
            (_, Some(Binary | Vary | Relation), _) | (.., "mspace") => [" ", "", " "],
            (.., "mfrac") => [left, "", if *next == Some(Fence) { " " } else { right }],
            _ => ["", "", if is_single(part) { "" } else { right }],
        };
        part.signs = [[before, open, &part.signs[0]].concat(), [&part.signs[1], close].concat()];
    }
    match &*node.tag {
        "msqrt" => node.signs = if is_single(node) { ["√", ""] } else { ["√(", ")"] }.map(Into::into),
        "mroot" => node.signs[1] += ")",
        "mtable" => (node.children.iter_mut().skip(1))
            .filter_map(|row| row.children.iter_mut().find_map(|cell| cell.children.first_mut()))
            .for_each(|first| first.signs[0].insert_str(0, "; ")),
        _ => {}
    }
}

/// On one line.
pub fn flat(formula: &Node) -> String {
    spoken(&plain(&formula.children)).split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Text that there was no font for.
pub fn missing(frame: &Frame) -> Option<&str> {
    frame.items().find_map(|(_, item)| match item {
        FrameItem::Group(group) => missing(&group.frame),
        FrameItem::Text(text) => text.glyphs.iter().find(|glyph| glyph.id == 0).map(|glyph| &text.text[glyph.range()]),
        _ => None,
    })
}

/// The page, with the shape of each glyph where it is first used.
pub fn gather(page: &str) -> String {
    let (mut parts, mut seen, mut gathered) = (page.split(SHAPE), HashSet::new(), String::new());
    while let Some(rest) = parts.next() {
        gathered += rest;
        if let Some(shape) = parts.next().filter(|shape| seen.insert(*shape)) {
            write!(gathered, r#"<defs><path id="{shape}/></defs>"#).unwrap();
        }
    }
    gathered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typeset::tests::set;

    /// The glyphs in each piece, with brackets around what stays on a line for as long as it can.
    fn sketch(typst: &str) -> String {
        let (blocks, _) = set(&format!("#let x = $x x x x x x$\n${typst}$"));
        let (mut sketch, mut glyphs) = (String::new(), 0);
        for tag in blocks[0].children[0].text.split('<') {
            match tag.split([' ', '>']).next().unwrap() {
                "use" => glyphs += 1,
                "/svg" => sketch += &take(&mut glyphs).to_string(),
                "i" => sketch += " ",
                "span" => sketch += "[",
                "/span" => sketch += "]",
                _ => {}
            }
        }
        sketch
    }

    #[test]
    fn lines_end_where_they_should() {
        for (typst, expected) in [
            ("a = b", "[3]"),
            ("#x = -#x + #x", "[7 8 6]"),
            (" #x = -#x + #x ", "[[6 [8 7]]]"),
            (" a = #x = #x ", "[[1 7 7]]"),
            (r" #x &+ #x \ &= 1 ", "[[6 7][2]]"),
            (" f(#x + #x, #x)^2 ", "[[[[8 8] 8]]]"),
            (" (#x)(#x) quad #x ", "[[[[8 8] 6]]]"),
            (" sqrt(#x + #x) ", "[[14]]"),
            (" cancel(#x) + cancel(#x) ", "[[[6 7]]]"),
            (r" #x + #context $a \ b$ + #x ", "[[16]]"),
            (r#" #x "abcdefghi jklmn 1" "#, "[[[6 [9 6]]]]"),
        ] {
            assert_eq!(sketch(typst), expected, "{typst}");
        }
    }

    #[test]
    fn glyphs_are_drawn_once() {
        let page = gather(&set("$a a b$").0[0].children[0].text);
        assert_eq!(["<use", "<path"].map(|tag| page.matches(tag).count()), [3, 2]);
    }

    #[test]
    fn comments_are_not_published() {
        assert_eq!(set("$a /* b */$").0[0].children[0].get("source"), Some("$a  $"));
    }

    #[test]
    fn formulas_mean_the_same_as_text() {
        for (typst, expected) in [
            ("2 1/9 x_1^2 y' = -a", "2 1/9 x_1^2 y′ = −a"),
            ("(|a| + |b|) / (c d)^(-1)", "(|a| + |b|)/((cd)^(−1))"),
            ("sqrt(2) x sqrt(a b) root(3, c)^2", "√2 x√(ab) (c^(1/3))^2"),
            ("mat(1, 2; 3, 4) binom(n, k)", "(1 2; 3 4)(n choose k)"),
            ("sum_(i = 1)^n sin f(i)", "∑_(i=1)^n sin f(i)"),
        ] {
            assert_eq!(plain(&set(&format!("${typst}$")).0).trim(), expected);
        }
    }

    #[test]
    fn letters_are_spoken_plainly() {
        assert_eq!(["𝑥ℎ𝜕ℝ", "\u{305}", "e\u{301} "].map(spoken), ["xh∂ℝ", "‾", "e\u{301}\u{A0}"]);
    }
}
