#import "/blog.typ": *

#show: post.with(
  title: [Notes on Euclidean Spaces],
  date: "2018-08-30T00:00:00Z",
  categories: "notes",
  tags: ("linear", "algebra", "mathematics", "euclidean"),
)

#set math.mat(delim: "[")
#set math.vec(delim: "[")
#let colon = math.class("punctuation", sym.colon)
#let inner(x, y) = $lr(chevron.l #x, #y chevron.r)$
#let proj = math.op("proj")

= Real euclidean spaces

Real euclidean spaces have definitions of inner product and norm. Examples in
$RR^n$:

- The usual inner product
- The unit-radius circumference when considering an unusual inner product
- Cauchy-Schwarz inequality

Let $V$ be a real vector space. A form or real function
$
  inner(dot, dot) colon V times V &-> RR \
  (x, y) &|-> inner(x, y)
$
is said to be an _inner product_ if, for all $x, y, z in V$ and all
$alpha in RR$,

+ $inner(x, y) = inner(y, x)$
+ $inner(alpha x, y) = alpha inner(x, y)$
+ $inner(x + y, z) = inner(x, z) + inner(y, z)$
+ $inner(x, x) >= 0 and (inner(x, x) = 0 ==> x = 0)$

A real linear space $V$ equipped with an inner product is called an (real)
*Euclidean Space*.

== Examples

+ Usual inner product in $RR^n$
  - $RR^2$
    $
      inner(x, y) &= norm(x) norm(y) cos theta \
      &= x_1 y_1 + x_2 y_2 quad "in" RR^2
    $
    where $theta in [0, pi]$ is the angle between the vectors $x$ and $y$. Note
    that the norm of the vector $x$ satisfies
    $ norm(x)^2 = inner(x, x) $
  - $RR^n$
    $
      inner(x, y) &= x_1 y_1 + x_2 y_2 + dots.c + x_n y_n \
      inner(x, y) &= y^T x = x^T y
    $
+ Another inner product in $RR^2$
  - *Exercise.* Determine the circumference $C$ of radius $1$ and centered at
    $(0, 0)$
    $ C = {(x_1, x_2) in RR^2 : norm((x_1, x_2)) = 1} $
    considering
    - The usual inner product
    - The inner product
      $ inner((x_1, x_2), (y_1, y_2)) = 1/9 x_1 y_1 + 1/4 x_2 y_2 $

== Norm and the triangle inequality

For all vectors $x in V$, we define the *norm* of $x$ as
$ norm(x) = sqrt(inner(x, x)) $
such that, for all $x in V$ and all $alpha in RR$ we have

+ $lr(|x||) >= 0 wide "and" wide norm(x) <==> x = 0$
+ $norm(alpha x) = abs(alpha) norm(x)$
+ $norm(x + y) <= norm(x) + norm(y) wide wide "(triangle inequality)"$

A function $V -> RR$ that satisfies the above conditions is said to be a *norm*
defined in $V$.

The proof that the function we defined earlier and called "inner product"
satisfies the triangle inequality will be done at a later point, since it relies
on the Cauchy-Schwarz inequality.

== Cauchy-Schwarz Inequality

*Theorem 1.* _Let $V$ be an euclidean space. For all $x, y in V$ we have_
$ abs(inner(x, y)) <= norm(x) norm(y) $
Note that in $RR^2$ and $RR^3$ we have:
$ inner(x, y) = norm(x) norm(y) cos theta $
where
$ abs(inner(x, y)) = norm(x) norm(y) abs(cos theta) <= norm(x) norm(y) $

== Distance

For all $x, y in V$, we define the *distance from $x$ to $y$* as
$ d(x, y) = norm(x - y) $

== Parallelogram Law

For all vectors $x, y in V$, we have
$ norm(x + y)^2 + norm(x - y)^2 = 2(norm(x)^2 + norm(y)^2) $

== Example

An inner product in $MM_(2 times 2) (RR)$.

For all matrices $A, B in MM_(2 times 2) (RR)$ we define
$
  inner(A, B) &= tr(B^T A) \
  &= sum_(i, j = 1)^2 a_(i j) b_(i j)
