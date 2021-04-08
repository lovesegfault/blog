+++
title = "C Arrays Are Not Pointers"
date = 2021-04-01T22:04:19-07:00
tags = [ "arrays", "pointers", "C", "programming" ]
categories = [ "essay" ]
+++

One of the biggest traps C lays for beginners is its idiosyncratic and confusing
treatment of arrays. I often times see people learn, or conclude themselves,
that arrays in C are just pointers, which is not the case. In C arrays are not
pointers, and in this post I want to go over the main reasons why that is the
case.[^1]

## 1. `sizeof()`

This is the first situation that caused me to see how arrays and pointers are
really distinct. Consider the following code:

```C
#include<stdio.h>
#include<stdlib.h>

int main(void) {
    int *ptr = calloc(5, sizeof(int));
    int arr[5] = {0};
    printf("sizeof(ptr) = %ld\n", sizeof(ptr));
    printf("sizeof(arr) = %ld\n", sizeof(arr));
}
```

First, let's understand what `sizeof()` is _supposed_ to do, from the [ISO C
Standard][cstd]:

> **6.5.3.4, item 2**
>
> The `sizeof` operator  yields  the  size  (in  bytes)  of  its  operand,
> which  may  be  an expression or the parenthesized name of a type.

So, given that I'm on a 64-bit system, we'd expect to see the following output,
were arrays and pointers to be the same:

```text
sizeof(ptr) = 8
sizeof(arr) = 8
```

The output is the number of _bytes_, so it'd be indicating they are both 64-bit
addresses, all golden, right? Well, no, this is what you actually see:

```text
sizeof(ptr) = 8
sizeof(arr) = 20
```

wat?

Alright, let's go take a gander at the standard again

> **6.2.5 item 20**
>
> An array type describes a contiguously allocated nonempty set of objects with
> a particular member object type, called the element type. Array types are
> characterized by their element type **and by the number of elements in the
> array**. An array type is said to be derived from its element type, and if its
> element type is T, the array type is sometimes called "array of T". The
> construction of an array type from an element type is called "array type
> derivation". *[Emphasis mine]*

The first clue lies here, where we already learn that arrays are decidedly
**not** pointers! Pointers have no relationship, from the compiler's point of
view, to the length of the data they point to. It's up to you, the programmer,
to keep track of how large the object you are pointing to is. Arrays, on the
other hand, are _characterized_ by their length. The type of a given "array of
T" is really "array of T with length N."[^2] There can be no array without a known
length[^3], in stark contrast to pointers, where the compiler never knows their
length.

> **6.5.3.4 item 4**
>
> When `sizeof` is applied to an operand that has type `char`, `unsigned char`, or
> `signed char`, (or a qualified version thereof) the result is 1. **When applied to
> an operand that has array type, the result is the total number of bytes in the
> array**. When applied to an operand that has structure or union type, the result
> is the total number of bytes in such an object, including internal and trailing
> padding. *[Emphasis mine]*

And there's the final clue, `sizeof()` has special behavior for arrays[^4]! For
an array it will look at the `sizeof()` the object type, the length of the
array, and compute the total length. For example, if I have `int foo[5];` and I
do `sizeof(foo)` I will hit this special behavior, and get the size as being
`sizeof(int) * 5 = 4 * 5 = 20`.

With this in mind our initial example starts making more sense, let's look at it
again:

```C
int *ptr = calloc(5, sizeof(int));
int arr[5] = {0};
printf("sizeof(ptr) = %ld\n", sizeof(ptr));
printf("sizeof(arr) = %ld\n", sizeof(arr));
```

And we got

```text
sizeof(ptr) = 8
sizeof(arr) = 20
```

The first `sizeof()` hits the normal behavior and evaluates to the size of the
pointer itself (not the data it points to), which in x86_64 is 8 bytes. The
second `sizeof()` hits the special behavior for arrays, and computes the total
length of `arr` in bytes, which is `20`.

## 2. Function Arguments

Consider the following example:

