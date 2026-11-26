# Maintaining the blog

@README.md has how to write and where things are. This has how to change them.

## Commands

Tools come from Nix, and from nowhere else. In the dev shell:

```sh
forno serve                 # http://127.0.0.1:1313, to be restarted after changing forno or the mural
forno build --drafts        # everything, into `public`
cargo test --workspace
pre-commit run --all-files  # the formatters, clippy and scrub
nix build                   # what is deployed, from the files that git knows of
nix flake check             # the same, with the specimen
```

Pull requests need `build` and `lint` to pass. The flake's inputs are updated
every month by pull requests that merge themselves.

## What must not break

- **Comments.** utterances finds a thread by the `<title>` of the page, which is
  `<post> :: null pointer`. Leave alone the format in `pages.rs`, `title` in
  `site.toml`, and how `Page::title` is made.
- **Addresses.** `/posts/<slug>/`, the files beside a post, the ids of headings
  (`slugify` names them as Hugo did) and of notes (`fn:1`, `fnref:1`), the feeds
  and their `guid`s. A page that moves gets a redirect in `site.toml`.
- **Privacy.** Files go through `scrub.rs`, whether they are copied or read
  by Typst. Drafts are not even in the source that `nix build` sees.
- **The writing.** The words in posts are the author's, slips included. Mend the
  markup, and ask before touching the prose.
- **The font.** Berkeley Mono is licensed. The copy in `assets/fonts` has the
  licensee's id taken out of its name table, and so must any that replaces it.
- **Reading without help.** Pages read with no script, in reader views, in
  print, in feeds and with a screen reader, from 320px up, by day and by night.
- **No silent loss.** What would be dropped or misread fails the build, with a
  file and a line (`Build::report`, `misread`, `noteless`, `links::check`).

## How a page is made

1. `typeset.rs` runs Typst as a library, for HTML, with its own rules for
   formulas, pictures and footnotes. `blog.typ` leaves `x-…` elements for forno.
2. `tree.rs` turns the result into `Node`s, and mends paragraphs, ids and quotes.
3. `article.rs` writes those twice: for the page, and for feeds.
4. `pages.rs` puts the site around them, and `site.rs` checks the whole.

A formula is laid out as on paper, cut where a line may end, and drawn in pieces
that the stylesheet fits to the screen. Hidden MathML beside it is for screen
readers and feeds, and `formula::flat` is for where there is only text.

What is slow to make is kept in `.cache`, under a digest of its inputs, of the
file that makes it and of `Cargo.lock`. A new input goes into the digest.
Changing `photo.rs` or `crates/mural` fires every tile again.

These are written in two places, and change together:

| What                     | Where                                                   |
| ------------------------ | ------------------------------------------------------- |
| The widths of the column | `site.css`, `article.rs` and `fitted` in `blog.typ`     |
| The named colours        | `site.css`, `blog.typ` and `paint` in `style.rs`        |
| Indents in formulas      | `site.css` and `INDENT` in `formula.rs`                 |
| How tracks are drawn     | `site.js` and `tracks` in `figure.rs`                   |
| Talking to programs      | `site.js`, `programs.rs` and the `extern "C"` functions |
| What the site is made of | `SOURCES` in `site.rs` and `site` in `flake.nix`        |
| What it is made with     | `Tools::new` in `site.rs` and `env` in `flake.nix`      |
| The names of font files  | `nix/fonts.nix`, `style.rs` and `figure.rs`             |
| What posts can use       | `blog.typ`, the README and the specimen                 |

## The stylesheet and the script

There is no build step but `style.rs`, and no library. Colours are written with
`light-dark(day, night)`: once per declaration, with plain values and outside
`@media`, so that `style.rs` can repeat them for older browsers. Those from 2023
on have to get the text and the formulas right, so nesting is out, and `:has()`
is for looks only. Text keeps a contrast of 4.5:1 in both themes.

## Checking a change

The tests cover the generator, and nothing that is seen. For that, go through
`/posts/specimen/` and `/posts/notes-on-euclidean-spaces/` in the preview:

- at 320, 390, 800, 1240 and 1440px, where nothing should spill sideways;
- in both themes, and with the script off;
- in Chrome, Firefox and Safari, which select and copy formulas differently;
- with the keyboard, into a note and back.

After a change to the generator, compare `public` from before and after: titles,
ids and `index.xml` should only differ where that was the point.

## Updating

- **Typst.** `formula.rs`, `tree.rs` and `typeset.rs` use its internals, so read
  the release notes on HTML export, and drop what it has learned to do itself.
  The `typst*` crates move together, along with `ttf-parser` and
  `unicode-math-class`, whose types cross over and which Dependabot leaves
  alone. Check the formulas by eye.
- **A language.** Its `arborium-*` crate in `crates/forno/Cargo.toml`, and a line
  in `highlight.rs`.
- **A Typst package or a font.** `typst` in `flake.nix`, or `nix/fonts.nix`.
- **Large files.** `.gitattributes` sends pictures, videos and whatever is under
  `images` or `files` to LFS. `assets` stays out of it.

## Known limits

- A counter or a label inside a box in a formula is counted twice.
- Hidden files, such as `static/.well-known`, are not published.
- `lang` goes for a whole post. `html.span(lang: …)` marks a phrase.

## Style

Code is short. A comment says what the code cannot, in a line, and never what it
used to do. Messages and documents are in plain words. Commits follow
Conventional Commits, with `content(<page>)` for the writing.