$
with $A = [a_(i j)]$ and $B = [b_(i j)]$#footnote[
  Note that $tr(B^T A) = tr(A^T B)$, which allows us to define
  $ inner(A, B) = tr(A^T B) $
]. Note that, letting $B_c$ be the canonical basis of $MM_(2 times 2) (RR)$

$ inner(A, B)_(MM_(2 times 2) (RR)) = inner((A)_(B_c), (B)_(B_c))_(RR^4) $

Meaning that the inner product defined above respects the isomorphism
$A |-> (A)_(B_c)$ between $MM_(2 times 2) (RR)$ and $RR^4$

== Proof of the triangle inequality

$
  norm(x + y)^2 &= inner(x + y, x + y) \
  &= inner(x, x) + 2 inner(x, y) + inner(y, y) \
  &= norm(x)^2 + 2 inner(x, y) + norm(y)^2
  wide ("Inner product in terms of the norm") \
  &<= norm(x)^2 + 2 abs(inner(x, y)) + norm(y)^2 \
  &<= norm(x)^2 + 2 norm(x) norm(y) + norm(y)^2
  wide ("Cauchy-Schwartz inequality") \
  &= (norm(x) + norm(y))^2
$

Where
$ norm(x + y) <= norm(x) + norm(y) wide ""_square.filled $

#divider()

= Gram Matrix

Let $V$ be a real euclidean space, and $B = (b_1, b_2, dots, b_n)$ a basis of
$V$. With $x, y in V$ such that $x_B = (alpha_1, alpha_2, dots, alpha_n)$ and
$y_B = beta_1, beta_2, dots, beta_n$, we have
$
  inner(x, y) &= inner(
    alpha_1 b_1 + alpha_2 b_2 + dots.c + alpha_n b_n,
    beta_1 b_1 + beta_2 b_2 + dots.c + beta_n b_n
  ) \
  &= mat(beta_1, beta_2, dots, beta_n)
  underbrace(
    mat(
      inner(b_1, b_1), inner(b_2, b_1), dots, inner(b_n, b_1);
      inner(b_1, b_2), inner(b_2, b_2), dots, inner(b_n, b_2);
      dots.v;
      inner(b_1, b_n), inner(b_2, b_n), dots, inner(b_n, b_n);
    ),
    G
  )
  vec(alpha_1, alpha_2, dots.v, alpha_n)
$

Therefore, given an inner product in $V$ and a basis $B$, it is possible to
determine a matrix $G$ such that
$ inner(x, y) = y_B^T G x_B $

This matrix $G = [g_(i j)]$, where for all $i, j = 1, dots, n$ we have
$g_(i j) = inner(b_j, b_i)$ is called the *Gram matrix* of the set of vectors
${b_1, b_2, dots, b_n}$.

Note that:

- $G$ is a symmetric ($G = G^T$) $n times n$ real matrix.
- For all non-null vectors $x in V$
  $ x_B^T G x_b > 0 $

A square real matrix $A$ of order $k$ is said to be *positive definite* if, for
all non-null vectors $x in RR^n$, $x^T A x > 0$

*Proposition 1.* _A symmetric real matrix is positive definite iff all your
proper values are positive._

*Theorem 2.* _Let $A$ be a symmetric real matrix of order $n$. The following
statements are equivalent._

+ _The expression
  $ inner(x, y) = y^T A x $
  defines an inner product in $RR^n$_
+ _$A$ is positive definite._

== Exercise

Consider that $RR^n$ is equipped with the canonical basis $cal(e)_n$. What is
the Gram matrix $G$ that corresponds to the usual inner product in $RR^n$? Also,
which Gram matrix corresponds to the inner product in item (2) of the previous
exercise?

#divider()

= Complex euclidean spaces and orthogonal vectors

Example of complex euclidean space: usual inner product in $CC^n$.

Let $V$ be a complex vector space. A complex function or form
$
  inner(dot, dot) colon V times V &-> CC \
  (x, y) &|-> inner(x, y)
