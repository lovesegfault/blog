+++
title = "Arrays Are Not Pointers"
date = 2021-04-01T22:04:19-07:00
tags = [ "arrays", "pointers", "C", "programming" ]
categories = [ "essay" ]
draft = true
+++

One of the biggest traps people who start learning C fall into is

I've know C, and in particular C99, for quite a while now. I've finally reached
the point where I've had more years of my life knowing (at least some) C than
not. With this in mind, and to commemorate the language that got me into
programming, and which I am still extremely fond of, I want to talk about what I
think is one of the biggest misconceptions in people's knowledge of C: the
difference between an array and a pointer. Having been a TA for a couple of C
courses, and having had to attend C courses, I could not believe how many
professors will teach this as a "useful simplification." So, let's start by
stating the truth - arrays are not pointers. Not at all. In this post I want to
explore, with examples, the three main differences between arrays and pointers
in C.

## 1. `sizeof()` Implementation

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
T" is really "array of T with length N."[^1] There can be no array without a known
length[^2], in stark contrast to pointers, where the compiler never knows their
length.

> **6.5.3.4 item 4**
>
> When `sizeof` is applied to an operand that has type `char`, `unsigned char`, or
> `signed char`, (or a qualified version thereof) the result is 1. **When applied to
> an operand that has array type, the result is the total number of bytes in the
> array**. When applied to an operand that has structure or union type, the result
> is the total number of bytes in such an object, including internal and trailing
> padding. *[Emphasis mine]*

And there's the final clue, `sizeof()` has special behavior for arrays[^3]! For
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

## 2. Crossing scopes

Consider the following example:

```C
#include<stdio.h>
#include<stdlib.h>

// Prints an array
void print_arr(int arr[], size_t len) {
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
    // Initialize the seed
    sranddev();
    // Suppose we get this guy from user input, or some file, or w/e.
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

So, when we pass `arr` to the `randomize()` function, it is actually _demoted_
to a regular pointer (`int*` in this case) and thus _loses_ it's length
information. It's as if C arrays are _fat pointers_[^3], but only the inner,
"thin," pointer actually gets passed when you use it as a function argument.

Since the length of an `int*` is 8 bytes, and the length of an `int` is 4 bytes,
we have the computed length equal to `2`, and thus only the first two elements
of the array are initialized. This is a not-too-bad manifestation of this issue,
consider for a moment what could happen if the array had length 1.

Luckily nowadays we have smart compilers that will yield warnings in case you
try to do something like this, namely as I was building this example locally I
got the following

```text
sizeof.c:14:24: warning: sizeof on array function parameter will return size of 'int *' instead of 'int []' [-Wsizeof-array-argument]
    size_t len = sizeof(arr)/sizeof(int);
                       ^
sizeof.c:12:20: note: declared here
void randomize(int arr[]) {
                   ^
```

This very precisely tells us we're about to do something that is most likely
incorrect[^4], but nonetheless this can be quite annoying to beginners.

Not incidentally, the `print_arr()` function shows the correct way to do this
kind of thing; you really have to pass the length[^5] along with the "array."

> **7.1.4 item 1**
>
> If a function argument is described as being an array,the pointer actually
> passed to the function shall have a value such that all address computations
> and accesses to objects (that would be valid if the pointer did point to the
> first element of such an array) are in fact valid.

[^1]: You'll commonly see this type of data structure, a pointer to the first
    element together with the length of the data, be referred to as a _fat
    pointer_. It's called that because it's larger than a normal pointer, since
    it needs to contain the length information too.
[^2]: Okay, I'm lying! There are these things called variable length arrays,
    VLAs, that don't have a length known at compile time. The standard lays out
    some special behavior for them, for example, while `sizeof(x)` is usually
    guaranteed to not evaluate `x` and be done at compile time, that isn't the
    case with VLAs. If `x` is a VLA then it's evaluated at runtime and the
    length computed. See **6.5.3.4 item 2**.
[^3]: For VLAs (Variable Length Arrays) the standard specifies the similar
    behavior to arrays, with the additional complications coming from their
    runtime-determined sizes.
[^4]: That we now have warnings that are this easy to read and grasp in C is
    absolutely amazing, and a crucial effort that often goes overlooked.
[^5]: Note the use of `size_t` for the lengths, if you didn't know about it, you
    should check it out https://man7.org/linux/man-pages/man7/system_data_types.7.html.

[cstd]: http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1548.pdf
