+++
title = "Notes on Euclidean Spaces"
date = 2018-08-30T00:00:00Z
draft = false
tags = [ "linear", "algebra", "mathematics", "euclidean" ]
categories = [ "notes" ]
comments = true
+++

## Real euclidean spaces

Real euclidean spaces have definitions of inner product and norm. Examples in
\\(\mathbb R^n\\):

* The usual inner product
* The unit-radius circumference when considering an unusual inner product
* Cauchy-Schwarz inequality

Let \\(V\\) be a real vector space. A form or real function \\[
\begin{aligned} \langle\cdot,\cdot\rangle\colon V\times V &\rightarrow \mathbb R
\\\\ (x, y) &\mapsto \langle x,y\rangle \end{aligned}\\ \\] is said to be an
_inner product_ if, for all \\(x, y, z \in V\\) and all \\(\alpha \in \mathbb
R\\),

1. \\(\langle x,y \rangle = \langle y, x \rangle\\)
1. \\(\langle \alpha x, y\rangle = \alpha\langle x, y\rangle\\)
1. \\(\langle x+y, z\rangle = \langle x, z\rangle + \langle y, z\rangle\\)
1. \\(\langle x, x\rangle \geq 0 \wedge (\langle x, x\rangle = 0 \implies x =
   0)\\)

A real linear space \\(V\\) equipped with an inner product is called an (real)
**Euclidean Space**.

### Examples

1. Usual inner product in \\(\mathbb R^n\\)
    1. \\(\mathbb R^2\\) \\[ \begin{aligned} \langle x, y\rangle &= \lVert
      x\rVert\lVert y\rVert \cos\theta \\\\ &= x\_1 y\_1 + x\_2 y\_2 \quad\text{in
      }\mathbb R^2 \end{aligned}\\ \\] where \\(\theta\in[0, \pi]\\) is the
      angle between the vectors \\(x\\) and \\(y\\). Note that the norm of the
      vector \\(x\\) satisfies \\[ \lVert x\rVert^2 = \langle x, x\rangle \\]
    1. \\(\mathbb R^n\\) \\[ \begin{aligned} \langle x, y\rangle &= x\_1 y\_1 + x\_2
      y\_2 + \cdots + x\_n y\_n \\\\ \langle x, y\rangle &= y^Tx = x^Ty
      \end{aligned}\\ \\]
1. Another inner product in \\(\mathbb R^2\\)
    1. **Exercise.** Determine the circumference \\(C\\) of radius \\(1\\) and
        centered at \\((0, 0)\\) \\[ C = \{(x\_1, x\_2)\in\mathbb R^2\colon
        \lVert(x\_1, x\_2)\rVert = 1\} \\] considering
        1. The usual inner product
        1. The inner product \\[\langle (x\_1, x\_2), (y\_1, y\_2) \rangle =
           \frac{1}{9} x\_1y\_1 + \frac{1}{4} x\_2y_2 \\]

### Norm and the triangle inequality

For all vectors \\(x\in V\\), we define the **norm** of \\(x\\) as \\[ \lVert
x\rVert = \sqrt{\langle x, x\rangle} \\] such that, for all \\(x\in V\\) and all
\\(\alpha\in\mathbb R\\) we have

1. \\(\lvert x\rVert\geq 0\qquad \text{and}\qquad \lVert x\rVert \iff x = 0\\)
1. \\(\lVert \alpha x\rVert = \lvert\alpha\rvert\lVert x\rVert\\)
1. \\(\lVert x + y\rVert\leq\lVert x\rVert + \lVert y\rVert \qquad\qquad
   \text{(triangle inequality)} \\)

A function \\(V \to \mathbb R\\) that satisfies the above conditions is said to
be a **norm** defined in \\(V\\).

The proof that the function we defined earlier and called "inner product"
satisfies the triangle inequality will be done at a later point, since it relies
on the Cauchy-Schwarz inequality.

### Cauchy-Schwarz Inequality