$
is said to be an *inner product* if, for all $x, y, z in V$ and all
$alpha in CC$

+ $inner(x, y) = overline(inner(y, x))$
+ $inner(alpha x, y) = alpha inner(x, y)$
+ $inner(x + y, z) = inner(x, z) + inner(y, z)$
+ $inner(x, x) >= 0 and (inner(x, x) = 0 ==> x = 0)$

A complex vector space $V$ equipped with an inner product is called a (complex)
*euclidean space*.

Much like with real euclidean spaces, we define the *norm* of a vector as
$ norm(x) = sqrt(inner(x, x)) $
and the *distance from $x$ to $y$ as
$ d(x, y) = norm(x - y) $*

*Example.* Usual inner product in $CC^n$. Let $x = (x_1, x_2, dots, x_n)$ and
$y_1, y_2, dots, y_n$ be vectors in $CC^n$, we define
$
  inner(x, y)
  = x_1 overline(y)_1 + x_2 overline(y)_2 + dots.c + x_n overline(y)_n
$
and therefore
$ inner(x, y) = overline(y)^T x $
Regarding the norm we have
$
  lr(|x||)^2 = inner(x, x)
  = x_1 overline(x)_1 + x_2 overline(x)_2 + dots.c + x_n overline(x)_n
$
or
$
  norm(x) = sqrt(norm(x\, x))
  = sqrt(abs(x_1)^2 + abs(x_2)^2 + dots.c + abs(x_n)^2)
$

All the remaining results that were presented regarding real euclidean spaces
are also true for complex euclidean spaces (Cauchy-Schwartz, triangle
inequality, parallelogram law, …).

== Complex Gram Matrix

Let $V$ be a complex euclidean space, and let $B = (b_1, b_2, dots, b_n)$ be a
basis of $V$. With $x, y in V$ such that
$x_B = (alpha_1, alpha_2, dots, alpha_n)$ and
$y_B = (beta_1, beta_2, dots, beta_n)$, we have
$
  inner(x, y) &= inner(
    alpha_1 b_1 + alpha_2 b_2 + dots.c + alpha_n b_n,
    beta_1 b_1 + beta_2 b_2 + dots.c + beta_n b_n
  ) \
  &= mat(overline(beta)_1, overline(beta)_2, dots, overline(beta)_n)
  underbrace(
    mat(
      inner(b_1, b_1), inner(b_2, b_1), dots, inner(b_n, b_1);
      inner(b_1, b_2), inner(b_2, b_2), dots, inner(b_n, b_2);
      dots.v;
      inner(b_1, b_n), inner(b_2, b_n), dots, inner(b_n, b_n);
    ),
    G
  )
  vec(alpha_1, alpha_2, dots.v, alpha_n)
$

Therefore, given an inner product in $V$ and a basis $B$, it is possible to
determine a matrix $G$ such that
$ inner(x, y) = overline(y)_B^T G x_B $

This matrix $G = [g_(i j)]$, where for all $i, j = 1, dots, n$ we have
$g_(i j) = inner(b_j, b_i)$ is called the *Gram matrix* of the set of vectors
${b_1, b_2, dots, b_n}$.

Note that:

- $G$ is an $n times n$ complex matrix such that ($G = overline(G)^T$).
- For all non-null vectors $x in V$
  $ overline(x)_B^T G x_b > 0 $

A complex square matrix $A$ of order $k$ is said to be *hermitian* if
$A = overline(A)^T$. Note that the spectrum $sigma(A)$ of a hermitian is
contained in $RR$.

A hermitian matrix $A$ of order $k$ is said to be *positive definite* if, for
all non-null vectors $x in CC^n$, $overline(x)^T A x > 0$.

*Proposition 2.* _A hermitian matrix is positive definite iff all of it's
proper values are positive._

*Theorem 3.* _Let $A$ be a hermitian matrix of order $n$. The following
statements are equivalent._

+ _The expression
  $ inner(x, y) = overline(y)^T A x $
  defines an inner product in $CC^n$_
