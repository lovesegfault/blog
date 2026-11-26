// What posts can use, on top of Typst itself.

// On paper, as in an editor's preview, everything is left plain.
#let web(rule) = it => if target() == "html" { rule(it) } else { it }

// Something for forno to finish.
#let part(name, body, ..options) = html.elem("x-" + name, metadata(options.named()) + body)
#let apart(name, body, ..options) = context web(it => part(name, it, ..options))(body)

// Where it would be lost.
#let noteless(body) = {
  show footnote: _ => panic("a note cannot be here")
  body
}

// Only what has a label gets a number.
#let labelled(pattern) = (..) => for it in query(here()).filter(it => it.has("label")) {
  let kind(other) = (other.has("label"), other.at("kind", default: none), other.at("block", default: none))
  numbering(pattern, query(selector(it.func()).before(here())).filter(other => kind(other) == kind(it)).len())
}

/// The colours of the site, for formulas and drawings.
#let warm = rgb("#a83a10")
#let cool = rgb("#17389c")
#let faint = rgb("#6b5c50")

/// Starts every page: `#show: post.with(title: [A Title])`.
#let post(
  title: none,
  /// As in `"2021-04-01T22:04:19-07:00"`. Only posts have one.
  date: none,
  updated: none,
  categories: (),
  tags: (),
  /// For link previews. The opening paragraph, otherwise.
  description: none,
  /// `false`, or the number of the issue that has them.
  comments: true,
  /// As in `"pt-BR"`.
  lang: "en",
  body,
) = {
  set document(title: title, description: description)
  set text(font: "Spectral", lang: lang.split("-").first(), region: lang.split("-").at(1, default: none))
  show raw: set text(font: "TX-02") // Berkeley Mono
  set quote(block: true)
  set figure(numbering: labelled("1"))
  set math.equation(numbering: labelled("(1)"), supplement: none)
  set math.mat(row-gap: .4em, column-gap: 1em)
  set math.vec(gap: .4em)
  set math.lr(size: 100% - .2em)

  // Typst also ends a paragraph at a displayed formula, which forno mends.
  show parbreak: it => it + metadata(none)
  show math.equation: it => {
    let font = text.font
    show text: it => {
      set text(font: ("Spectral", font).flatten()) if text.font == font and it.text.contains(regex("[^\d.]"))
      it
    }
    show selector.or(link, ref): set text(cool)
    // What Typst would leave out for screen readers and feeds.
    show math.overline: web(it => math.accent(it.body, "\u{305}"))
    show math.underline: web(it => math.accent(it.body, "\u{332}"))
    show math.cancel: web(it => html.elem("mrow", attrs: (notation: "updiagonalstrike"), it.body))
    show math.frac.where(style: "skewed"): web(it => math.frac(it.num, it.denom, style: "vertical"))
    show hide: web(it => html.elem("mphantom", attrs: (style: "visibility:hidden"), it.body))
    noteless(it)
  }
  show figure.where(kind: raw): set figure(kind: image)
  show figure: it => {
    let bare = if not it.has("label") and it.caption != none { it.caption.body }
    show figure.caption.where(body: bare): caption => web(it => html.figcaption(it))(caption.body)
    it
  }
  show ref: it => if it.element != none and it.element.func() == heading and it.element.numbering == none {
    show footnote: none
    link(it.target, if it.supplement == auto { it.element.body } else { it.supplement })
  } else if it.element != none and not it.element.at("block", default: true) {
    panic("only a formula on display has a number")
  } else { it }
  show footnote: web(it => {
    h(0pt, weak: true)
    box(if type(it.body) == label { part("note", none, again: str(it.body)) } else {
      part("note", block(html.div(it.body + parbreak())), again: none)
    })
  })
  show quote.where(block: true): web(it => block(part("quote", {
    html.blockquote(it.body + parbreak())
    let source = it.attribution
    if source != none { html.figcaption[—~#if type(source) == label { cite(source, form: "prose") } else { source }] }
  })))
  show raw: web(it => if it.block { html.pre(lang: it.at("lang", default: none) + "", it.text) } else {
    html.code(it.text.replace(regex("\s*\n\s*"), " "))
  })
  show table.cell: web(it => {
    let side = if it.align != auto { it.align.x }
    if side in (center, right, end) { html.div(style: "text-align:" + repr(side), it) } else { it }
  })

  let front = noteless(context if target() == "html" { html.div(description) } + std.title())
  apart("front", front, date: date, updated: updated, categories: categories, tags: tags, comments: comments)
  body
}

#let aside(kind, body) = block(apart("aside", body + parbreak(), kind: kind))
#let note = aside.with("Note")
#let tip = aside.with("Tip")
#let warning = aside.with("Warning")
#let caution = aside.with("Caution")
/// A statement set apart.
#let important = aside.with("Important")

/// What `image` takes, and then a caption.
#let picture(..options, caption) = figure(image(..options), caption: caption)
/// Takes over the margin.
#let fullwidth(body) = block(apart("wide", body))
/// Stands in the margin.
#let margin(body) = block(apart("margin", body))
/// Pictures side by side.
#let row(..pictures) = block(apart("row", pictures.pos().join()))
/// A file, or an address on YouTube.
#let video(source, description) = block(apart("video", noteless(description), source: source))
/// One of the programs in `crates/explainers/src/bin`.
#let explainer(name, options: "", description) = block(
  apart("explainer", noteless(description), name: name, options: options),
)

// What takes all the room it is given gets the column.
#let fitted(it) = context if measure(it, width: 32.5em).width > measure(it).width { block(width: 32.5em, it) } else { it }

/// Anything that Typst can draw.
#let drawing(alt: none, body) = context web(it => block(part("drawing", html.frame(fitted(it)), alt: alt)))(
  noteless(body),
)

/// A `dot` block, for Graphviz to lay out.
#let diagram(alt: none, body) = block(apart("drawing", noteless(body), alt: alt))

/// A block of code, or the path of a file. After it, `/ name: …` is a note on the line that ends in `<name>`.
#let code(
  title: none,
  /// For a file whose name does not tell.
  lang: none,
  /// Lines to light up, as in `3` or `"3,5-7"`.
  hl: (),
  /// Phrases to underline.
  mark: (),
  /// Lines to fold away.
  fold: (),
  /// The only lines to show.
  lines: (),
  /// The number of the first line.
  start: 1,
  numbers: auto,
  /// Whether the first column has `+`, `-` or a space.
  diff: false,
  /// What programs print wraps, but for what is lined up.
  wrap: auto,
  /// `true`, or the flags to open Compiler Explorer with.
  run: false,
  ..source,
) = {
  assert(source.named() == (:), message: "unexpected argument: " + source.named().keys().join(", "))
  let file = source.filter(it => type(it) == str)
  let shown(it) = if type(it) != str { it } else {
    raw(read(..file), lang: if lang == none { it.split(".").last() } else { lang }, block: true)
  }
  let ranges(value) = (value,).flatten().map(str).join(",", default: "")
  block(apart(
    "code",
    source.pos().map(shown).join(),
    whole: file.len() > 0,
    title: if title == none and file.len() > 0 { file.at(0).split("/").last() } else { title },
    hl: ranges(hl),
    mark: mark,
    fold: ranges(fold),
    lines: ranges(lines),
    start: start,
    numbers: numbers,
    diff: diff,
    wrap: wrap,
    run: if run == true { "" } else if run != false { run },
  ))
}