```C
#include<stdio.h>
#include<stdlib.h>

// Prints an array
void print_arr(int *arr, size_t len) {
    printf("arr[ ");
    for(size_t i = 0; i < len; ++i) {
        printf("%d ", arr[i]);
    }
    printf("]\n");
}

// Fills an array with random ints
void randomize(int arr[]) {
    // First find the length of the array
    size_t len = sizeof(arr) / sizeof(int);
    // For clarity's sake
    printf("(randomize) len = %zu\n", len);
    // Now let's fill the array with random values
    for(size_t i = 0; i < len; ++i) {
        arr[i] = rand();
    }
}

int main(void) {
    // Initialize the seed.
    sranddev();
    // 0-initialize it.
    int arr[10] = {0};
    // Let's randomize it!
    size_t len = sizeof(arr) / sizeof(int);
    printf("(main) len = %zu\n", len);
    randomize(arr);
    // And now let's see what happened.
    print_arr(arr, 10);
}
```

We have two simple functions, `print_arr()` and `randomize()`, we initialize an
array of length 10 with zeroes, compute and print it's length, call
`randomize()` on it, and finally print it.

We'd hope to see output that looks like this:

```text
(main) len = 10
(randomize) len = 10
arr[ 2834979 0827650 48721364 8723 73 427360 4 0297346 72 9273 ]
```

We never modify the length of our data, so we see length as the same in
`main()` and `randomize()`, and we successfully fill our array with random data
and print it. Great!

Or it would be, if that code worked at all. If you were to actually run that
snippet this is what you'd get:

```text
(main) len = 10
(randomize) len = 2
arr[ 987922591 1583865774 0 0 0 0 0 0 0 0 ]
```

What? How did the length change? And why is only the beginning of the array
getting filled up with data? What the hell is going on here! Let's go back to
the standard for a moment:

> **6.7.5.3 item 7**
>
> **A declaration of a parameter as "array of type" shall be adjusted to
> "qualified pointer to type"**, where the type qualifiers (if any) are those
> specified within the `[` and `]` of the array type derivation. If the keyword
> static also appears within the `[` and `]` of the array type derivation, then
> for each call to the function, the value of the corresponding actual argument
> shall provide access to the first element of an array with at least as many
> elements as specified by the size expression. _[Emphasis mine]_

As if things weren't confusing enough, the standard defines that whenever you
declare a parameter to a function as an array, such as `void randomize(int
arr[])`, it is automatically "adjusted" to a pointer such that you end up with,
for example, `void randomize(int *arr)`. So even though arrays are not pointers,
when you put an array in argument position, you are actually writing a pointer
as argument! Sometimes you'll hear people refer to this as the array being
"demoted" to a pointer.

This means that whenever you have a function taking in an array as argument, you
fall into the `sizeof()` issue we saw in the previous section. In the caller
scope, `sizeof(arr)` will evaluate to the total length in bytes of the array,
whereas in the function scope `sizeof(arr)` will evaluate to the length in bytes
of a pointer.

With this in mind, let's revisit the output we got:

```text
(main) len = 10
(randomize) len = 2
arr[ 987922591 1583865774 0 0 0 0 0 0 0 0 ]
```

Recall that we had `size_t len = sizeof(arr) / sizeof(int)`, and `sizeof(int) ==
4`.

So, the length in the caller scope is `10` because `sizeof(arr)` computes the
total length of our array, which is `40`, and `40 / 4 == 10`. On the scope of
the `randomize()` function though, `arr` is not an array, despite looking like
it in the function signature, but a _pointer_. Due to this we get `sizeof(arr)
== 8` and `8 / 4 == 2`. Finally, because this caused us to computer `len`
incorrectly, we only actually randomize the first two elements of the array,
which is what we see in the output.

To reiterate, whenever you have `someType my_function(myType my_arg[])` it may look like
you have an argument `my_arg` of type "array of `myType`", but that is an
illusion, you will actually end up with a pointer to `myType`. Personally, I
find the usage of `[]` in function arguments misleading, and that you're always
better off by just making it clear that the input is a pointer.

Luckily, nowadays we have smart compilers that will yield warnings in case you
try to do something like this, as I was building this example locally I got the
following warning[^5]:

