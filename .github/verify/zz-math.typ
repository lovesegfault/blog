#import "/blog.typ": *
#show: post.with(title: [Torture], date: "2000-01-01")

#let both(..all) = all.pos().map(it => {
  let f(b) = math.equation(block: b, eval(it.text, mode: "math", scope: (warm: warm, cool: cool)))
  [#f(true)

  #drawing(alt: "", f(true))

  Text (#f(false)), text. #divider()]
}).join()

#(both(
  `mono("sizeof")(mono(T)) = mono(8) dot mono("x86 y") + bold("route 66") + sans("ab 12") + bb("AB") + frak("so") + cal("AB") + #raw("a_b") + x_mono("id") + op(bold("Set"))`,
  `lr((1/2), size: #1em) = dif x + integral dif y, dif z + f(x) dif x`,
  `a + b - c = -d, quad +x, quad a dot b times c div d, quad x != y`,
  `f(x) = x^2 + x_1^2 + x_(i j)^(k l) + f'(x) + f''_n + x'^2`,
  `sum_(i=1)^n i + product_(k in K) k + integral_0^oo e^(-x) dif x + integral.cont_C f`,
  `lim_(x -> 0) (sin x)/x = 1, quad limsup_n a_n, quad max_(x in X) f(x), quad sin^2 theta + log_2 n`,
  `limits(A)_1^2 scripts(sum)_1^2 limits(integral)_a^b op("Foo", limits: #true)_x y`,
  `1/2 + (a+b)/(c+d) + a/b/c + frac(1, 1 + frac(1, x)) + binom(n, k) + binom(n, k_1, k_2)`,
  `frac(a, b, style: "skewed") + frac(a + b, c, style: "horizontal") + 1\/2`,
  `sqrt(x) + sqrt(x^2 + y^2) + root(3, x) + root(n + 1, 1/2) + √2 + ∛x`,
  `hat(x) tilde(x) macron(x) dot(x) dot.double(x) arrow(v) breve(u) caron(c) acute(e) grave(e) circle(a) hat(i) arrow(j)`,
  `hat(a b c) tilde(a b c) arrow(A B) overline(x + y) underline(x + y) arrow.l(a b) harpoon(v) dot.triple(x) arrow.l.r(A B)`,
  `overbrace(1 + 2 + 3, "six") underbrace(a + b, c) overbracket(x y, 1) underbracket(x y) overparen(A B) underparen(A B) overshell(a b) undershell(a b, c)`,
  `(1/2) [1/2] {1/2} abs(1/2) norm(1/2) floor(1/2) ceil(1/2) round(1/2) lr(chevron.l 1/2 chevron.r) lr(]1/2\[) lr(|1/2|)`,
  `{x mid(|) x > 1/2} quad lr((a), size: #200%) quad \(1/2\) quad lr(1/2|)_0^1 quad [|x|] |x| + |y|`,
  `mat(1, 2; 3, 4) vec(a, b) mat(delim: "[", 1, 0; 0, 1) mat(delim: "|", a, b; c, d) mat(delim: #none, 1, 2) mat(1, 2, 3; 4, 5, 6; augment: #2)`,
  `mat(align: #right, -1, 10; 100, -2) mat(a &+ b, c; x y &+ z, w) mat(1, 2; 3, 4; augment: #(hline: 1, vline: 1)) mat(1/2, dots.h.c; dots.v, dots.down)`,
  `f(x) = cases(1 & "if" x > 0, 0 & "otherwise") quad cases(reverse: #true, a, b) = c`,
  `a &= b + c \ &= d & "(why)" \ e + f &< g`,
  `a \ b + c`,
  `bold(x) bold(alpha) bold(upright(A)) upright(d) italic("ab") sans(A x) mono(A x) frak(g G) bb(1 k R) cal(A B L) scr(A B L) bold(cal(A)) bold(1) bold(+)`,
  `alpha beta Gamma Delta epsilon epsilon.alt phi phi.alt theta.alt nabla partial oo aleph ell planck emptyset forall exists Re Im`,
  `a -> b => c <-> d |-> e arrow.r.long f ~> g arrow.hook h arrow.r.twohead i <== j`,
  `A ->^f B stretch(->)^"long label" C stretch(=)^"def" D ->^x_y E stretch(<-, size: #3em) F =^! G`,
  `attach(X, tl: a, bl: b, tr: c, br: d) attach(sum, t: n, b: m, tr: *) attach(C, tl: 14, bl: 6) ""^2 x`,
  `cancel(x) + cancel(a + b, inverted: #true) + cancel(y, cross: #true)`,
  `a thin b med c thick d quad e wide f #h(1em) g #h(-.2em) h space i`,
  `"text" x "is" y, quad x"th" quad "a, b" quad 3.14 quad 1,000 quad 10% quad n! quad 90 degree quad x...y`,
  `display(1/2) inline(1/2) script(a b) sscript(a b) x_display(y) display(sum_a^b)`,
  `class("relation", "loves") a class("binary", star) b class("normal", +) c class("large", X)_1 d`,
  `#text(warm)[$x$] + #text(cool, $y^2$) + #box(stroke: 1pt, inset: 2pt, $E = m c^2$) + #hide($x$) + #strong[bold] #emph[it] #link("https://a.b")[$z$]`,
  `x in A subset.eq B union C inter D without E, not p and q or r, a equiv b (mod n), a mod b, a | b, a || b, tack.r p, models q`,
  `integral.double integral.triple union.big_i inter.big_i and.big_i plus.o.big_i times.o product.co_i`,
  `dif x dif y, Dif f, (dif y)/(dif x), (partial f)/(partial x), x^* x^+ x^(-1) x_- x^dagger x^top x^circle.small`,
  `P(A | B) = (P(B | A) P(A)) / P(B), quad EE[X], quad Pr(X <= x), quad O(n log n), quad a := b`,
  `é "é ñ" ж "fi" alpha beta gamma arrow.t.b #sym.qed & < > "<&>"`,
  `sum_(0 <= i < n \ i != j) a_i + x_(a &= b \ c &= d) + (a \ b)`,
  `primes(#5) f''' f'''' prime x^prime`,
))


#drawing(alt: "x", stack(dir: ltr, spacing: 1em, rect(fill: warm.transparentize(80%), stroke: warm)[`*ptr` _it_ *bold*], circle(fill: white)[$x^2$], rect(fill: gradient.linear(warm, cool))[a]))

```typ
= Heading
Some *bold* and _it_ `raw` $x^2 + sum_(i=1)$ <label> @ref
- item
#let f(x) = { if x > 1 [a] else { "b" } } // comment
```

$ $ and $$ and #text(warm, $x > y$)
