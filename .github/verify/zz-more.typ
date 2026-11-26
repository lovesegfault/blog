#import "/blog.typ": *
#show: post.with(title: [More], date: "2020-01-01", comments: false)

One#footnote[First of all.] and two#footnote[Second, with its own.#footnote[Third, inside.]] and $#text(red)[$x$] + y$.

#drawing(alt: "A box", rect(fill: aqua, stroke: red)[a label])

#image("/static/themed.svg", alt: "A line", width: 50%)

#image("/static/lines.svg", alt: "A black line", width: 50%)

#diagram(alt: "Two nodes")[
  ```dot
  digraph { rankdir=LR; a -> b [label="to"]; b [style=filled, fillcolor=pink] }
  ```
]

+ [x] done
+ [ ] not done
+ neither

- [x] done
- neither

#lorem(300)