**Theorem 1.** _Let \\(V\\) be an euclidean space. For all \\(x, y \in V\\) we
have_ \\[ \lvert\langle x, y\rangle\rvert \leq \lVert x\rVert \lVert y\rVert \\]
Note that in \\(\mathbb R^2\\) and \\(\mathbb R^3\\) we have: \\[ \langle x,
y\rangle = \lVert x\rVert\lVert y\rVert\cos\theta \\] where \\[ \lvert\langle x,
y\rangle\rvert = \lVert x\rVert\lVert y\rVert\lvert \cos\theta\rvert \leq \lVert
x\rVert \lVert y\rVert \\]

### Distance

For all \\(x, y \in V\\), we define the **distance from \\(x\\) to \\(y\\)** as
\\[ d(x, y) = \lVert x - y\rVert \\]

### Parallelogram Law

For all vectors \\(x, y \in V\\), we have \\[ \lVert x + y\rVert^2 + \lVert x -
y\rVert^2 = 2(\lVert x\rVert^2 + \lVert y\rVert^2) \\]

### Example

An inner product in \\(\mathbb M\_{2\times 2}(\mathbb R)\\).

For all matrices \\(A, B \in \mathbb M\_{2\times 2}(\mathbb R)\\) we define \\[
\begin{aligned} \langle A, B\rangle &= tr(B^T A) \\\\ &= \sum^2\_{i,
j=1}{a\_{ij}b\_{ij}} \end{aligned}\\] with \\(A = [a\_{ij}]\\) and \\(B =
[b\_{ij}]\\)[^1]. Note that, letting \\(B\_c\\) be the canonical basis of
\\(\mathbb M\_{2\times 2}(\mathbb R)\\),

\\[\langle A, B \rangle\_{\mathbb M\_{2\times 2}(\mathbb R)} =
\langle (A)\_{B\_c}, (B)\_{B\_c}\rangle\_{\mathbb R^4}\\]

Meaning that the inner product defined above respects the isomorphism \\( A
\mapsto (A)\_{B\_c}\\) between \\( \mathbb{M}\_{2 \times 2}(\mathbb R) \\) and
\\(\mathbb R^4\\)

### Proof of the triangle inequality

\\[ \begin{aligned} \lVert x + y\rVert^2 &= \langle x+y, x+y\rangle \\\\ &=
\langle x, x\rangle + 2\langle x, y\rangle + \langle y, y\rangle \\\\ &= \lVert
x\rVert^2 + 2\langle x, y\rangle + \lVert y\rVert^2 \qquad (\text{Inner product
in terms of the norm})\\\\ &\leq \lVert x\rVert^2 + 2\lvert\langle x,
y\rangle\rvert + \lVert y\rVert^2 \\\\ &\leq \lVert x\rVert^2 + 2\lVert
x\rVert\lVert y\rVert + \lVert y\rVert^2 \qquad (\text{Cauchy-Schwartz
inequality}) \\\\ &=(\lVert x\rVert + \lVert y\rVert)^2 \end{aligned}\\]

Where \\[ \lVert x + y\rVert \leq \lVert x\rVert + \lVert y\rVert
\qquad_\blacksquare\\]

---

## Gram Matrix

Let \\(V\\) be a real euclidean space, and \\(B = (b\_1, b\_2, \ldots, b\_n)\\) a
basis of \\(V\\). With \\(x, y \in V\\) such that \\(x\_B = (\alpha\_1, \alpha\_2,
\ldots, \alpha\_n)\\) and \\(y\_B = \beta\_1, \beta\_2, \ldots, \beta\_n\\), we have
\\[\begin{aligned} \langle x, y\rangle &=
\langle\alpha\_1 b\_1 + \alpha\_2 b\_2 + \cdots + \alpha\_n b\_n,
\beta\_1 b\_1 + \beta\_2 b\_2 + \cdots + \beta\_n b\_n\rangle
\\\\ &= \begin{bmatrix}\beta\_1 & \beta\_2 & \ldots & \beta\_n
\end{bmatrix} \underbrace{\begin{bmatrix}\langle b\_1, b\_1\rangle &
\langle b\_2, b\_1\rangle &\ldots & \langle b\_n, b\_1\rangle
\\\\ \langle b\_1, b\_2\rangle & \langle b\_2, b\_2\rangle &\ldots &
\langle b\_n, b\_2\rangle
\\\\ \vdots
\\\\ \langle b\_1, b\_n\rangle & \langle b\_2, b\_n\rangle &\ldots &
\langle b\_n, b\_n\rangle
\\\\ \end{bmatrix}}\_G \begin{bmatrix}\alpha\_1
\\\\ \alpha\_2
\\\\ \vdots
\\\\ \alpha\_n\end{bmatrix} \end{aligned}\\ \\]

