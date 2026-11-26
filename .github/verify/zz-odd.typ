#import "/blog.typ": *
#show: post.with(title: [Odd $x^2 + 1$ title], date: "2000-01-02")
#let n = counter("n")
#let step = [#n.step()#context n.display("(i)")]

= On $sqrt(2)$ and $f'(x)$

Long on a desk,
$ (a + b)^9 = a^9 + 9 a^8 b + 36 a^7 b^2 + 84 a^6 b^3 + 126 a^5 b^4 + 126 a^4 b^5 + 84 a^3 b^6 + 36 a^2 b^7 + 9 a b^8 + b^9 $
and numbered,
$ (a + b)^9 = a^9 + 9 a^8 b + 36 a^7 b^2 + 84 a^6 b^3 + 126 a^5 b^4 + 126 a^4 b^5 + 84 a^3 b^6 + 36 a^2 b^7 + 9 a b^8 + b^9 $ <nine>
and counted,
$ a &= b & wide #step \ &= c & #step \ &= d & #[by @nine] $
after which #context n.get().first() steps were taken. A *bold $x + y$*, an _italic $x + y$_, a #link("/about/")[linked $x + y$], #highlight[marked $x$], #strike[struck $x$], $f$'s and "$g$".
#math.equation(alt: "a red square", $#rect(width: 1em, height: 1em, fill: warm)$) has words of its own,
#math.equation(alt: "turned", $#rotate(20deg, reflow: true)[ab] #rotate(20deg, reflow: true)[ab]$).
Empty: $$ and $ $ and $#h(2em)$ and $#hide[x]$ and $ \ $ and $&$ and $ & = 1 $ and $ a \ $ and $ a & \ & b $.
Big: #text(2em)[$x + y$] and $#text(2em)[$x$] + y$ and $#text(stroke: .3pt, $x$)$ and $#text(gradient.linear(warm, cool), $x y z$)$.
$ (a \ b)^2 + attach((a + b), t: x, tr: y) + limits((a + b))^x + lr(size: #200%, (a + b)) + (dif x) $
$ #raw("size_t") n = #`sizeof`\(x) "for all" x "in 2 steps, see 5" $
#table(columns: 2, $ a + b = c + d + e + f + g + h + i + j $, [$a + b = c + d + e + f + g + h + i + j$])
#margin[$ a + b = c + d + e + f + g + h + i + j + k $]
#quote[$ e^(i pi) + 1 = 0 $ is $beta$]
#note[$ a = b $]
A note.#footnote[$ sum_(i=0)^n a_i b_i = a_0 b_0 + a_1 b_1 + dots.c + a_n b_n $]
#figure($ a = b $, caption: [With $x$])
$ #[#box[x]<inside>] + y $ #link(<inside>)[there]
$ ((((((((((a + b + c + d + e + f + g + h + i + j + k + l + m + n + o + p)))))))))) $
$ f(g(h(alpha + beta + gamma, delta + epsilon), zeta + eta + theta + iota), kappa + lambda + mu + nu + xi + pi) $
#quote[Verse \ $ a + b + c + d + e + f = g + h + i + j + k + l + m $]
$ x + #context $a \ b$ + y $
#[#show math.equation: set block(fill: warm.transparentize(80%), inset: .4em); $ a = b $]
#[#show math.equation: set block(clip: true); $ a = b $]
#table(columns: 4, $ x $, $ y $, $ z $, $ w $)
#[#set block(fill: warm, inset: 1em, width: 100%); #set par(first-line-indent: (amount: 2em, all: true)); $a + b$ and $ a + b $]
In #box[a $ x + y = z $] box.
$ cancel(a b c) + cancel(d e f, inverted: #true) + cancel(g h i, cross: #true) + cancel(j k l) + cancel(m n o) + cancel(p q r) + cancel(s t u) + cancel(v w) $
#math.equation(block: true, alt: "turned", $x #rotate(30deg, reflow: true)[turned] + #scale(x: -100%)[flipped] + #box(clip: true, width: 1em, height: .6em)[clipped]$)
#[#show "such that": strong; $ x "such that the words go on for quite a while, ε and all" y "שלום עולם" $]
$ x #underline[such that the words go on for a while] y $
#let twice(x) = $2 #x$
#twice($y$), $a #h(-3em) b$, $p$, $ p = #h(-2em) & q $
