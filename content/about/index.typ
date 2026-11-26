#import "/blog.typ": *

#show: post.with(title: [About])

#let ist = link.with(
  "https://en.wikipedia.org/wiki/Instituto_Superior_T%C3%A9cnico",
)
#let cv = link.with(
  "https://github.com/lovesegfault/cv/releases/latest/download/cv.pdf",
)

#margin(image(
  "me.jpg",
  alt: "Me in a rust-colored jacket, looking down into the finder of a medium format camera",
))

I'm Bernardo, a software engineer at Anthropic, where I work on RL
infrastructure. I'm also a NixOS maintainer.

Previously, I worked at AWS on the Rust build infrastructure and developer
experience, and at Google Quantum AI on the QKernel, the control stack for the
quantum computer, and QCC, the quantum circuit compiler.

I was born in Brazil, and studied Computer Engineering at #ist[IST] in Lisbon,
Portugal before dropping out. I now live in New York City, and you can reach me
at #link("mailto:bernardo@meurer.org").

You can read more about me in my #cv[CV].

#divider()

Here are, for curiosity's sake, some other things I enjoy:

- Problems
  - Music archival
  - Safe cross-process memory sharing
  - Reproducible builds

- Music
  - Hermeto Pascoal
  - Ravel, Maurice
  - King Gizzard and The Lizard Wizard
  - Caetano Veloso
  - Talking Heads
  - Death Grips

- Projects
  - Rust
  - Nix
  - Gentoo
  - Neovim
  - bcachefs

- Movies
  - The Seventh Seal
  - The Great Beauty
  - Apocalypse Now
  - Wild Strawberries

- Books
  - The Tale of The Unknown Island - José Saramago
  - Discipline and Punish - Michel Foucault
  - The Stranger - Albert Camus
  - The World As Will and Representation - Arthur Schopenhauer
  - In Defense of Lost Causes - Slavoj Žižek
