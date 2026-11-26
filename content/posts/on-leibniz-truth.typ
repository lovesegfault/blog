#import "/blog.typ": *

#show: post.with(
  title: [On Leibniz's Truth],
  date: "2017-10-25T00:00:00Z",
  categories: "essay",
  tags: ("philosophy", "logic", "leibniz", "theory"),
)

Let a _statement_ be any sentence of form

$ cal(S) + phi.alt $

Where $cal(S)$ is a _subject_ and $phi.alt$ is a _predicate_. The sentences
"Spinoza died in the Hague", and "Grey is grey", for example, qualify as
statements. In the first, we have "Spinoza" as the subject, and "died in the
Hague" as the predicate. Note that, for a sentence to qualify as a statement,
its predicate does not need to contain an object (consider the sentence
$X "is."$) Statements are said to be _true_ if and only if "the
_concept-predicate_ is contained in the _concept-subject_" (Leiniz's
Concept-Containment Theory of Truth).

$ cal(S) + phi.alt "is true" <==> phi.alt in cal(C)(cal(S)) $

Where here we use $cal(C)(cal(S))$ to denote the _concept-subject_, or _the
concept of $cal(S)$_. Trivial examples of concept-containment are tautological
statements such as

$ X "is" X $

Statements like this are clearly true, and the concept-containment is evident as
well, since it follows from the Law of Identity that the concept of $X$ is
contained in $X$, in other words, tautological statement are such that the
subject-concept and the predicate-concept are the same. These types of
statements are called _Primary Truths_.

Let us now consider for a moment the following statement:

$ "Washington crossed the Delaware" $

Unlike the identity statement we saw above, here the subject-concept is _not_
the same as the predicate-concept, and most importantly there is no clear
containment of the latter into the former, which is why we call it a _Secondary
Truth_. One can, for example, easily imagine Washington _not_ having crossed the
Delaware, while on the other hand attempting to imagine $X "is not" X$ is
fruitless at best (and maddening at worst). In a sense, it seems that while all
primary truths are "self sufficient", and depend on no external factors to be
true, the same is not the case for secondary truths, they seem to depend on
large class of externalites. For Washington to have crossed the Delaware, for
example, a number of dependencies must have been satisfied, namely, Washington
must exist, and so must the Delaware, and the Earth on which they act, and so
on. It seems to us that the statement could well be false, without any dooming
consequences, once again, it does not take a lot of mental gymnastics to imagine
a universe where Washington simply did not cross the Delaware. The solution to
this problem lies in the fact that all truths are what we call _analytic_,
meaning they can be reduced to primary truths, which in turn are trivially true.

Let us begin by acknowledging Washington as being more than an individual, we
are referring here to a _concept_ of Washington. This concept, we must note, is
not atomic, and can be divided into a set of elements that compose it, which is
to say we have

$
  cal(C)(cal(S)) = {alpha_1, alpha_2, dots, alpha_n},
  quad n in NN union {+infinity}
$

Denoting a concept-subject as being composed of $n$ characteristics. We now
note, however, that any such $cal(C)(cal(S))$ is unique, that is to say that
there can be no subject $cal(S)'$ such that
$cal(C)(cal(S)') = {alpha_1, alpha_2, dots, alpha_n}$, for were that to be the
case, then it would follow that $cal(C)(cal(S)) subset cal(C)(cal(S)')$ and
$cal(C)(cal(S)') subset cal(C)(cal(S))$, which trivially implies
$cal(C)(cal(S)) = cal(C)(cal(S)')$, and subsequently that $cal(S) = cal(S)'$.
This is to say that every concept is uniquely defined by the union of its parts,
and if two concepts are composed of exactly the same elements, then they are in
fact the same concept. Applying this to our statement, we can say that there
exists a concept of our subject, Washington, $cal(C)(W)$, composed of $n$
$alpha$-elements, one of which takes form $alpha_k = "crossed the Delaware"$,
with $1 < k < n$. This is to say that

$ cal(C)(W) = {alpha_1, alpha_2, dots, alpha_k, dots, alpha_n} $

And also that, fundamentally, $"Washington is Washington"$ if, and only if

$ cal(C)(W) = union.big_(i = 1)^n alpha_i $

Let us now suppose that Washington did _not_ cross the Delaware. This would mean
we now have a concept, which we'll denote
$cal(C)(W') = {beta_1, dots, beta_(n - 1)}$ where

$
  (forall alpha_i in cal(C)(W) without {alpha_k})
  (exists beta_j in cal(C)(W')) "s.t." alpha_i = beta_j
$

In other words, we have a concept $cal(C)(W')$ that contains all elements of
$cal(C)(W)$ _except_ $alpha_k$. This, however, implies that while
$cal(C)(W') subset cal(C)(W)$, the converse isn't true, and
$cal(C)(W) subset.not cal(C)(W')$. Since we had previously defined that the only
way for two subject-concepts to be the same, namely
$cal(C)(cal(S)) = cal(C)(cal(S)') <==> cal(S) = cal(S)'$, is for them to
mutually contain each other, we conclude that $cal(C)(W) != cal(C)(W')$, and
most importantly

$ W != W' $

Fundamentally this means that our imaginary idea of a Washington that did not
cross the Delaware is, in fact, not an idea of Washington at all. It is a
separate concept, without equivalence. Furthermore, Leibniz will argue that
$cal(C)(W')$ would not even make sense, because for one predicate to be lost
from the subject-concept, an infinite number of other predicates must be
changed, to the point where the concept has changed entirely.

Moreover, we can see that there is a set of concepts

$ Omega = cal(P)(cal(C)(W)) without cal(C)(W) $

Which constitutes all the "lesser forms of Washington", meaning concepts which
are proper subsets of the main $cal(C)(W)$. According to Leibniz, the only thing
that separates these lesser forms to the Washington we know is that God only
gave existence to one such concept ($cal(C)(W)$). This shows us something
curious, which is that every concept that exists is the largest (in terms of the
number of predicates that compose it) that it could be. In other words, a
concept is defined by the set of its characteristics, and any proper subset of
it defines something which is not the concept itself, formally:

$ (forall omega in Omega) (abs(omega) < abs(cal(C)(W))) $

Finally, we conclude that it follows from the tautological statement
$“ cal(S) "is" cal(S) ”$ that any change in the composition of the
subject-concept yields a new concept which is not the original, and therefore
statements are only true insofar as their predicate is part of the composition
of the subject-concept, for otherwise we violate the Law of Identity. In other
words,

$
  wide wide cal(S) + phi.alt "is true" <==> phi.alt in cal(C)(cal(S))
  wide wide square.filled
$
