# Totalisation

## Overview

A function is _total_ iff it is _defined_ for any argument in its domain. In MiniZinc, the domain of a function is determined by the _type-insts_ of its arguments, but not by any _constraints_ on its arguments.

A function is _partial_ iff it is _undefined_ for certain arguments in its domain. In MiniZinc, a function is partial if its return type is not a subtype of `var opt bool`, and at least one of these conditions holds:

- If at least one of its arguments has a constrained type. E.g., `function int: foo(1..3: x) = x + 1;` is partial, because its argument `x` is constrained.
- If its body is a partially defined expression
- If it is a built-in function that is known to be partial (e.g. integer division, modulo etc.)

A function that is partial according to the rules above can be annotated as `::promise_total` if it can be guaranteed that it will return a value for any input in its domain.

An expression is _partially defined_ if its type is not a subtype of `var opt bool` and it does not evaluate to a value for all possible evaluations of its free variables. At least one of the following conditions needs to hold for an expression to be partially defined:

- It is a call to a partial function or built-in operator
- It is a `let` expression with a type that is not a subtype of `var opt bool`, and one of the cases in the `let` is a `constraint`, or one of the variables in the `let` has a constrained type-inst
- One of its immediate subexpressions is partially defined

## Array types

Two options:

1. An undefined element in an array makes the entire array undefined (MiniZinc 2 semantics)
2. An undefined element in an array only causes undefinedness if the element is accessed (Stuckey/Frisch paper)

Issues with option 2: Passing an array to a function now means that all arrays have to be arrays of tuples (even if all elements are statically known to be defined); or, we have to generate different versions (tuple and non-tuple) for each function, for all array argument types.

## Transformation

The basic idea of the transformation is to turn any partially defined expression into a pair of expressions `(b,e)` where `b` is a Boolean that is true iff the expression is defined, and `e` is the value of the expression if it is defined, and `⊥` otherwise.

1. The return type of a partial function returning type `T` is changed to `tuple(bool, T)`.
2. Any variable declaration of type `T` whose right hand side is a partially defined expression is changed into a variable declaration of type `tuple(bool, T)`. **NOTE:** What about array types? Do they capture definedness for each element, or for the array?
3. Set literals (containing partially defined expressions)
   The type is transformed from `set of T` into `tuple(bool, set of T)` as follows.
   `{ e1, e2, pde1, ..., pdek, ... }` into
   `let { any: (b1,tmp1) = pde1;
      any: (bk,tmpk) = pdek; }
in (forall([b1,...,bk]), {e1, e2, tmp1, ..., tmpk ...})`
4. Array literals (containing partially defined expressions)
   ~~Change type `array[...] of T` into `array[...] of tuple(bool, T)` and change
   `[ e1, ..., pde1, ... pdek, ... en]` into
   `[ (true, e1), ..., pde1, ... pdek, ... (true,en) ]`~~
   Change type `array[...] of T` into `tuple(bool, array [...] of T` and change `[ e1, ..., (b1, pde1), ... (bk, pdek), ... en]` into `(b1 /\ ... /\ bk, [e1, pde1, ... pdek, ... en])`
5. Array comprehensions
   - Partially defined generators are not permitted (static type error).
   - Partially defined generated expressions are fine. The resulting type changes from `array[...] of T` into `tuple(bool, array[...] of T)` (like array literals). May need to create `array [...] of tuple(bool, T)` first, then extract the definedness from that.\*\*\*\*
   - Partially defined where clauses are fine (they are their own Boolean context)
   - These examples show why it would be a bad idea to allow partial generators. - par comprehensions
     `[ pde | i in 0..10, j in x[i] ]`
     `[ pde | i in 0..10, (b,e)=x[i] where b, j in e ]` - var generator comprehensions:
     `var 0..10: x`
     `[ pde | i in 0..10, j in i div x..100]`
     `[ let { any: (b,t) = i div x..100;
        any: (eb, et) = pde } in
  if eb /\ b /\ j in t..100 then (true,et) else (b,<>) endif) |
i in 0..10, j in ub(i div x..100) ]`
6. Set comprehensions
   These are transformed into array comprehensions and `array2set` in a previous compiler phase.
7. Array access  
   `var` array access turns into `element()` which has the right semantics.
   `function tuple(bool, any $T): element(array [int, ...] of any $T: x, int: i, ...)` where the boolean is
8. `if then else endif`  
   Can't use a normal function since the arguments would escape into the boolean context outside, so instead define the function which returns the pair:  
   `function tuple(var bool, var $T): if_then_else(array [int] of var bool: c, array [int] of tuple(var bool, var $T): r)`
9. Tuple field access  
   Same as for arrays, so undefined inside a tuple makes the whole tuple undefined
10. Calls
    Transformed to return a tuple if not known to be total
11. `let { ... } in`
    If not in `::promise_total`/root context and non-boolean then the conjunction of the constraints and the variable definedness becomes the let's definedness
    ```
    let {
        var int: x = e1;
        constraint c;
    } in e2
    ```
    becomes
    ```
    let {
        tuple(var bool, var int): x' = (de1, e1');
        var bool: c' = c;
    } in (de1 /\ c' /\ de2, e2')
    ```
12. Annotations
    Annotations are compiled in the root context.
13. Output
    The expression in an output statement must be total. It is a static type error if it isn't. Users can use `default` to make all expressions total.
