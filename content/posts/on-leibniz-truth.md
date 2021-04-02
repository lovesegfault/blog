+++
title = "On Leibniz's Truth"
date = 2017-10-25T00:00:00Z
draft = false
tags = [ "philosophy", "logic", "leibniz", "theory" ]
categories = [ "essay" ]
comments = true
+++

Let a *statement* be any sentence of form

\\[
\mathcal S + \phi
\\]

Where \\(\mathcal S\\) is a *subject* and \\(\phi\\) is a *predicate*. The
sentences "Spinoza died in the Hague", and "Grey is grey", for example, qualify
as statements. In the first, we have "Spinoza" as the subject, and "died in the
Hague" as the predicate. Note that, for a sentence to qualify as a statement,
its predicate does not need to contain an object (consider the sentence
\\(X\text{ is.}\\)) Statements are said to be *true* if and only if "the
*concept-predicate* is contained in the *concept-subject*" (Leiniz's
Concept-Containment Theory of Truth).

\\[
\mathcal S + \phi\text{ is true} \iff \phi\in\mathcal C(\mathcal S)
\\]

Where here we use \\(\mathcal C(\mathcal S)\\) to denote the *concept-subject*,
or *the concept of \\(\mathcal S\\)*. Trivial examples of concept-containment
are tautological statements such as

\\[
X\text{ is }X
\\]

Statements like this are clearly true, and the concept-containment is evident as
well, since it follows from the Law of Identity that the concept of \\(X\\) is
contained in \\(X\\), in other words, tautological statement are such that the
subject-concept and the predicate-concept are the same. These types of
statements are called *Primary Truths*.

Let us now consider for a moment the following statement:

\\[
\text{Washington crossed the Delaware}
\\]

Unlike the identity statement we saw above, here the subject-concept is *not*
the same as the predicate-concept, and most importantly there is no clear
containment of the latter into the former, which is why we call it a *Secondary
Truth*. One can, for example, easily imagine Washington *not* having crossed the
Delaware, while on the other hand attempting to imagine \\(X \text{ is not }X\\)
is fruitless at best (and maddening at worst). In a sense, it seems that while
all primary truths are "self sufficient", and depend on no external factors to
be true, the same is not the case for secondary truths, they seem to depend on
large class of externalites. For Washington to have crossed the Delaware, for
example, a number of dependencies must have been satisfied, namely, Washington
must exist, and so must the Delaware, and the Earth on which they act, and so
on. It seems to us that the statement could well be false, without any dooming
consequences, once again, it does not take a lot of mental gymnastics to imagine
a universe where Washington simply did not cross the Delaware. The solution to
this problem lies in the fact that all truths are what we call *analytic*,
meaning they can be reduced to primary truths, which in turn are trivially true.

Let us begin by acknowledging Washington as being more than an individual, we
are referring here to a *concept* of Washington. This concept, we must note, is
not atomic, and can be divided into a set of elements that compose it, which is
to say we have

\\[
\mathcal C(\mathcal S) = \\{\alpha_1, \alpha_2, \ldots, \alpha_n\\},
\quad n\in\Bbb N \cup \\\{+\infty\\\}
\\]

Denoting a concept-subject as being composed of \\(n\\) characteristics. We now
note, however, that any such \\(\mathcal C(\mathcal S)\\) is unique, that is to
say that there can be no subject \\(\mathcal S'\\) such that \\(\mathcal
C(\mathcal S') = \\{\alpha_1, \alpha_2, \ldots, \alpha_n\\}\\), for were that to
be the case, then it would follow that \\(\mathcal C(\mathcal S) \subset
\mathcal C(\mathcal S')\\) and \\(\mathcal C(\mathcal S') \subset \mathcal
C(\mathcal S)\\), which trivially implies \\(\mathcal C(\mathcal S) = \mathcal
C(\mathcal S') \\), and subsequently that \\(\mathcal S = \mathcal S'\\). This
is to say that every concept is uniquely defined by the union of its parts, and
if two concepts are composed of exactly the same elements, then they are in fact
the same concept. Applying this to our statement, we can say that there
exists a concept of our subject, Washington, \\(\mathcal C(W)\\), composed
of \\(n\\) \\(\alpha\\)-elements, one of which takes form \\(\alpha_k =
\text{crossed the Delaware}\\), with \\(1 < k < n\\). This is to say that

\\[
\mathcal C(W) = \\{\alpha_1, \alpha_2, \ldots, \alpha_k, \ldots, \alpha_n\\}
\\]

And also that, fundamentally, \\(\text{Washington is Washington}\\) if, and only
if

\\[
\mathcal C(W) = \bigcup_\{i=1\}^{n}\alpha_i
\\]

Let us now suppose that Washington did *not* cross the Delaware. This would mean
we now have a concept, which we'll denote
\\(\mathcal C(W')= \\{\beta_1, \ldots, \beta\_\{n-1\}\\}\\) where

\\[
(\forall \alpha_i\in \mathcal C(W)\setminus\\{\alpha_k\\})
(\exists\beta_j\in\mathcal C(W'))
\text{ s.t. }\alpha_i = \beta_j
\\]

In other words, we have a concept \\(\mathcal C(W')\\) that contains all
elements of \\(\mathcal C(W)\\) *except* \\(\alpha_k\\). This, however, implies
that while \\(\mathcal C(W') \subset \mathcal C(W)\\), the converse isn't true,
and \\(\mathcal C(W) \not\subset \mathcal C(W')\\). Since we had previously
defined that the only way for two subject-concepts to be the same, namely
\\(\mathcal C(\mathcal S) = \mathcal C(\mathcal S') \iff \mathcal S = \mathcal
S'\\), is for them to mutually contain each other, we conclude that
\\(\mathcal C(W) \neq \mathcal C(W')\\), and most importantly

\\[
W \neq W'
\\]

Fundamentally this means that our imaginary idea of a Washington that did not
cross the Delaware is, in fact, not an idea of Washington at all. It is a
separate concept, without equivalence. Furthermore, Leibniz will argue that
\\(\mathcal C(W')\\) would not even make sense, because for one predicate to be
lost from the subject-concept, an infinite number of other predicates must be
changed, to the point where the concept has changed entirely.

Moreover, we can see that there is a set of concepts

\\[
\Omega = \mathcal P(\mathcal C(W))\setminus\mathcal C(W)
\\]

Which constitutes all the "lesser forms of Washington", meaning concepts which
are proper subsets of the main \\(\mathcal C(W)\\). According to Leibniz, the
only thing that separates these lesser forms to the Washington we know is that
God only gave existence to one such concept (\\(\mathcal C(W)\\)). This shows us
something curious, which is that every concept that exists is the largest (in
terms of the number of predicates that compose it) that it could be. In other
words, a concept is defined by the set of its characteristics, and any proper
subset of it defines something which is not the concept itself, formally:

\\[
(\forall \omega \in \Omega)(|\omega| < |\mathcal C(W)|)
\\]

Finally, we conclude that it follows from the tautological statement
\\(``\mathcal S\text{ is }\mathcal S"\\) that any change in the composition of
the subject-concept yields a new concept which is not the original, and
therefore statements are only true insofar as their predicate is part of the
composition of the subject-concept, for otherwise we violate the Law of
Identity. In other words,

\\[
\qquad\qquad\mathcal S + \phi \text{ is true }\iff \phi\in\mathcal C(\mathcal S)\qquad\qquad_\blacksquare
\\]