Therefore, given an inner product in \\(V\\) and a basis \\(B\\), it is possible
to determine a matrix \\(G\\) such that \\[\langle x,y\rangle = y\_B^T Gx\_B\\]

This matrix \\(G = [g\_{ij}]\\), where for all \\(i, j = 1, \ldots, n\\) we have
\\(g\_{ij} = \langle b\_j, b\_i \rangle\\) is called the **Gram matrix** of the set
of vectors \\(\\{b\_1, b\_2, \ldots, b\_n\\}\\).

Note that:

* \\(G\\) is a symmetric (\\(G = G^T\\)) \\(n\times n\\) real matrix.
* For all non-null vectors \\(x\in V\\) \\[x\_B^T Gx\_b > 0\\]

A square real matrix \\(A\\) of order \\(k\\) is said to be **positive
definite** if, for all non-null vectors \\(x\in\mathbb R^n\\), \\(x^T Ax >
0\\)

**Proposition 1.** _A symmetric real matrix is positive definite iff all your
proper values are positive._

**Theorem 2.** _Let \\(A\\) be a symmetric real matrix of order \\(n\\). The
following statements are equivalent._

1. _The expression \\[\langle x, y\rangle = y^T Ax\\] defines an inner product
   in \\(\mathbb R^n\\)_
1. _\\(A\\) is positive definite._

### Exercise

Consider that \\(\mathbb R^n\\) is equipped with the canonical basis
\\(\mathcal{e}\_n\\). What is the Gram matrix \\(G\\) that corresponds to the usual
inner product in \\(\mathbb R^n\\)? Also, which Gram matrix corresponds to the
inner product in item (2) of the previous exercise?

---

## Complex euclidean spaces and orthogonal vectors

Example of complex euclidean space: usual inner product in \\(\mathbb C^n\\).

Let \\(V\\) be a complex vector space. A complex function or form
\\[ \begin{aligned}\langle\cdot,\cdot\rangle\colon V \times V &\to \mathbb C
\\\\ (x, y) &\mapsto \langle x, y\rangle\end{aligned} \\]
is said to be an **inner product** if, for all \\(x, y, z \in V\\) and all
\\(\alpha \in \mathbb C\\)

1. \\(\langle x, y\rangle = \overline{\langle y, x\rangle}\\)
1. \\(\langle \alpha x, y\rangle = \alpha\langle x,y\rangle\\)
1. \\(\langle x + y, z\rangle = \langle x, z\rangle + \langle y, z\rangle\\)
1. \\(\langle x, x\rangle \geq 0 \wedge (\langle x, x\rangle = 0 \implies x =
   0)\\)

A complex vector space \\(V\\) equipped with an inner product is called a (complex)
**euclidean space**.

Much like with real euclidean spaces, we define the **norm** of a vector as
\\[\lVert x\rVert = \sqrt{\langle x, x\rangle}\\] and the **distance from
\\(x\\) to \\(y\\) as \\[d(x, y) = \lVert x - y\rVert\\]**