+ _A is positive definite._

== Angle between two vectors

Let $x$ and $y$ be non-null vectors belonging to some *real* euclidean space
$V$. We define the _angle_ between the vectors $x$ and $y$ as being the angle
$theta$, with $0 <= theta <= pi$, such that
$ cos theta = inner(x, y) / (norm(x) norm(y)) $
With Cauchy-Schwartz we can see that $abs(cos theta) <= 1$.

Let $x$ and $y$ be (possibly null) vectors belonging to some *real or complex*
euclidean space $V$. The vectors $x$ and $y$ are said to be *orthogonal*,
written $x perp y$, if
$ inner(x, y) = 0 $

*Exercise.* What are the orthogonal vectors to $v = (1, 1, 0)$ considering
$RR^3$ with the usual inner product?

*Theorem 4. (Pythagoras Theorem)* Let $x$ and $y$ be orthogonal vectors of some
euclidean space $V$. Then
$ norm(x + y)^2 = norm(x)^2 + norm(y)^2 $

_Proof._ Exercise

#divider()

= Orthogonal complement

Let $X$ be a subspace of an euclidean space $V$. We say that $u$ is *orthogonal
to* $X$ if $u$ is orthogonal to all elements of $X$. We write this $u perp W$.

For example, $(1, 1, 0)$ is orthogonal to the plane $S$ of the previous
exercise.

Let $W$ be a subspace of $V$. The *orthogonal complement* of $W$, written
$W^perp$, is defined as
$ W^perp = {u in V : u perp W} $

*Exercise.* Determine the orthogonal complement of the line generated by the
vector $(1, 1, 0)$.

*Proposition 3.* $W^perp$ is a subspace of V.

*Proposition 4.* _Let $W$ be a linear subspace of an euclidean space $V$ and let
${u_1, u_2, dots, u_k}$ be a generator set of $W$. Then, $e in V$ is orthogonal
to $W$ iff it is orthogonal to ${u_1, u_2, dots, u_k}$._

*Corollary 1.* _In the conditions of the previous proposition, $u in V$ is
orthogonal to $W$ iff it is orthogonal to a basis of $W$._

*Exercise.* Determine the orthogonal complement of the plane $W in RR^3$ with
the cartesian equation $x = y$.

*Solution.* $W^perp$ is the line described by the equations
$ cases(x = -y, z = 0) wide wide bold("cartesian equations") $
or
$ (x, y, z) = t(-1, 1, 0) wide (t in RR) wide wide bold("vector equation") $
or
$
  cases(x = -t, y = t, z = 0) wide (t in RR)
  wide wide bold("parametric equations")
$

*Proposition 5.* _Let $W$ be a subspace of an euclidean space $V$._

+ $W inter W^perp = 0$
+ $W^(perp perp) = W$

A subset $X$ of an euclidean space $V$ is said to be an *orthogonal set* if, for
all $x, y in X$ with $x != y$ we have $x perp y$.

*Question.* Let $X$ be an orthogonal set not containing the null vector.

- If $X subset.eq RR^2$, how many vectors does $X$ have at most?
- If $X subset.eq RR^3$, how many vectors does $X$ have at most?

*Proposition 6.* _Let $V$ be an euclidean space. Let $X = {v_1, v_2, dots, v_k}$
be an orthogonal set such that $v_j != 0$ for all $j in [1, dots, k]$. Then $X$
is linearly independent._

_Proof._
$
  inner(alpha_1 v_1 + alpha_2 v_2 + dots.c + alpha_k v_k, v_j)
  = alpha_j^2 norm(v_j)^2 = 0 ==> alpha_j = 0
$

*Corollary 2.* _Let $V$ be an euclidean space of dimension $n$, and let
$X = {v_1, v_2, dots, v_k}$ be an orthogonal set such that $v_j != 0$ for all
$j in [1, k]$. Then $k <= n$._

*Corollary 3.* _Let $V$ be an euclidean space of dimension $n$, and let
$X = {v_1, v_2, dots, v_n}$ be an orthogonal set such that $v_j != 0$ for all
$j in [1, n]$. Then $X$ is a basis of $V$._

