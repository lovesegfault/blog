# null pointer

The source of <https://lovesegfault.com>, and of `forno`, which builds it.

## Writing

In the dev shell (`direnv allow`, or `nix develop`):

```sh
forno new "A Title"    # starts content/drafts/a-title/index.typ
forno serve            # previews everything, and follows your edits
forno publish a-title  # moves it to content/posts, dated now
```

Pushing to `master` deploys. Mistakes, broken links and whatever the web cannot
show come up in the preview, and fail the build.

Posts are [Typst](https://typst.app/docs/reference/syntax/), with the functions
in [`blog.typ`](blog.typ). [The specimen](content/drafts/specimen/index.typ)
uses everything below, and is at `/posts/specimen/` in the preview. The shell
has `tinymist` for your editor.

Comments are found by the title. To rename a post that has some, set `comments`
to the number of its issue.

### Text

| Write                                         | To get                      |
| --------------------------------------------- | --------------------------- |
| `#footnote[…]`                                | A note in the margin        |
| `#quote(attribution: […])[…]`                 | A quotation                 |
| `\` at the end of quoted lines                | Verse                       |
| `#note[…]`, `#tip`, `#warning` or `#caution`  | An aside                    |
| `#important[…]`                               | A statement set apart       |
| `$x$` or `$ x $`                              | A formula                   |
| `#math.equation(alt: "…", $…$)`               | One that has a drawing      |
| `<label>` after a formula or a figure         | A number, for `@label`      |
| `- [x]` or `- [ ]`                            | A task                      |
| `#divider()`                                  | A break                     |

### Pictures

| Write                                  | To get                             |
| -------------------------------------- | ---------------------------------- |
| `#image("a.jpg", alt: "…")`            | A picture                          |
| `#picture("a.jpg", alt: "…")[…]`       | One with a caption                 |
| `#row(image(…), image(…))`             | Some side by side                  |
| `#video("a.mp4")[…]`                   | A video, which can be on YouTube   |
| A `dot` block                          | A Graphviz diagram                 |
| `#drawing(alt: "…", …)`                | What Typst or CeTZ can draw        |
| `#explainer("name")[…]`                | An interactive figure              |
| `#figure(…, caption: […])`             | A caption for any of them          |
| `#margin(…)` or `#fullwidth(…)`        | It in the margin, or across it     |

An SVG that uses `currentColor` follows the theme.

JPEGs and PNGs are published without their location or camera details, and the
`scrub` pre-commit hook keeps those out of the repository too. Videos and other
kinds of picture fail the build until you clean them.

### Code

A block needs nothing but its language. `console` blocks tell `$` commands from
their output, and blocks with no language are output alone. `#code(…)[…]` goes
around a block to name it, light up or fold lines, put notes on them, and more;
`#code("a.rs")` shows a file.

### Explainers

Each file in `crates/explainers/src/bin` is an interactive figure. Start from
`projection.rs`; the preview recompiles it on save.

## Where things are

| Path                | What                                                 |
| ------------------- | ---------------------------------------------------- |
| `site.toml`         | Names and redirects                                  |
| `blog.typ`          | What posts import                                    |
| `assets`            | The stylesheet, the script and the drawing           |
| `static`            | Files published as they are                          |
| `crates/forno`      | The generator                                        |
| `crates/mural`      | Fires `assets/valley.svg` into tiles                 |
| `crates/collision`  | Turns writing into particle tracks                   |
| `nix/vendor.nix`    | Graphviz, fonts and Typst packages                   |

`nix build` makes the site. `.cache` can be deleted at any time.

Berkeley Mono is a commercial font by U.S. Graphics Company. It is here for
this site, and is not free to reuse.