**Example.** Usual inner product in \\(\mathbb C^n\\). Let \\(x = (x\_1, x\_2,
\ldots, x\_n)\\) and \\(y\_1, y\_2, \ldots, y\_n\\) be vectors in \\(\mathbb C^n\\),
we define \\[\langle x, y\rangle = x\_1\overline{y}\_1 + x\_2\overline{y}\_2 +
\cdots + x\_n\overline{y}\_n\\] and therefore \\[\langle x, y\rangle =
\overline{y}^T x\\] Regarding the norm we have \\[\lvert x\rVert^2 =
\langle x, x\rangle = x\_1\overline{x}\_1 + x\_2\overline{x}\_2 + \cdots +
x\_n\overline{x}\_n\\] or \\[\lVert x\rVert = \sqrt{\lVert x, x\rVert} =
\sqrt{\lvert x\_1\rvert^2 + \lvert x\_2\rvert^2 + \cdots + \lvert x\_n\rvert^2}\\]

All the remaining results that were presented regarding real euclidean spaces
are also true for complex euclidean spaces (Cauchy-Schwartz, triangle
inequality, parallelogram law, …).

### Complex Gram Matrix

Let \\(V\\) be a complex euclidean space, and let \\(B = (b\_1, b\_2, \ldots,
b\_n)\\) be a basis of \\(V\\). With \\(x, y \in V\\) such that \\(x\_B=(\alpha\_1,
\alpha\_2, \dots, \alpha\_n)\\) and \\(y\_B=(\beta\_1, \beta\_2, \dots, \beta\_n)\\),
we have
\\[ \begin{aligned} \langle x, y\rangle &= \langle\alpha\_1 b\_1 +
\alpha\_2 b\_2 + \cdots + \alpha\_n b\_n, \beta\_1 b\_1 + \beta\_2 b\_2 +
\cdots + \beta\_n b\_n\rangle
\\\\ &= \begin{bmatrix}\overline{\beta}\_1 & \overline{\beta}\_2 & \ldots &
\overline{\beta}\_n\end{bmatrix} \underbrace{\begin{bmatrix}\langle b\_1,
b_1\rangle & \langle b\_2, b\_1\rangle &\ldots & \langle b\_n, b\_1\rangle
\\\\ \langle b\_1, b\_2\rangle & \langle b\_2, b\_2\rangle &\ldots & \langle
b\_n, b\_2\rangle
\\\\ \vdots
\\\\ \langle b\_1, b\_n\rangle & \langle b\_2, b\_n\rangle &\ldots &
\langle b\_n, b\_n\rangle
\\\\ \end{bmatrix}}\_G \begin{bmatrix}\alpha\_1
\\\\ \alpha\_2
\\\\ \vdots
\\\\ \alpha\_n\end{bmatrix} \end{aligned}\\ \\]

Therefore, given an inner product in \\(V\\) and a basis \\(B\\), it is possible
to determine a matrix \\(G\\) such that
\\[\langle x,y\rangle = \overline{y}\_B^T Gx\_B\\]

This matrix \\(G = [g\_{ij}]\\), where for all \\(i, j = 1, \ldots, n\\) we have
\\(g\_{ij} = \langle b\_j, b\_i \rangle\\) is called the **Gram matrix** of the
set of vectors \\(\\{b\_1, b\_2, \ldots, b\_n\\}\\).

Note that:

* \\(G\\) is an \\(n\times n\\) complex matrix such that (\\(G =
  \overline{G}^T\\)).
* For all non-null vectors \\(x\in V\\) \\[\overline{x}\_B^T Gx\_b > 0\\]

A complex square matrix \\(A\\) of order \\(k\\) is said to be **hermitian** if
\\(A = \overline{A}^T\\). Note that the spectrum \\(\sigma(A)\\) of a hermitian
is contained in \\(\mathbb R\\).

A hermitian matrix \\(A\\) of order \\(k\\) is said to be **positive definite**
if, for all non-null vectors \\(x\in\mathbb C^n\\), \\(\overline{x}^T Ax > 0\\).

**Proposition 2.** _A hermitian matrix is positive definite iff all of it's
proper values are positive._

**Theorem 3.** _Let \\(A\\) be a hermitian matrix of order \\(n\\). The
following statements are equivalent._

1. _The expression \\[\langle x, y\rangle = \overline{y}^T Ax\\] defines an
   inner product in \\(\mathbb C^n\\)_
