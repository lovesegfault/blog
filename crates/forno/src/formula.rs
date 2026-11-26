//! Typst's formulas as MathML. What only MathJax needs to know is in `data-` attributes.

use std::ops::ControlFlow;

use maud::html;
use typst::diag::{SourceResult, bail};
use typst::ecow::EcoString;
use typst::engine::Engine;
use typst::foundations::{Content, NativeElement, Packed, ShowFn, StyleChain};
use typst::introspection::{Counter, Locator, TagElem};
use typst::layout::{Abs, Axis, FixedAlignment, HideElem, Rel};
use typst::math::ir::{
    AccentItem, AlignedRow, GlyphItem, MathComponent, MathItem, MathKind, Position, ScriptsItem, TableItem,
    resolve_equation,
};
use typst::math::{AugmentOffsets, EquationElem, LeftRightAlternator, MathSize};
use typst::routines::Arenas;
use typst::syntax::Span;
use typst::text::TextElem;
use typst::visualize::{Color, Paint};
use typst_html::{HtmlAttr, HtmlElem, tag};
use unicode_math_class::MathClass;

use crate::typeset::is_written;

pub const RULE: ShowFn<EquationElem> = |equation, engine, styles| {
    let (location, block) = (equation.location().unwrap(), equation.block.get(styles));
    let number = match equation.numbering.get_ref(styles) {
        Some(numbering) if block => {
            Counter::of(EquationElem::ELEM).display_at(engine, location, styles, numbering, equation.span())?
        }
        _ => Content::empty(),
    };
    let mut scribe = Scribe { engine, tags: Vec::new() };
    let around = Around { size: if block { MathSize::Display } else { MathSize::Text }, ink: &Color::BLACK };
    let inside = scribe.nested(&equation.body, styles, Locator::synthesize(location), around)?;
    let alt = equation.alt.get_cloned(styles);
    let math = html! { math display=[block.then_some("block")] alttext=[&alt] aria-label=[&alt] {} }.0;
    // An apostrophe after this is not taken for an opening quote.
    scribe.tags.push(TextElem::packed('\u{FFFC}'));
    Ok(HtmlElem::new(tag::mathml::math)
        .with_attr(HtmlAttr::constant("mathml"), math.replacen("><", &format!(">{inside}<"), 1))
        .with_attr(HtmlAttr::constant("number"), number.plain_text())
        .with_attr(HtmlAttr::constant("text"), equation.body.plain_text())
        .with_body(Some(Content::sequence(scribe.tags)))
        .pack()
        .spanned(equation.body.traverse(&mut written).break_value().unwrap_or(Span::detached())))
};

fn written(part: Content) -> ControlFlow<Span> {
    if is_written(part.span().id()) { ControlFlow::Break(part.span()) } else { ControlFlow::Continue(()) }
}

struct Scribe<'a, 'e> {
    engine: &'a mut Engine<'e>,
    /// What Typst has to find in the page again.
    tags: Vec<Content>,
}

/// What an element inherits.
#[derive(Clone, Copy)]
struct Around<'a> {
    size: MathSize,
    ink: &'a Color,
}

impl Around<'_> {
    fn smaller(self, steps: usize) -> Self {
        const SIZES: [MathSize; 4] = [MathSize::ScriptScript, MathSize::Script, MathSize::Text, MathSize::Display];
        Self { size: SIZES[(self.size as usize).saturating_sub(steps)], ..self }
    }

    fn script(self) -> Self {
        self.smaller(if self.size == MathSize::Display { 2 } else { 1 })
    }
}

fn element(name: &str, attributes: &str, inside: &str) -> String {
    format!("<{name}{attributes}>{inside}</{name}>")
}

fn token(name: &str, attributes: &str, text: &str) -> String {
    element(name, attributes, &html!((text)).0)
}

