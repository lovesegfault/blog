#import "/blog.typ": *
#import "@preview/cetz:0.5.2"

#show: post.with(
  title: [Specimen],
  date: "2000-01-01",
  updated: "2000-01-02",
  categories: "notes",
  tags: "specimen",
)

#let inner(x, y) = $lr(chevron.l #x, #y chevron.r)$
#let proj = math.op("proj")
#let typst = link.with("https://typst.app/docs/reference/syntax/")

Everything a post can have, to copy from and to check the design against. This
page is a draft on purpose.#footnote[
  Anything under `content/drafts` is only ever seen in the preview.
] Text can be _slanted_, *strong*, #strike[struck], #highlight[marked] or
`code`. It can link to #link("/about/")[a page], to #typst[an address that was
given a name], or to a section such as @pictures.

= Code

A title, a lit line and a marked phrase. Line numbers are links.

#code(title: "sizeof.c", hl: 3, mark: "sizeof(arr)")[
  ```c
  void randomize(int arr[]) {
      // See https://en.cppreference.com/w/c/language/array
      size_t len = sizeof(arr) / sizeof(int);
      for (size_t i = 0; i < len; ++i)
          arr[i] = rand();
  }
  ```
]

Notes on lines, which go in the margin like any other.

#code[
  ```rust
  impl<T> Drop for Unique<T> {
      fn drop(&mut self) {
          // SAFETY: nothing else has this pointer.
          unsafe {
              self.ptr.drop_in_place(); // <drop>
              dealloc(self.ptr.cast(), Layout::new::<T>()); // <free>
          }
      }
  }
  ```
  / drop: This runs the destructor, but leaves the memory.
  / free: And this gives the memory back. The layout has to be the one it was
    allocated with.
]

What changed, still coloured as the language it is in.

#code(title: "flake.nix", start: 12, diff: true)[
  ```nix
   devShells.default = pkgs.mkShell {
     packages = with pkgs; [
  -    hugo
  +    cargo
  +    rustc
     ];
     shellHook = ''
       echo "hello from ${system}"
     '';
   };
  ```
]

Part of a file that sits beside the post, with the dull part folded away.

#code("ring.rs", lines: "9-37", fold: "10-21")

A whole program, which readers can open on Compiler Explorer.

#code(run: "-O2 -Wall")[
  ```c
  #include <stdio.h>

  int main(void) {
      printf("%p não aponta para lugar nenhum\n", (void *)0);
  }
  ```
]

A session. Copying it takes the commands only.

```console
$ cc -O2 sizeof.c \
    -o sizeof
$ ./sizeof | head -n 2
(main) len = 10
(randomize) len = 2
```

What a program printed.

```
sizeof(ptr) = 8
sizeof(arr) = 20
```

Long lines take over the margin.

```rust
fn lookup<'a>(table: &'a SymbolTable, name: &str, version: Option<&Version>) -> Option<&'a Symbol> {
    table.buckets[gnu_hash(name) as usize % table.buckets.len()].iter().find(|s| s.is(name, version))
}
```

```diff
--- a/build.rs
+++ b/build.rs
@@ -3,2 +3,3 @@
 let mut args = Vec::new();
+args.push("-Clink-args=-Wl,--disable-new-dtags");
```

#code(numbers: false)[
  ```asm
  _start:
      mov  x0, #0     // the null pointer
      ldr  x1, [x0]
  ```
]

= Quotations

#quote(attribution: [attributed to Edsger Dijkstra])[
  Computer science is no more about computers than astronomy is about
  telescopes.
]

A longer one is set at the size of the text.

#quote(attribution: [ISO C, 6.3.2.3, item 3])[
  An integer constant expression with the value 0, or such an expression cast to
  type `void *`, is called a _null pointer constant_. If a null pointer constant
  is converted to a pointer type, the resulting pointer, called a _null
  pointer_, is *guaranteed to compare unequal to a pointer to any object or
  function*.
]

Lines that end in a backslash are verse.

#quote(attribution: [Gonçalves Dias, _Canção do Exílio_])[
  Minha terra tem palmeiras, \
  Onde canta o Sabiá; \
  As aves, que aqui gorjeiam, \
  Não gorjeiam como lá.
]