1. _A is positive definite._

### Angle between two vectors

Let \\(x\\) and \\(y\\) be non-null vectors belonging to some **real** euclidean
space \\(V\\). We define the _angle_ between the vectors \\(x\\) and \\(y\\) as
being the angle \\(\theta\\), with \\(0\leq\theta\leq\pi\\), such that
\\[\cos\theta = \frac{\langle x, y\rangle}{\lVert x\rVert \lVert y\rVert}\\]
With Cauchy-Schwartz we can see that \\(\lvert\cos\theta\rvert\leq 1\\).

Let \\(x\\) and \\(y\\) be (possibly null) vectors belonging to some **real or
complex** euclidean space \\(V\\). The vectors \\(x\\) and \\(y\\) are said to
be **orthogonal**, written \\(x \perp y\\), if
\\[\langle x, y\rangle = 0\\]

**Exercise.** What are the orthogonal vectors to \\(v = (1, 1, 0)\\) considering
\\(\mathbb R^3\\) with the usual inner product?

**Theorem 4. (Pythagoras Theorem)** Let \\(x\\) and \\(y\\) be orthogonal
vectors of some euclidean space \\(V\\). Then
\\[\lVert x + y\rVert^2 = \lVert x\rVert^2 + \lVert y \rVert^2\\]

_Proof._ Exercise

---

## Orthogonal complement

Let \\(X\\) be a subspace of an euclidean space \\(V\\). We say that \\(u\\) is
**orthogonal to** \\(X\\) if \\(u\\) is orthogonal to all elements of \\(X\\).
We write this \\(u \perp W\\).

For example, \\((1, 1, 0)\\) is orthogonal to the plane \\(S\\) of the previous exercise.

Let \\(W\\) be a subspace of \\(V\\). The **orthogonal complement** of \\(W\\),
written \\(W^\perp\\), is defined as
\\[W^\perp = \\{u\in V\colon u\perp W\\}\\]

**Exercise.** Determine the orthogonal complement of the line generated by the
vector \\((1, 1, 0)\\).

**Proposition 3.** \\(W^\perp\\) is a subspace of V.

**Proposition 4.** *Let \\(W\\) be a linear subspace of an euclidean space
\\(V\\) and let \\(\\{u\_1, u\_2, \ldots, u\_k\\}\\) be a generator set of
\\(W\\). Then, \\(e\in V\\) is orthogonal to \\(W\\) iff it is orthogonal to
\\(\\{u\_1, u\_2, \ldots, u\_k\\}\\).*

**Corollary 1.** *In the conditions of the previous proposition, \\(u\in V\\) is
orthogonal to \\(W\\) iff it is orthogonal to a basis of \\(W\\).*

**Exercise.** Determine the orthogonal complement of the plane \\(W\in\mathbb
R^3\\) with the cartesian equation \\(x=y\\).

**Solution.** \\(W^\perp\\) is the line described by the equations
\\[\begin{cases}x = -y \\\\ z = 0\end{cases}\qquad\qquad\textbf{cartesian equations}\\]
or
\\[(x, y, z) = t(-1, 1, 0)\qquad(t\in\mathbb R)\qquad\qquad\textbf{vector equation}\\]
or
\\[\begin{cases}x = -t \\\\ y = t \\\\ z = 0\end{cases}\qquad(t\in\mathbb
R)\qquad\qquad\textbf{parametric equations}\\]

**Proposition 5.** *Let \\(W\\) be a subspace of an euclidean space \\(V\\).*

1. \\(W\cap W^\perp = 0\\)
1. \\(W^{\perp\perp} = W\\)

A subset \\(X\\) of an euclidean space \\(V\\) is said to be an **orthogonal
set** if, for all \\(x, y\in X\\) with \\(x \neq y\\) we have \\(x \perp y\\).

**Question.** Let \\(X\\) be an orthogonal set not containing the null vector.

* If \\(X\subseteq \mathbb R^2\\), how many vectors does \\(X\\) have at most?
* If \\(X\subseteq \mathbb R^3\\), how many vectors does \\(X\\) have at most?