== Orthogonal complements of the subspaces of a real matrix

*Proposition 7.* _Let $A$ be a $n times k$ matrix with real elements. Then,
considering in $RR^n$ and $RR^k$ the usual inner products we have:_#footnote[
  I don't know whether these function names are right in English. IIRC, from my
  Portuguese notes, L(A) is the space of the lines of a matrix, C(A) is the
  space of the columns, and N(A) is the kernel.
]

+ $L(A)^perp = N(A)$
+ $N(A)^perp = L(A)$
+ $C(A)^perp = N(A^T)$
+ $N(A^T)^perp = C(A)$

#divider()

= Orthogonal Projections

== Orthogonal bases and orthonormal bases

A basis $cal(B)$ of an euclidean space $V$ is said to be:

- An *orthogonal basis* if it is an orthogonal set;
- An *orthonormal basis* if it is an orthogonal set, and all it's elements
  have unitary norm.

Let $x in V$ some vector, and let
$ (x)_cal(B) = (alpha_1, alpha_2, dots, alpha_n) $
be the coordinate vector of $x$ in the basis $cal(B)$.

== Coordinate vector in an orthogonal basis $cal(B)$
<coordinate-vector-in-an-orthogonal-basis-mathcal-b>

$ alpha_j = inner(x, b_j) / norm(b_j)^2 $

== Coordinate vector in an orthonormal basis $cal(B)$
<coordinate-vector-in-an-orthonormal-basis-mathcal-b>

$ alpha_j = inner(x, b_j) $

*Question.* Will there always be an orthogonal and/or an orthonormal basis?

*Answer.* Yes -> Orthogonalization through Gram-Schmidt method.

== Orthogonal projections

We define the *orthogonal projection of $x$ over $b_j$* as the vector
$
  proj_(b_j) x &= inner(x, b_j) / norm(b_j)^2 b_j \
  &= alpha_j b_j
$

In a more general sense, given two vectors $u$ and $v$ from an euclidean space
$V$, with $v != 0$ the *orthogonal projection of $u$ over $v$* is the vector
$ proj_v u = frac(inner(u, v), norm(v))^2 v $

*Example.* Considering that $RR^2$ is equipped with the canonical basis
$cal(E)_2 = (e_1, e_2)$, any vector $u in RR^2$ can be expressed as a sum
$
  u &= proj_(e_1) u + proj_(e_2) u \
  &= u_W + u_(W^perp)
$
Where $W$ is the $x$ axis.

*Theorem 5.* _Let $W$ be a linear subspace of some euclidean space $V$. All
vectors $u$ of $V$ can be decomposed *uniquely* as_
$ u = u_W + u_(W^perp) $
_where $u in W$ and $u_(W^perp) in E^perp$._

In these conditions, we say that $V$ is the *direct sum* of $W$ with $W^perp$
and write
$ V = W plus.o W^perp $
Which, by definition, is to say:

- $V = W + W^perp$
- $W inter W^perp = {0}$

We define the *orthogonal projection of $u$ over $W$* as being the vector $u_W$.

If we consider that $W$ is equipped with the ordered *orthogonal* basis
$cal(B) = (b_1, b_2, dots, b_k)$, we have
$ proj_W u = proj_(b_1) u + proj_(b_2) u + dots.c + proj_(b_k) u $

*Question.* How can we compute the vector $u_(W^perp)$ or, in other words,
$proj_(W^perp) u$?

*Answer.*
$ proj_(W^perp) u = u - u_W $
or, if we consider that $W^perp$ is equipped with the ordered orthogonal basis
$cal(B)' = (b'_1, b'_2, dots, b'_l)$, we have
$ proj_(W^perp) u = proj_(b'_1) u + proj_(b'_2) u + dots.c + proj_(b'_l) u $

*Question.* What is the number $l$ of vectors in the basis of $cal(B)'$?

*Answer.* Assuming that $V$ has dimension $n$, we have $l = n - k$ since

+ $cal(B) union cal(B)'$ is linearly independent.#footnote[Because it is
  orthogonal.]
