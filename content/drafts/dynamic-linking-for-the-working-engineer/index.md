+++
title = "Dynamic Linking for the Working Engineer"
date = 2022-08-18T15:16:03-05:00
tags = [ "linker", "loader", "linking", "loading", "elf" ]
categories = [ "essay" ]
draft = true
+++

## TOC

FIXME

## Introduction

I recently solved an issue at work that involved a one line diff, and about a
page-long comment. The diff looked like this:

```diff
+ args.push("-Clink-args=-Wl,--disable-new-dtags")
```

It took a few days of investigation to figure out, but it did feel pretty good
to be in the "one line change with bible-length comment" part of my career.
Regardless, while working on it, I found that resources on how dynamic linking
works are few and far between. You're essentially forced to choose between the
mountain of information contained in the [ELF specification][elfspec] and
accompanying [Portable Formats Specification][elfpfs], which while very thorough
take a while to digest, or a million breadcrumbs of information spread all over
the internet, none of which quite tell you what you want to know, and a few of
which are wrong. What I want to provide here, then, is a resource for the
working software engineer to understand and solve issues related to dynamic
linking and loading.

One issue with writing an article such as this one is that engineers of
different levels of expertise often have conflicting needs from their resources.
While a beginner might need a lot of background to understand something, a
more seasoned software developer is likely interested in a more specifically
relevant bit of information. I don't think there's a fundamentally write way to
resolve this tension, but I think it's possible to alleviate it. The strategy
I've chosen for that is to split major topics into three categories: FIXME,
FIXME, and FIXME. You're encouraged to use this page as a working resource, and
hopefully those help you find what you're looking for more easily.

## Scope

The topic of linking is incredibly vast and filled with fun architecture and
operating system specific bits. The three main operating systems (Linux, macOS,
Windows) each use different file formats for dynamically linked and linkable
objects (ELF, Mach-O, and PE, respectively). If three formats wasn't fun enough
for you, there are so many toolchain specificities that if someone found a way to
plot them I'm convinced the result would be Riemann-integrable. It's too much to
handle, and I don't know enough to tackle it all, so we're going to scope it
down.

This article is __only__ going to tackle the dynamic linking of ELF objects,
and will focus mostly on Linux. Corrections and notes on other widely used
UNIX-like operating systems are relevant and will be included, but I'm not going
to spend time figuring out why `ld(1)`'s behavior is special on SunOS, or
documenting some edge-case of [NonStop Kernel][nonstop] systems.

## Linking

Linking is the process of resolving references, know as symbols or links, into
library modules. This process can be done entirely at compile time, in which
case it is known as static linking, or at runtime, in which case we call it
dynamic linking.

FIXME: Diagram of compilation?

FIXME: verify
In the beginning, all that existed was static linking. During the build, the
compiler would find the libraries which contained the referenced symbols, and would
just bundle them in with the produced binary. Despite its age, static linking is
still very relevant today, mainly due to one key advantage it has over dynamic
linking: it's vastly simpler and more predictable. We'll revisit this point
later in this section, at which point I think proving it will be trivial.

## ELF: Executable and Linkable Format

Originally called the "Extensible Linking Format", the Executable and Linkable
Format (ELF) is a standard for many kinds of binary files, such as
executable files ("programs"), object code (i.e. not-yet-linked binaries),
shared libraries (both static and dynamic), and core dumps (what you get when
your program crashes). It is, broadly, __the__ format used for binary executable
data in UNIX-like operating systems, where it replaced the older
[a.out][aout][^1] and [COFF][coff] formats. If you've written software targeting
UNIX-like platforms in the past 20+ years, you've almost certainly been dealing
with ELF.

There is a lot to be said about ELF, but in order to remain sane we're going to
stay narrowly focused on how it relates to dynamic linking.

### Workings

{{< figure src="./images/elf101.png" caption="ELF 101 by FIXME">}}

[^1]: If you've ever used GCC to compile a C program with no output name, you
  might remember it spitting out a `a.out` file. That's a vestige of the a.out
  format, as the name remains even though GCC created an ELF binary.

[elfspec]: https://refspecs.linuxbase.org/elf/elf.pdf
[elfpfs]: https://refspecs.linuxbase.org/elf/elfspec.pdf
[aout]: https://en.wikipedia.org/wiki/A.out
[coff]: https://en.wikipedia.org/wiki/COFF
[pe]: https://en.wikipedia.org/wiki/Portable_Executable
[macho]: https://en.wikipedia.org/wiki/Mach-O
[nonstop]: https://en.wikipedia.org/wiki/NonStop_(server_computers)