**Proposition 6.** *Let \\(V\\) be an euclidean space. Let \\(X = \\{v\_1, v\_2,
\ldots, v\_k\\}\\) be an orthogonal set such that \\(v\_j\neq 0\\) for all
\\(j\in[1,\ldots,k]\\). Then \\(X\\) is linearly independent.*

*Proof.*
\\[\langle\alpha\_1 v\_1 + \alpha\_2 v\_2 + \cdots + \alpha\_k v\_k, v\_j\rangle
= \alpha\_j^2 \lVert v\_j \rVert^2 = 0 \implies \alpha\_j = 0\\]

**Corollary 2.** *Let \\(V\\) be an euclidean space of dimension \\(n\\), and
let \\(X = \\{v\_1, v\_2, \ldots, v\_k\\}\\) be an orthogonal set such that
\\(v\_j \neq 0\\) for all \\(j\in[1, k]\\). Then \\(k \leq n\\).*

**Corollary 3.** *Let \\(V\\) be an euclidean space of dimension \\(n\\), and
let \\(X = \\{v\_1, v\_2, \ldots, v\_n\\}\\) be an orthogonal set such that
\\(v\_j \neq 0\\) for all \\(j\in[1, n]\\). Then \\(X\\) is a basis of \\(V\\).*

### Orthogonal complements of the subspaces of a real matrix

**Proposition 7.** *Let \\(A\\) be a \\(n \times k\\) matrix with real elements.
Then, considering in \\(\mathbb R^n\\) and \\(\mathbb R^k\\) the usual inner
products we have:*[^2]

1. \\(L(A)^\perp = N(A)\\)
1. \\(N(A)^\perp = L(A)\\)
1. \\(C(A)^\perp = N(A^T)\\)
1. \\(N( A^T )^\perp = C(A)\\)

---

## Orthogonal Projections

### Orthogonal bases and orthonormal bases

A basis \\(\mathcal{B}\\) of an euclidean space \\(V\\) is said to be:

* An **orthogonal basis** if it is an orthogonal set;
* An **orthonormal basis** if it is an orthogonal set, and all it's elements
    have unitary norm.

Let \\(x\in V\\) some vector, and let
\\[(x)\_\mathcal{B} = (\alpha\_1, \alpha\_2, \ldots, \alpha\_n)\\]
be the coordinate vector of \\(x\\) in the basis \\(\mathcal B\\).

### Coordinate vector in an orthogonal basis \\(\mathcal B\\)

\\[\alpha\_j = \frac{\langle x, b\_j\rangle}{\lVert b\_j\rVert^2}\\]

### Coordinate vector in an orthonormal basis \\(\mathcal B\\)

\\[ \alpha\_j = \langle x, b\_j\rangle\\]

**Question.** Will there always be an orthogonal and/or an orthonormal basis?

**Answer.** Yes -> Orthogonalization through Gram-Schmidt method.

### Orthogonal projections

We define the **orthogonal projection of \\(x\\) over \\(b\_j\\)** as the vector
\\[\begin{aligned}\text{proj}\_{b\_j} x &= \frac{\langle x, b\_j \rangle}{\lVert
b\_j \rVert^2}b\_j \\\\ &= \alpha\_j b\_j \end{aligned}\\]

In a more general sense, given two vectors \\(u\\) and \\(v\\) from an euclidean
space \\(V\\), with \\(v\neq 0\\) the **orthogonal projection of \\(u\\) over
\\(v\\)** is the vector
\\[\text{proj}\_v u = \frac{\langle u, v \rangle}{\lVert v \rVert}^2 v\\]

**Example.** Considering that \\(\mathbb R^2\\) is equipped with the canonical
basis \\(\mathcal{E}\_2 = (e\_1, e\_2)\\), any vector \\(u\in\mathbb R^2\\) can
be expressed as a sum
\\[\begin{aligned}u &= \text{proj}\_{ e\_1} u + \text{proj}\_{ e\_2} u \\\\ &=
u\_W + u\_{W^\perp}\end{aligned}\\]
Where \\(W\\) is the \\(x\\) axis.