+ Theorem 5 guarantees that $cal(B) union cal(B)'$ generates $V$.

Therefore $cal(B) union cal(B)'$ is a basis of $V$ and the solution becomes
trivial.

#divider()

= Distance from a point to a subspace & $k$-plane cartesian equations

== Optimal approximation

Given $u in V$ and some subspace $W$ of $V$ we hope to answer the following
question:

#important[
  Which element $x$ of $W$ is closest to $u$?
]

$
  d(u, x)^2 = norm(u - x)^2 &= norm((u - proj_W u) + (proj_W u - x))^2 \
  &= norm(u - proj_W u)^2 + norm(proj_W u - x)^2 wide "(Pythagoras)" \
  &= norm(proj_(W^perp) u)^2 + norm(proj_W u - x)^2
$

Whereby we conclude that

#important[
  The optimal approximation coincides with $proj_W u$~#footnote[
    The closest point to $u$ in $W$ is $proj_W u$.
  ]
]

With that, we define the *distance from $u$ to a subspace $W$* as

$ d(u, W) = norm(proj_(W^perp) u) $

== $k$-plane cartesian equations

A *$k$-plane* of $RR^n$ is any subset $S$ of $RR^n$ which can be expressed as

$ S = W + p $

Where $W$ is a subspace of $RR^n$ with dimension $k$ and $p$ is an element of
$RR^n$. Depending on the dimension of $W$, we have the following nomenclature:

- If $k = 0$, $S$ is said to be a *point*.
- If $k = 1$, $S$ is said to be a *line*.
- If $k = 2$, $S$ is said to be a *plane*.
- If $k = n - 1$, $S$ is said to be a *hyperplane*.#footnote[If $k = n$,
  $S = RR^n$.]

Let $x = (x_1, x_2, dots, x_n)$ be an elements of $S$, there exists $y$ in $W$
such that

$ x = y + p $

Or equivalently

$ y = x - p $

The last equation show that, using vector, cartesian, or parametric equations of
$W$ we can easily obtain (substituting $y$ for $x - p$) vector, cartesian, or
parametric equations of $S$, respectively.

Analogously, using the subspace $W^perp$ we can also obtain equations of $S$. If
$B_(W^perp) = (v_1, v_2, dots, v_(n - k))$ is a basis for the orthogonal
complement of $W$, with $dim W = k$, we have $x - p in W$ or, equivalently

$
  underbrace(vec(v_1^T, v_2^T, dots.v, v_(n - k)^T), (n - k) times n)
  underbrace(vec(x_1 - p_1, x_2 - p_2, dots.v, x_n - p_n), n times 1)
  = underbrace(vec(0, 0, dots.v, 0), (n - k) times 1)
$

Defining the matrix $A$ as

$ A = vec(v_1^T, v_2^T, dots.v, v_(n - k)^T) $

We obtain the homogeneous linear equation system $A(x - p) = 0$. Consequently,
from a vector equation of $N(A)$, or cartesian equations of $N(A)$, or
parametric equations of $N(A)$, we can obtain the corresponding equations of
$S$.

*Exercise.* Determine a vector equation, the cartesian equations, and the
parametric equations of the plane passing the point $p = (1, 2, 0)$ which is
perpendicular to the line passing this same point with direction
$n = (5, 1, -2)$

== Distance from a point to a $k$-plane

Let $S = W + p$ and consider a point $q in RR^n$. Given $x$ in $S$,

$
  d(q, x) &= norm(q - x) \
  &= class("opening", ||) (q - p) + (p - x) \
  &= norm((q - p) - y) \
  &= d(q - p, y)
$

The minimal value for this distance can be obtained for $y = proj_W (q - p)$, as
previously described. We then define the *distance from point $q$ to the plane
$S$* as

$
  d(q, S) &= d(q - p, W) \
  &= norm(proj_(W^perp) (q - p))
$

*Exercise.* Compute the distance from $(3, 2, -1)$ to the plane $S$ from the
previous exercise.