fn sized(size: MathSize, inside: &str) -> String {
    let level = 2_usize.saturating_sub(size as usize);
    element("mstyle", &format!(r#" displaystyle="{}" scriptlevel="{level}""#, size == MathSize::Display), inside)
}

fn class(class: MathClass) -> String {
    let name = match class {
        MathClass::Large => "OP",
        MathClass::Binary => "BIN",
        MathClass::Relation => "REL",
        MathClass::Opening => "OPEN",
        MathClass::Closing => "CLOSE",
        MathClass::Punctuation => "PUNCT",
        _ => "ORD",
    };
    format!(r#" data-mjx-texclass="{name}""#)
}

fn component<'i, 'a>(item: &'i MathItem<'a>) -> Option<&'i MathComponent<'a>> {
    if let MathItem::Component(component) = item { Some(component) } else { None }
}

fn glyph<'i>(item: &'i MathItem) -> Option<&'i GlyphItem> {
    if let MathKind::Glyph(glyph) = &component(item)?.kind { Some(glyph) } else { None }
}

/// As `sin` and `lim` are.
fn is_function(item: &MathItem) -> bool {
    component(item).is_some_and(|component| match &component.kind {
        MathKind::Scripts(scripts) => is_function(&scripts.base),
        kind => matches!(kind, MathKind::Text(_)) && component.props.class() == MathClass::Large,
    })
}

/// The letter that a styled one was made from, and its style.
fn unstyle(styled: char) -> Option<(char, &'static str)> {
    const STYLES: &str = "bold italic bold-italic . . . . . sans-serif bold-sans-serif sans-serif-italic \
                          sans-serif-bold-italic monospace";
    let from = |first| (styled as u32 - first as u32) as usize;
    match styled {
        'ℎ' => Some(('h', "italic")),
        '𝐀'..='𝚣' => ('A'..='Z').chain('a'..='z').nth(from('𝐀') % 52).zip(STYLES.split(' ').nth(from('𝐀') / 52)),
        '𝟎'..='𝟿' => ('0'..='9').nth(from('𝟎') % 10).zip(STYLES.split(' ').nth([0, 3, 8, 9, 12][from('𝟎') / 10])),
        '𝛼'..='𝜛' => ('α'..='ω').chain("∂ϵϑϰϕϱϖ".chars()).nth(from('𝛼')).zip(Some("italic")),
        _ => None,
    }
    .filter(|(_, style)| *style != ".")
}

/// In the fonts of the page.
fn prose(style: &str, text: &str) -> String {
    let size = if style == "monospace" { r#" mathsize=".8em""# } else { "" };
    token("mtext", &format!(r#" mathvariant="{style}"{size}"#), text)
}

fn words(text: &str, function: bool) -> String {
    let style = text.chars().find_map(unstyle).map_or("normal", |(_, style)| style);
    let plain: String = text.chars().map(|letter| unstyle(letter).map_or(letter, |(plain, _)| plain)).collect();
    let [before, after] = [plain.trim_start(), plain.trim_end()].map(|rest| "\u{a0}".repeat(plain.len() - rest.len()));
    let text = format!("{before}{}{after}", plain.trim());
    if function || style.contains("sans") || text.contains(|letter| matches!(letter, 'ℂ'..='ⅉ' | '𝐀'..='𝟿')) {
        let class = class(if function { MathClass::Large } else { MathClass::Normal });
        return token("mi", &format!(r#"{class} mathvariant="{style}""#), &text);
    }
    prose(style, &text)
}

fn symbol(glyph: &GlyphItem, MathComponent { props, styles, .. }: &MathComponent) -> String {
    let (text, first) = (glyph.text.as_str(), glyph.text.chars().next().unwrap());
    let stretch = [Axis::X, Axis::Y].into_iter().find(|axis| glyph.stretch.get().is_explicit(*axis));
    let mut attributes = match props.class() {
        MathClass::Normal if "′″‴⁗".contains(first) && props.size < MathSize::Text => {
            r#" data-mjx-alternate="1""#.into()
        }
        MathClass::Normal if "!./′″‴⁗…⋮⋯⋰⋱".contains(first) => String::new(),
        MathClass::Normal | MathClass::Alphabetic | MathClass::Special if stretch.is_none() => {
            return match unstyle(first) {
                Some((plain, "italic")) => token("mi", "", &text.replace(first, &plain.to_string())),
                Some((_, "monospace")) => words(text, false),
                _ if text == "∅" => token("mi", r#" data-mjx-alternate="1""#, text),
                _ => token("mi", r#" mathvariant="normal""#, text),
            };
        }
        // The slash of a fraction.
        MathClass::Binary if first == '/' => String::new(),
        other => class(other),
    };
    if let Some(axis) = stretch {
        attributes += r#" stretchy="true" symmetric="true""#;
        if let Some(size) = glyph.stretch.get().resolve_requested(axis) {
            let em = styles.resolve(TextElem::size);
            let least = format!(r#" minsize="{:.3}em""#, size.rel.get() + size.abs.at(em) / em);
            attributes += &if size.rel.is_zero() { least.replace("min", "max") + &least } else { least };
            if axis == Axis::X {
                return element("mover", "", &(token("mo", &attributes, text) + "<mrow></mrow>"));
            }
        }
    } else if matches!(glyph.class, MathClass::Opening | MathClass::Closing | MathClass::Fence)
        || matches!(first, '←'..='⇿' | '⟰'..='⟿' | '⤀'..='⥿')
    {
        attributes += r#" stretchy="false""#;
    } else if glyph.class == MathClass::Large {
        attributes += r#" movablelimits="false""#;
    }
    token("mo", &attributes, &text.replace(' ', "\u{a0}"))
}

impl Scribe<'_, '_> {
    fn nested(&mut self, body: &Content, styles: StyleChain, locator: Locator, around: Around) -> SourceResult<String> {
        let (arenas, equation) = (Arenas::default(), Packed::new(EquationElem::new(body.clone())));
        let item = resolve_equation(&equation, self.engine, locator, &arenas, styles)?;
        self.row(&item, around)
    }

    /// Any number of elements.
    fn row(&mut self, item: &MathItem, around: Around) -> SourceResult<String> {
        let items = item.as_slice();
        let mut row = String::new();
        for (at, item) in items.iter().enumerate() {
            let before = items[..at].last().and_then(component).map_or(MathClass::Normal, |it| it.props.class());
            row += &match item {
                MathItem::Component(component) => self.component(item, component, around)?,
                MathItem::Space => r#"<mspace width=".3em"></mspace>"#.into(),
                MathItem::Spacing(.., true) if !class(before).contains("ORD") && before != MathClass::Closing => {
                    continue;
                }
                MathItem::Spacing(width, em, _) => {
                    let width = width.at(*em) / *em;
                    let glued = if width < 0.3 { r#" linebreak="nobreak""# } else { "" };
                    format!(r#"<mspace width="{width:.3}em"{glued}></mspace>"#)
                }
                MathItem::Tag(tag) => {
                    self.tags.push(TagElem::packed(tag.clone()));
                    continue;
                }
            };
            if is_function(item) && at + 1 < items.len() {
                row += "<mo>\u{2061}</mo>";
            }
        }
        Ok(row)
    }

    /// Exactly one element.
    fn one(&mut self, item: &MathItem, around: Around) -> SourceResult<String> {
        match item {
            MathItem::Component(component) => self.component(item, component, around),
            _ => Ok(element("mrow", "", &self.row(item, around)?)),
        }
    }

    fn component(&mut self, item: &MathItem, component: &MathComponent, around: Around) -> SourceResult<String> {
        let MathComponent { kind, props, styles } = component;
        let ink = if let Paint::Solid(ink) = styles.get_ref(TextElem::fill) { ink } else { around.ink };
        let here = Around { size: props.size, ink };
        // The class that MathJax works out by itself.
        let mut natural = None;
        let mut base = |scribe: &mut Self, base| {
            natural = self::component(base).and_then(|base| base.props.class);
            scribe.one(base, here)
        };
        let mut written = match kind {
            MathKind::Group(_) => element("mrow", "", &self.row(item, here)?),
            MathKind::Glyph(glyph) => {
                natural = props.class;
                symbol(glyph, component)
            }
            MathKind::Text(text) => {
                natural = Some(if is_function(item) { MathClass::Large } else { MathClass::Alphabetic });
                words(&text.text, is_function(item))
            }
            MathKind::Number(number) if number.text.contains(|digit| digit >= '𝟶') => words(&number.text, false),
            MathKind::Number(number) => token("mn", "", &number.text),
            MathKind::Primes(primes) => token("mo", r#" data-mjx-alternate="1""#, &"′".repeat(primes.count)),
            MathKind::Radical(root) => match &root.index {
                Some(index) => {
                    element("mroot", "", &(self.one(&root.radicand, here)? + &self.one(index, here.smaller(3))?))
                }
                None => element("msqrt", "", &self.row(&root.radicand, here)?),
            },
            MathKind::Fenced(fenced) => {
                let parts = [fenced.open.as_ref(), Some(&*fenced.body), fenced.close.as_ref()];
                let inside: String =
                    parts.iter().flatten().map(|part| self.row(part, here)).collect::<Result<_, _>>()?;
                let attributes = match fenced.body.as_slice() {
                    [MathItem::Component(MathComponent { kind: MathKind::Table(_), .. })] => {
                        r#" data-mjx-texclass="INNER""#
                    }
                    // A line of text can end inside a long one.
                    body if body.len() > 8 => r#" data-semantic-added="true""#,
                    _ => "",
                };
                element("mrow", attributes, &inside)
            }
            MathKind::Fraction(it) => {
                let parts = self.one(&it.numerator, here.smaller(1))? + &self.one(&it.denominator, here.smaller(1))?;
                element("mfrac", if it.line { "" } else { r#" linethickness="0""# }, &parts)
            }
            MathKind::SkewedFraction(it) => {
                let parts = self.one(&it.numerator, here.smaller(1))? + &self.one(&it.denominator, here.smaller(1))?;
                element("mfrac", r#" bevelled="true""#, &parts)
            }
            MathKind::Scripts(scripts) => {
                let base = base(self, &scripts.base)?;
                self.scripts(base, scripts, here.script())?
            }
            MathKind::Accent(accent) => accented(base(self, &accent.base)?, accent),
            MathKind::Line(line) => {
                let name = if line.position == Position::Above { "mover" } else { "munder" };
                element(name, "", &(base(self, &line.base)? + r#"<mo accent="true" stretchy="true">―</mo>"#))
            }
            MathKind::Cancel(cancel) => {
                let up = if cancel.invert_first_line { "" } else { "updiagonalstrike " };
                let down = if cancel.invert_first_line || cancel.cross { "downdiagonalstrike" } else { "" };
                element("menclose", &format!(r#" notation="{up}{down}""#), &base(self, &cancel.base)?)
            }
            MathKind::Multiline(lines) if lines.rows.is_empty() => "<mrow></mrow>".into(),
            MathKind::Multiline(lines) => {
                let display = here.size == MathSize::Display;
                let align = match lines.rows[0].len() {
                    1 if here.size == MathSize::Text => "left".into(),
                    1 => "center".into(),
                    columns => "right left ".repeat(columns),
                };
                let apart = if display { ".5em" } else { ".1em" };
                let layout = format!(
                    r#" displaystyle="{display}" columnalign="{align}" columnspacing="0em" rowspacing="{apart}""#
                );
                element("mtable", &layout, &self.cells(lines.rows.iter().map(std::slice::from_ref), here)?)
            }
            MathKind::Table(table) => {
                let cells = self.cells(table.cells.iter().map(Vec::as_slice), here.smaller(1))?;
                sized(here.smaller(1).size, &element("mtable", &layout(table, *styles), &cells))
            }
            MathKind::Box(boxed) => {
                let body = boxed.elem.body.get_cloned(*styles).unwrap_or_default();
                let inside = self.nested(&body, *styles, boxed.locator.relayout(), here)?;
                let framed = boxed.elem.stroke.get_cloned(*styles).iter().any(|side| matches!(side, Some(Some(_))));
                if framed { element("menclose", r#" notation="box""#, &inside) } else { element("mrow", "", &inside) }
            }
            MathKind::Mathml(mathml) => {
                let html = mathml.elem.to_packed::<HtmlElem>().unwrap();
                let pair = |(name, value): &(HtmlAttr, EcoString)| format!(r#" {name}="{}""#, html!((value)).0);
                let attributes: String = html.attrs.get_cloned(*styles).0.iter().map(pair).collect();
                match &mathml.body {
                    Some(body) if !tag::mathml::is_token(html.tag) => {
                        element(&html.tag.resolve(), &attributes, &self.row(body, here)?)
                    }
                    _ => token(&html.tag.resolve(), &attributes, &mathml.elem.plain_text()),
                }
            }
            MathKind::External(other) => {
                let html = other.content.to_packed::<HtmlElem>();
                let body = html.and_then(|html| html.body.get_cloned(*styles)).unwrap_or_default();
                match (html.map(|html| html.tag), other.content.to_packed::<HideElem>()) {
                    (Some(tag::strong), _) => prose("bold", &body.plain_text()),
                    (Some(tag::em), _) => prose("italic", &body.plain_text()),
                    (Some(tag::code), _) => prose("monospace", &body.plain_text()),
                    (Some(tag::a), _) => {
                        let attributes = html.unwrap().attrs.get_cloned(*styles);
                        let href = html!((attributes.get(typst_html::attr::href).map_or("", |href| href))).0;
                        let inside = self.nested(&body, *styles, other.locator.relayout(), here)?;
                        element("mrow", &format!(r#" href="{href}""#), &inside)
                    }
                    (_, Some(hidden)) => {
                        element("mphantom", "", &self.nested(&hidden.body, *styles, other.locator.relayout(), here)?)
                    }
                    _ => bail!(props.span, "this cannot be part of a formula"; hint: "a `drawing` can hold anything"),
                }
            }
        };
        if let Some(given) = props.class.filter(|given| Some(*given) != natural) {
            written = element("mrow", &format!(r#"{} movablelimits="false""#, class(given)), &written);
        }
        if ink != around.ink {
            written = written.replacen('>', &format!(r#" mathcolor="{}">"#, ink.to_hex()), 1);
        }
        Ok(if here.size == around.size { written } else { sized(here.size, &written) })
    }

    fn scripts(&mut self, mut base: String, item: &ScriptsItem, small: Around) -> SourceResult<String> {
        let mut pair = |low: &Option<MathItem>, high: &Option<MathItem>| -> SourceResult<_> {
            let mut write =
                |script: &Option<MathItem>| script.as_ref().map(|script| self.one(script, small)).transpose();
            Ok([write(low)?, write(high)?])
        };
        let attach = |base: String, names: [&str; 3], scripts: [Option<String>; 2]| match scripts {
            [None, None] => base,
            [Some(low), None] => element(names[0], "", &(base + &low)),
            [None, Some(high)] => element(names[1], "", &(base + &high)),
            [Some(low), Some(high)] => element(names[2], "", &(base + &low + &high)),
        };
        let (before, after) = (pair(&item.bottom_left, &item.top_left)?, pair(&item.bottom_right, &item.top_right)?);
        base = if before == [None, None] {
            attach(base, ["msub", "msup", "msubsup"], after)
        } else {
            let [before, after] =
                [before, after].map(|pair| pair.map(|it| it.unwrap_or("<mrow></mrow>".into())).concat());
            element("mmultiscripts", "", &format!("{base}{after}<mprescripts></mprescripts>{before}"))
        };
        // An arrow reaches past its label, and one that is itself a label is not an accent.
        let reaches = glyph(&item.base).is_some_and(|glyph| glyph.stretch.get().is_explicit(Axis::X));
        let room = if reaches { r#" width="+.8em" lspace=".4em""# } else { "" };
        let label = |label: String| element("mpadded", room, &format!("<mi></mi>{label}"));
        let limits = pair(&item.bottom, &item.top)?.map(|limit| limit.map(label));
        Ok(attach(base, ["munder", "mover", "munderover"], limits))
    }

    fn cells<'i>(&mut self, rows: impl Iterator<Item = &'i [AlignedRow<'i>]>, around: Around) -> SourceResult<String> {
        let mut table = String::new();
        for row in rows {
            let mut cells = String::new();
            for cell in row.iter().flat_map(AlignedRow::iter) {
                // An operator is spaced from what is across the alignment point.
                let mut inside = cell.as_slice().iter().filter_map(component).map(|component| component.props);
                let opens = inside.clone().next().is_some_and(|first| first.align_form_infix || first.lspace.is_some());
                let closes = inside.next_back().is_some_and(|last| last.rspace.is_some());
                let [before, after] = [opens, closes].map(|spaced| if spaced { "<mi></mi>" } else { "" });
                cells += &element("mtd", "", &format!("{before}{}{after}", self.row(cell, around)?));
            }
            table += &element("mtr", "", &cells);
        }
        Ok(table)
    }
}

fn accented(mut base: String, accent: &AccentItem) -> String {
    const SPACING: [(char, char); 13] = [
        ('\u{300}', '`'),
        ('\u{301}', '´'),
        ('\u{302}', '^'),
        ('\u{303}', '~'),
        ('\u{304}', '¯'),
        ('\u{305}', '―'),
        ('\u{306}', '˘'),
        ('\u{307}', '˙'),
        ('\u{308}', '¨'),
        ('\u{30A}', '˚'),
        ('\u{30B}', '˝'),
        ('\u{30C}', 'ˇ'),
        ('\u{332}', '―'),
    ];
    let mark = glyph(&accent.accent).and_then(|glyph| glyph.text.chars().next()).unwrap_or(' ');
    let mark = SPACING.iter().find(|(combining, _)| *combining == mark).map_or(mark, |(_, spacing)| *spacing);
    let name = if accent.position == Position::Above { "mover" } else { "munder" };
    let narrow = glyph(&accent.base).is_some() && !accent.exact_frame_width && mark != '―';
    if accent.dotless && narrow && name == "mover" {
        base = base.replace(">i<", ">ı<").replace(">j<", ">ȷ<");
    }
    element(name, "", &(base + &token("mo", &format!(r#" accent="true" stretchy="{}""#, !narrow), &mark.to_string())))
}

fn layout(table: &TableItem, styles: StyleChain) -> String {
    let (em, cases) = (styles.resolve(TextElem::size), table.alternator == LeftRightAlternator::None);
    let gap = |gap: Rel<Abs>| format!("{:.3}em ", gap.abs / em);
    let line = |lines: fn(_) -> _, at: usize, of: usize| {
        let drawn =
            |AugmentOffsets(lines): &_| lines.iter().any(|line: &isize| line.rem_euclid(of as isize) == at as isize);
        if table.augment.as_ref().map(lines).is_some_and(drawn) { "solid " } else { "none " }
    };
    let (rows, columns) = (table.cells.len(), table.cells.first().map_or(&[][..], Vec::as_slice));
    let [mut align, mut spacing, mut lines] = [const { String::new() }; 3];
    for (column, parts) in columns.iter().map(AlignedRow::len).enumerate() {
        for part in 0..parts {
            align += match table.align {
                _ if parts > 1 && !cases => ["right ", "left "][part % 2],
                FixedAlignment::Start => "left ",
                FixedAlignment::Center => "center ",
                FixedAlignment::End => "right ",
            };
            spacing += &gap(if part + 1 == parts { table.gap.x } else { (em * 0.7 * f64::from(cases)).into() });
            lines += if part + 1 < parts { "none " } else { line(|augment| &augment.vline, column + 1, columns.len()) };
        }
    }
    let across: String = (1..=rows).map(|row| line(|augment| &augment.hline, row, rows)).collect();
    let drawn =
        table.augment.as_ref().map_or(String::new(), |_| format!(r#" columnlines="{lines}" rowlines="{across}""#));
    format!(r#" columnalign="{align}" columnspacing="{spacing}" rowspacing="{}"{drawn}"#, gap(table.gap.y))
}

#[cfg(test)]
mod tests {
    use crate::typeset::tests::set;

    #[test]
    fn notation_keeps_its_meaning() {
        for (typst, mathml) in [
            ("x_1^2", "<msubsup><mi>x</mi><mn>1</mn><mn>2</mn></msubsup>"),
            ("(a b)/c", "<mfrac><mrow><mi>a</mi><mi>b</mi></mrow><mi>c</mi></mfrac>"),
            ("-x < RR", r#"ORD">−</mo><mi>x</mi><mo data-mjx-texclass="REL">&lt;</mo><mi mathvariant="normal">ℝ"#),
            ("sin x", "sin</mi><mo>\u{2061}</mo><mi>x</mi>"),
            (r#"""_n bold("if ")"#, "<msub><mrow></mrow><mi>n</mi></msub>"),
            (r#"bold("if ")"#, "<mtext mathvariant=\"bold\">if\u{a0}</mtext>"),
            (" sum_i ", "<munder>"),
            ("sum_i", "<msub>"),
            (r" a &= b \ &= c ", "<mtr><mtd></mtd><mtd><mi></mi><mo"),
            ("hat(i)", r#"<mover><mi>ı</mi><mo accent="true" stretchy="false">^</mo></mover>"#),
        ] {
            let (blocks, _) = set(&format!("${typst}$"));
            let written = blocks[0].children[0].get("mathml").unwrap();
            assert!(written.contains(mathml), "{written}");
        }
    }
}