```text
sizeof.c:14:24: warning: sizeof on array function parameter will return size of 'int *' instead of 'int []' [-Wsizeof-array-argument]
    size_t len = sizeof(arr)/sizeof(int);
                       ^
sizeof.c:12:20: note: declared here
void randomize(int arr[]) {
                   ^
```

This is trying to tell us "Hey, this `sizeof(arr)` call that you think is
returning the size of an array, will actually return the type of a pointer, and
you should wise up." Despite these warnings, this issue with automatic demotion
of arrays to pointers when crossing scopes is something I see beginners trip on
often.

Not incidentally, the `print_arr()` function shows the correct way to pass an
array to a function; you have to pass the array's length alongside a pointer to
the first element of the array.

## 3. Provenance

If you've ever learned that pointers in C are just numeric values, maybe
representing an address in memory, then you may want to sit down. In response to
[Defect Report 260][dr260] (DR-260) the Committe says:

> If two objects have identical bit-pattern representations and their types are
> the same they may still compare as unequal (for example if one object has an
> indeterminate value) and if one is an indeterminate value attempting to read
> such an object invokes undefined behavior. Implementations are permitted to
> track the origins of a bit-pattern and treat those representing an
> indeterminate value as distinct from those representing a determined value.
> **They may also treat pointers based on different origins as distinct even
> though they are bitwise identical.** _[Emphasis mine]_

This answer brings forth the idea of **pointer provenance**, that pointers are
characterized not only by their value, but also by their origin. Pointers with
identical numerical values, but distinct origins, can still be different.

It is no surprise, then, that pointers created by passing an array to a
function, like we saw in the previous section, and pointers created by, for
example, `malloc()` have different provenances. The standard touches on this
indirectly when they talk about library functions that take an array as
argument:

> **7.1.4 item 1**
>
> If a function argument is described as being an array, the pointer actually
> passed to the function shall have a value such that all address computations
> and accesses to objects (that would be valid if the pointer did point to the
> first element of such an array) are in fact valid.

Now, this isn't _precisely_ the same issue that motivates DR-260, but
nonetheless it alludes to the fact that there is a semantic difference between a
function that takes a pointer as argument and one that takes an array. There are
different expectations in place.

There is a lot more to be said about provenance beyond the shallow point I'm
making here, I recommend anyone interest take a look at ["n2263: Clarifying
Pointer Provenance v4"][ptrprov] for an in-depth look into the issues that arise
from pointer provenance and the proposed changes to the standard.

## Conclusion

So there you have it folks, arrays are definitely not pointers and now you know
way too much about why!

If you believe I missed something in this post, please feel free to mention it
in the comments bellow, or reach me at
[bernardo@arraysarenotpointers.dev](mail).

[^1]: This post is aimed at beginners and, to a lesser degree, intermediate
  users of C. If you are an expert you are unlikely to be surprised by what I
  show here but, hey, maybe you realize I missed something and help me improve
  the list :)
[^2]: You'll commonly see this type of data structure, a pointer to the first
    element together with the length of the data, be referred to as a _fat
    pointer_. It's called that because it's larger than a normal pointer, since
    it needs to contain the length information too.
[^3]: Okay, I'm lying! There are these things called variable length arrays,
    VLAs, that don't have a length known at compile time. The standard lays out
    some special behavior for them, for example, while `sizeof(x)` is usually
    guaranteed to not evaluate `x` and be done at compile time, that isn't the
    case with VLAs. If `x` is a VLA then it's evaluated at runtime and the
    length computed. See **6.5.3.4 item 2**.
[^4]: For VLAs (Variable Length Arrays) the standard specifies similar behavior
    to arrays, with the additional complications coming from their
    runtime-determined sizes.
[^5]: That we now have warnings that are this easy to read and grasp in C is
    absolutely amazing, and a crucial effort that often goes overlooked.

[cstd]: http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1548.pdf
[dr260]: http://www.open-std.org/jtc1/sc22/wg14/www/docs/dr_260.htm
[ptrprov]: http://www.open-std.org/jtc1/sc22/wg14/www/docs/n2263.htm
[mail]: mailto://bernardo@arraysarenotpointers.dev
