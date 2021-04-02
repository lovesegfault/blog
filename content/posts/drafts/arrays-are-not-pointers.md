+++
title = "Arrays Are Not Pointers"
date = 2021-04-01T22:04:19-07:00
tags = [ "arrays", "pointers", "C", "programming" ]
categories = [ "essay" ]
draft = true
+++

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
Standard][cstd] 6.5.3.4, item 2:

> The `sizeof` operator  yields  the  size  (in  bytes)  of  its  operand,
> which  may  be  an expression or the parenthesized name of a type.

So, given that I'm on a 64-bit system, we'd expect to see the following output,
were arrays and pointers to be the same:

```C
sizeof(ptr) = 8
sizeof(arr) = 8
```

The output is the number of _bytes_, so it'd be indicating they are both a
64-bit address, all golden, right? Well, no, this is what you actually
see:

```C
sizeof(ptr) = 8
sizeof(arr) = 20
```

wat?

The answer, as usual, lies within the standard, in particular there are two
interesting sections regarding this, 6.2.5 item 20 and 6.5.3.4 item 3.

> An array type describes a contiguously allocated nonempty set of objects with
> a particular member object type, called the element type. Array types are
> characterized by their element type **and by the number of elements in the
> array**. An array type is said to be derived from its element type, and if its
> element type is T, the array type is sometimes called "array of T". The
> construction of an array type from an element type is called "array type
> derivation". [Emphasis mine]

So, perhaps unexpectedly, `sizeof()` has special behavior for arrays (it also has
special behavior for VLAs[^1]) and it instead returns the sum of the sizes of the
elements of the array in bytes. While this may seem like a small or unimportant
detail, it is actually the cause of a lot of bugs in beginners' code, because
it means that code like the following does _not_ work as expected.

```C
#include<stdio.h>
#include<stdlib.h>

void print_arr(int arr[], size_t len) {
    printf("arr[ ");
    for(size_t i = 0; i < len; ++i) {
        printf("%d ", arr[i]);
    }
    printf("]\n");
}

void randomize(int arr[]) {
    // First find the length of the array
    size_t len = sizeof(arr) / sizeof(int);
    // For clarity's sake
    printf(">>>> len = %zu\n", len);
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
    printf(">>>> len = %zu\n", len);
    randomize(arr);
    // And now let's see what happened.
    print_arr(arr, 10);
}
```

```C
>>>> len = 10
>>>> len = 2
arr[ 987922591 1583865774 0 0 0 0 0 0 0 0 ]
```

What is happening here? Let's go back to the standard for a moment:

[INSERT CITATION/QUOTE]

So, when we pass `arr` to the `randomize()` function, it is actually _demoted_
to a regular pointer (`int*` in this case) and thus _loses_ it's length
information. It's as if C arrays are _fat pointers_[^2], but only the inner,
"thin," pointer actually gets passed when you use it as a function argument.

Since the length of an `int*` is 8 bytes, and the length of an `int` is 4 bytes,
we have the computed length equal to `2`, and thus only the first two elements
of the array are initialized. This is a not-too-bad manifestation of this issue,
consider for a moment what could happen if the array had length 1.

Luckily nowadays we have smart compilers that will yield warnings in case you
try to do something like this, namely as I was building this example locally I
got the following

```
sizeof.c:14:24: warning: sizeof on array function parameter will return size of 'int *' instead of 'int []' [-Wsizeof-array-argument]
    size_t len = sizeof(arr)/sizeof(int);
                       ^
sizeof.c:12:20: note: declared here
void randomize(int arr[]) {
                   ^
```

This very precisely tells us we're about to do something that is most likely
incorrect[^3], but nonetheless this can be quite annoying to beginners.

Not incidentally, the `print_arr()` function shows the correct way to do this
kind of thing; you really have to pass the length[^4] along with the "array."

[^1]: For VLAs (Variable Length Arrays) the standard specifies the same behavior
    as for arrays, with the additional complications coming from their
    runtime-determined sizes.
[^2]: A "fat pointer" is lingo for a structure containing both a pointer and its
    length, basically.
[^3]: That we now have warnings that are this easy to read and grasp in C is
    absolutely amazing, and a crucial effort that often goes overlooked.
[^4]: Note the use of `size_t` for the lengths, if you didn't know about it, you
    should check it out[INSERT LINK].

[cstd]: http://www.open-std.org/jtc1/sc22/wg14/www/docs/n1256.pdf
