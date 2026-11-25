# null pointer

The source of <https://lovesegfault.com>, and of `forno`, which builds it.

## Writing

In the dev shell (`direnv allow`, or `nix develop`):

```sh
forno new "A Title"    # starts content/drafts/a-title/index.md
forno serve            # previews everything, and follows your edits
forno publish a-title  # moves it to content/posts, dated now
```

Pushing to `master` deploys. Broken links, bad formulas and missing pictures
show up in the preview, and fail the build.

[The specimen](content/drafts/specimen/index.md) uses everything below. It is
at `/posts/specimen/` in the preview.

### Front matter

| Key                  | For                                                 |
| -------------------- | --------------------------------------------------- |
| `title`, `date`      | Every post. Pages outside `posts` have no date.     |
| `categories`, `tags` | Lists of names.                                     |
| `updated`            | A date to show next to the first.                   |
| `description`        | Link previews. The opening paragraph, otherwise.    |
| `comments`           | `false`, or an issue number.                        |
| `[macros]`           | TeX macros, on top of the ones in `site.toml`.      |

Comments are found by the title. To rename a post that has some, set `comments`
to the number of its issue.

### Text

GitHub's Markdown, and:

| Write                                       | To get                       |
| ------------------------------------------- | ---------------------------- |
| `$x$`, `$$x$$`, or a `math` block           | A formula                    |
| `\$`                                        | A dollar sign                |
| `[^note]`                                   | A note in the margin         |
| `> -- Someone` as a last paragraph          | The source of a quotation    |
| `\` at the end of quoted lines              | Verse                        |
| `> [!NOTE]`, `TIP`, `WARNING` or `CAUTION`  | An aside                     |
| `> [!IMPORTANT]`                            | A statement set apart        |
| A `dot` block, with a `title` or not        | A Graphviz diagram           |

### Pictures

A paragraph that starts with `![description](file)` is a figure, and the lines
after it are the caption. Several pictures share a row. `file.jpg#wide` takes
over the margin and `file.jpg#margin` stands in it.

The file can also be a video, a YouTube address, or `explainer:name`. An SVG
that uses `currentColor` follows the theme.

JPEGs and PNGs are published without their location or camera details, and the
`scrub` pre-commit hook keeps those out of the repository too. Videos and other
kinds of picture fail the build until you clean them.

### Code

Options follow the language, as in `rust title="main.rs" hl=3,5-7`.

| Option                   | Does                                            |
| ------------------------ | ----------------------------------------------- |
| `title="…"`              | Names the block                                 |
| `hl=3,5-7`               | Lights lines up                                 |
| `mark="…"`               | Underlines a phrase                             |
| `fold=1-12`              | Folds lines away                                |
| `start=40`               | Counts from there                               |
| `diff`                   | Reads `+`, `-` or a space from the first column |
| `file=a.rs lines=9-30`   | Takes the code from beside the post             |
| `run`, `run="-O2"`       | Links to Compiler Explorer                      |
| `wrap`                   | Wraps long lines                                |
| `numbers`, `nonumbers`   | Overrides the default                           |

A line that ends in `// [^note]` gets that note. `console` blocks tell `$`
commands from their output, and `text` blocks are output alone.

### Explainers

Each file in `crates/explainers/src/bin` is an interactive figure. Start from
`projection.rs`; the preview recompiles it on save.

## Where things are

| Path                | What                                                 |
| ------------------- | ---------------------------------------------------- |
| `site.toml`         | Names, macros and redirects                          |
| `assets`            | The stylesheet, the script and the drawing           |
| `static`            | Files published as they are                          |
| `crates/forno`      | The generator                                        |
| `crates/mural`      | Fires `assets/valley.svg` into tiles                 |
| `crates/collision`  | Turns writing into particle tracks                   |
| `nix/vendor.nix`    | Graphviz, KaTeX and the open fonts                   |

`nix build` makes the site. `.cache` can be deleted at any time.

Berkeley Mono is a commercial font by U.S. Graphics Company. It is here for
this site, and is not free to reuse.