**Theorem 5.** _Let \\(W\\) be a linear subspace of some euclidean space
\\(V\\). All vectors \\(u\\) of \\(V\\) can be decomposed **uniquely** as_
\\[u = u\_W + u\_{W^\perp}\\]
_where \\(u\in W\\) and \\(u\_{W^\perp}\in E^\perp\\)._

In these conditions, we say that \\(V\\) is the **direct sum** of \\(W\\) with
\\(W^\perp\\) and write
\\[V = W \oplus W^\perp\\]
Which, by definition, is to say:

* \\(V = W + W^\perp\\)
* \\(W \cap W^\perp = \\{0\\}\\)

We define the **orthogonal projection of \\(u\\) over \\(W\\)** as being the
vector \\(u\_W\\).

If we consider that \\(W\\) is equipped with the ordered __orthogonal__ basis
\\(\mathcal{B} = (b\_1, b\_2, \ldots, b\_k)\\), we have
\\[\text{proj}\_W u = \text{proj}\_{b\_1} u + \text{proj}\_{b\_2} u + \cdots +
\text{proj}\_{b\_k} u\\]

**Question.** How can we compute the vector \\(u\_{W^\perp}\\) or, in other
words, \\(\text{proj}\_{W^\perp} u\\)?

**Answer.**
\\[\text{proj}\_{W^\perp} u = u - u\_W\\]
or, if we consider that \\(W^\perp\\) is equipped with the ordered orthogonal
basis \\(\mathcal{B}' = (b\_1', b\_2', \ldots, b\_l')\\), we have
\\[\text{proj}\_{W^\perp} u = \text{proj}\_{b'\_1} u + \text{proj}\_{b'\_2} u +
\cdots + \text{proj}\_{b'\_l} u\\]

**Question.** What is the number \\(l\\) of vectors in the basis of \\(\mathcal{B}'\\)?

**Answer.** Assuming that \\(V\\) has dimension \\(n\\), we have \\(l = n - k\\)
since

1. \\(\mathcal{B} \cup \mathcal{B}'\\) is linearly independent.[^3]
1. Theorem 5 guarantees that \\(\mathcal{B}\cup\mathcal{B}'\\) generates \\(V\\).

Therefore \\(\mathcal{B} \cup \mathcal{B}'\\) is a basis of \\(V\\) and the
solution becomes trivial.

---

## Distance from a point to a subspace & \\(k\\)-plane cartesian equations

### Optimal approximation

Given \\(u\in V\\) and some subspace \\(W\\) of \\(V\\) we hope to answer the
following question:

> Which element \\(x\\) of \\(W\\) is closest to \\(u\\)?

\\[\begin{aligned}d(u, x)^2 = \lVert u - x\rVert^2 &= \lVert(u - \text{proj}\_W
u) + (\text{proj}_W u - x)\rVert^2 \\\\ &= \lVert u - \text{proj}\_W u\rVert^2 +
\lVert \text{proj}\_W u - x\rVert^2 \qquad \text{(Pythagoras)}\\\\ &= \lVert
\text{proj}\_{W^\perp} u\rVert^2 + \lVert\text{proj}\_W u -
x\rVert^2\end{aligned}\\]

Whereby we conclude that

> The optimal approximation coincides with \\(\text{proj}\_W u\\) [^4]

With that, we define the **distance from \\(u\\) to a subspace \\(W\\)** as

\\[d(u, W) = \lVert proj\_{W^\perp} u \rVert\\]

### \\(k\\)-plane cartesian equations

A **\\(k\\)-plane** of \\(\mathbb R^n\\) is any subset \\(S\\) of \\(\mathbb
R^n\\) which can be expressed as

\\[S = W + p\\]

Where \\(W\\) is a subspace of \\(\mathbb R^n\\) with dimension \\(k\\) and
\\(p\\) is an element of \\(\mathbb R^n\\). Depending on the dimension of
\\(W\\), we have the following nomenclature:

* If \\(k = 0\\), \\(S\\) is said to be a **point**.
* If \\(k = 1\\), \\(S\\) is said to be a **line**.
* If \\(k = 2\\), \\(S\\) is said to be a **plane**.
* If \\(k = n - 1\\), \\(S\\) is said to be a **hyperplane**.[^5]

Let \\(x = (x\_1, x\_2, \ldots, x\_n)\\) be an elements of \\(S\\), there exists
\\(y\\) in \\(W\\) such that

\\[x = y + p\\]

Or equivalently

\\[y = x - p\\]

The last equation show that, using vector, cartesian, or parametric equations of
\\(W\\) we can easily obtain (substituting \\(y\\) for \\(x-p\\)) vector,
cartesian, or parametric equations of \\(S\\), respectively.

Analogously, using the subspace \\(W^\perp\\) we can also obtain equations of
\\(S\\). If \\(B\_{W^\perp} = (v\_1, v\_2, \ldots, v\_{n-k})\\) is a basis for
the orthogonal complement of \\(W\\), with \\(\text{dim} W = k\\), we have
\\(x - p \in W\\) or, equivalently

\\[\underbrace{\begin{bmatrix} v^T\_1 \\\\ v^T\_2 \\\\ \vdots
\\\\ v^T\_{n-k} \end{bmatrix}}\_{(n-k)\times n}
\underbrace{\begin{bmatrix} x\_1 - p\_1
\\\\ x\_2 - p\_2
\\\\ \vdots
\\\\ x\_n - p\_n \end{bmatrix}}\_{n\times 1} = \underbrace{\begin{bmatrix} 0
\\\\ 0
\\\\ \vdots
\\\\ 0
\end{bmatrix}}\_{(n-k)\times 1}\\]

Defining the matrix \\(A\\) as

\\[A = \begin{bmatrix}v^T\_1 \\\\ v^T\_2 \\\\ \vdots \\\\ v^T\_{n-k} \end{bmatrix}\\]

We obtain the homogeneous linear equation system \\(A(x -p) = 0\\).
Consequently, from a vector equation of \\(N(A)\\), or cartesian equations of
\\(N(A)\\), or parametric equations of \\(N(A)\\), we can obtain the
corresponding equations of \\(S\\).

**Exercise.** Determine a vector equation, the cartesian equations, and the
parametric equations of the plane passing the point \\(p = (1, 2, 0)\\) which is
perpendicular to the line passing this same point with direction \\(n=(5, 1, -2)\\)

### Distance from a point to a \\(k\\)-plane

Let \\(S=W+p\\) and consider a point \\(q\in\mathbb R^n\\). Given \\(x\\) in
\\(S\\),

\\[\begin{aligned}d(q, x) &= \lVert q - x \rVert \\\\  &= \lVert (q - p) + (p -
x) \\\\ &= \lVert (q - p) - y \rVert \\\\ &= d(q-p, y) \\\\ \end{aligned}\\]

The minimal value for this distance can be obtained for \\(y =
\text{proj}\_{W}(q - p)\\), as previously described. We then define the
**distance from point \\(q\\) to the plane \\(S\\)** as

\\[\begin{aligned}d(q, S) &= d(q - p, W)
\\\\ &= \lVert \text{proj}\_{W^\perp}(q - p )\rVert\end{aligned}\\]

**Exercise.** Compute the distance from \\((3, 2, -1\\) to the plane \\(S\\)
from the previous exercise.

[^1]: Note that \\(tr(B^T A) = tr(A^T B)\\), which allows us to define
    \\[\langle A, B\rangle = tr(A^T B) \\]
[^2]: I don't know whether these function names are right in English. IIRC, from
    my Portuguese notes, L(A) is the space of the lines of a matrix, C(A) is the
    space of the columns, and N(A) is the kernel.
[^3]: Because it is orthogonal.
[^4]: The closest point to \\(u\\) in \\(W\\) is \\(\text{proj}\_W u\\).
[^5]: If \\(k = n\\), \\(S = \mathbb R^n\\).