A reply.

#quote[
  #quote[Arrays are just pointers.]

  They are not, and the standard says so in several places. It is worth going
  through them one at a time, because each is surprising in a way of its own,
  and none of them is hard.
]

== Asides

#note[Something to keep in mind, in your own words. There is also `#tip`.]

#warning[Something that bites. There is also `#caution`.]

#important[The one thing to take away, which needs no label.]

= Mathematics

For $u in RR^n$ and a subspace $W$, the closest point to $u$ is its
projection#footnote[If the basis is orthonormal, $norm(b_j) = 1$.] <norm>
$ proj_W u = sum_(j = 1)^k inner(u, b_j) / norm(b_j)^2 b_j. $ <projection>
A formula with a label gets a number, so @projection can be referred to, and so
can a note.@norm

$
  d(u, x) & = sqrt(norm(u - proj_W u)^2 + norm(proj_W u - x)^2) \
          & >= underbrace(norm(u - proj_W u), d(u, W))
$

One that is too long for the screen is broken into lines.
$
  (a + b)^6 = a^6 + 6 a^5 b + 15 a^4 b^2 + #text(warm, $20 a^3 b^3$) + 15 a^2 b^4
  + 6 a b^5 + b^6
$

TeX is there for what Typst has no notation for.

#tex(```
\begin{prooftree}
  \AxiomC{$\Gamma, x : \sigma \vdash e : \tau$}
  \RightLabel{abs}
  \UnaryInfC{$\Gamma \vdash \lambda x.\, e : \sigma \to \tau$}
\end{prooftree}
```)

#figure(
  explainer("projection", options: "16")[A vector, and its shadow on a line],
  caption: [Drag the tip of $u$.],
)

= Pictures <pictures>

#picture(
  "../../posts/home/images/entrance.jpg",
  alt: "A stone path across a lawn",
)[The caption can have _any_ `markup`.]

#figure(
  row(
    image("../../posts/home/images/flowers.jpg", alt: "A baby among flowers"),
    image("../../posts/home/images/hall.jpg", alt: "A hallway"),
    image("../../posts/home/images/horse.jpg", alt: "A child on a horse"),
  ),
  caption: [Pictures can share a row.],
)

#margin(picture(
  "../../posts/home/images/chickens.jpg",
  alt: "A toddler among geese",
)[In the margin.])

A picture can also stand in the margin, beside the text that talks about it, or
take the margin over. On a narrow screen both go back into the column.

#fullwidth(picture(
  "../../posts/the-lost-ark/images/turntable.jpg",
  alt: "A turntable on a shelf of records",
)[Wide.])

Graphviz lays out @lookup, and the one after it is drawn by Typst.

#figure(caption: [How a symbol is found])[
  ```dot
  digraph {
    rankdir=LR
    call [label="call puts@plt"]
    plt [label=".plt"]
    got [label=".got.plt", class=warm]
    ld [label="ld.so", style="rounded,filled"]
    call -> plt -> got
    got -> ld [label="once", style=dashed]
    got -> puts [label="after"]
    ld -> puts
  }
  ```
] <lookup>

#drawing(alt: "An angle in the unit circle", cetz.canvas(length: 4em, {
  import cetz.draw: *
  circle((0, 0), stroke: faint + .5pt)
  line((-1.3, 0), (1.3, 0), stroke: faint + .5pt)
  line((0, 0), (40deg, 1), name: "ray")
  line("ray.end", ("ray.end", "|-", (0, 0)), stroke: warm, name: "sine")
  line((0, 0), "sine.end", stroke: cool, name: "cosine")
  content((1.1, .3), anchor: "west", text(warm, $sin theta$))
  content("cosine.mid", anchor: "north", padding: .1, text(cool, $cos theta$))
}))

= The rest

#table(
  columns: 3,
  align: (left, right, left),
  table.header[Type][`sizeof`][Decays to],
  [`int[5]`], [20], [`int *`],
  [`int *`], [8], [itself],
)

- A list
  - inside a list
    - inside a list
- [x] Done
- [ ] Not done

+ First
+ Second

/ Provenance: Where a pointer came from.

#divider()

== A smaller heading

=== And the smallest

The end.
