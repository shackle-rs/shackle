# Type checking

The type-checker performs bottom-up typing of expressions, and checks that the types are correct. This is performed on a
per-item basis. The type-checker deals with 'signatures' and 'bodies' of items separately. Signatures give the type
information needed to compute the type of an expression referring to the item. Typing a body involves typing the rest of
the item (generally this is the annotations and the RHS).

Consider the items:

```mzn
function float: foo(int: x :: my_ann) = x + 0.5;
1..3: a = 23;
any: b = foo(a);
```

- `foo` has a signature of `function float: foo(int)`. Its RHS of `x + 0.5` is typed separately and verified against the
  signature return type, along with the annotation `my_ann`.
- `a` has a signature of `int: a`, which is computed using only the LHS of the declaration as the type is complete. The
  RHS of `23` is typed separately and verified against the signature type.
- `b` uses an `any` type and so its signature of `float: b` is computed using the RHS.

## Type checker results

Type checking an item produces several maps containing the computed types. The `TypeResult` provides access to all types
for an item, by using the signature types and the body types as needed. This can be obtained using the
`db.lookup_item_types(item_ref)` query.

- Indexing using an `ArenaIndex<Expression>` gives the `Ty` of the expression.
  - E.g. the expression `{1, 1.5}` will have type `set of float`.
- Indexing using an `ArenaIndex<Pattern>` gives the `PatternTy` of the pattern.
  - The `PatternTy` includes information about what the pattern is being used for, rather than just the type of the
    pattern itself (e.g. so we can tell if a pattern is simply for destructuring, or if it actually declares a new
    variable).
  - E.g. the declaration `any: (x, y) = (1, 1.5)` will be such that
    - `(x, y)` is a `PatternTy::Destructuring(tuple(int, float))`
    - `x` is a `PatternTy::Variable(int)`
    - `y` is a `PatternTy::Variable(float)`
- The `name_resolution(ident)` method finds the `PatternRef` for an expression which is an identifier (e.g. identifier
  pointing to a variable).
- The `pattern_resolution(ident)` method finds the `PatternRef` for a pattern which is an identifier (e.g. enum atom
  used in a case expression pattern)

## Expressions

Typing of expressions is done recursively, using the types of child expressions
to determine the type of the outer expression.

For example, the expression `([1, 2.5], 3)` will be typed by:

- The first expression is a tuple, so visit its child fields
  - The first tuple field is an array literal so visit the child members
    - The first child member is an integer literal, so its type is `int`
    - The second child member is a float literal, so its type is `float`
  - Therefore the array literal is determined to have type
    `array [int] of float` as float is the most specific supertype of `int` and
    `float`.
  - The second tuple field is an integer literal, so its type is `int`
- Therefore the tuple is of type `tuple(array [int] of float, int)`

### Calls

One exception to bottom-up typing occurs when dealing with calls, as in order to perform overloading resolution, we
must specially handle the case of a call with an identifier callee. See the following section on function resolution
for more detail.

For example, the call `foo(1, "a")` would be typed by:

- Getting the types of each argument (in this case `int` and `string`)
- Finding all the function items named `foo`
- Performing overloading resolution given the argument types
- `foo` is given the type of the operation
- The call is given the return type of the operation

If the call does not have an identifier as its callee, the callee type is determined in the usual bottom-up fashion, and
no overloading resolution is required.

### Identifiers

Identifier expressions cause the type checker to lookup the identifier in the expression's [scope](./scope.md), and
fetch the type signature of the retrieve pattern's item to determine the type of the identifier. For bodies, we can
fetch the signature of the item directly (and reuse the computation of that signature if it's already done), but for
signatures which depend on other signatures, we do not do this because of the possibility of cycles.

Instead, the signatures of the dependencies are always computed, and if we reach the initiating signature during this
process, we can break out of the cycle and return the error type (at that stage, not at the initiating site). Any
cyclic definition errors will be emitted later during topological sorting. This is important for dealing with function
signatures that include calls to other overloads of the function, since these should work, but resolving the overloaded
call would require the type of 'this' signature, which is in the middle of being computed.

## Ascribed types

Typing user-written types (the HIR `Type`) involves computing the concrete type (`Ty`) they represent. For example, the
type `var 1..3` is typed by computing the type of the domain `1..3`. Since it is a `set of int`, then the variable is an
`int` type, and since the type has the `var` modifier, the complete type is computed to be `var int`.

Typing these becomes more complex when dealing with incomplete types. In these cases, we always have a RHS expression
which can be typed first, and then we can use this computed type to fill in the 'holes' in the ascribed type.

For example, consider the (contrived) declaration:

```mzn
array [_] of var set of any: x = [{1, 2}];
```

In this case, we start by computing the type of the RHS, and determine it to be of type `array [int] of set of int`.
This type is unified with the ascribed type to obtain the type `array [int] of var set of int` for `x`.

## Patterns

Typing of patterns is done top-down, based on an already computed type that the pattern must take. For example, to type
the declaration

```mzn
tuple(var int, any): (a, b) = (1, 2.5);
```

- The concrete type of the declaration is `tuple(var int, float)`
  (found by computing the type of the RHS and using it to fill in hole in the
  LHS type)
- Therefore, the root pattern `(a, b)` has type `tuple(var int, float)`.
- We then visit the child patterns and set their type to the corresponding tuple
  field type.
- So `a` has type `var int`
- And `b` has type `float`

One complication is that when dealing with constructor calls in patterns, we only know the 'return type' of the call, as
we know the type of the call pattern itself, but not its arguments. We use this return type to determine the what the
actual constructor call must be. For example:

```mzn
enum Foo = A(Bar) ++ {B, C};
enum Bar = {D, E};
var Foo: x;
any: y = case x of
    A(v) => v,
    _ => E
endcase;
```

- The type of `x` is `var Foo`
- Therefore the type of the first case pattern `A(v)` is also `var Foo`
  - So based on that, we must be using the constructor `var Foo: A(var Bar)`, and so this is the type of the `A` pattern
  - The type of `v` is therefore `var Bar`
  - So the result for the first case arm is `var Bar`
- The second case pattern `_` should also match the type of `x` (`var Bar`)
  - Its result type is `E`, which is the type `Bar`
  - So the case expression is of type `var Bar`

## Function resolution

Function resolution involves determining a call identifier and a list of argument `Ty`s, the best matching overload to
use (if there is one). The algorithm used is:

- If there is a variable which is an `op` function with the correct name, use it.
- If not, then find the function items in scope with the correct name.
- Remove any candidates which have an incorrect number of arguments.
- Remove any candidates which cannot be instantiated with the argument `Ty`s
- For every pair of candidates `a`, and `b` remaining, eliminate the 'less specific' function
  - Instantiate `a` and `b` with our call's argument types
  - If the instantiated argument types of `a` are all subtypes of the instantiated argument types of `b`, but not the
    other way around, then eliminate `b` (and vice-versa).
  - If both instantiated argument types are equivalent, then
    - If `a` is polymorphic but `b` is a concrete function, then eliminate `a` (and vice-versa)
    - If both candidates are polymorphic, then
      - If `a` can be instantiated with `b`'s (original) parameter types, but not the other way around, eliminate `a`
        (and vice-versa)
    - If `a` has a body but `b` doesn't, eliminate `b` (and vice-versa)
  - Otherwise arbitrarily eliminate `a`

Essentially, we choose such that:

- More specific parameter types are preferred: `bool` over `int`, `int` over `var int`. We take the instantiated
  argument types into account, so a function taking `$$E` will be preferred over a function taking `float` if called
  with an `int` argument.
- Where two candidates would instantiate to the same types, prefer `$$E` functions over `$T` functions, and prefer
  concrete types over either of those.

It is an error if we are left with more than one candidate, or no candidates at the end of the process. Additionally,
it should be noted that in the case of two identical functions which have bodies, we will simply arbitrarily choose one
at this stage to allow type checking to continue with a reasonable return type. The duplicate function error will be
emitted during final validation of the HIR.

### Type-inst variables

There are two kinds of type-inst variables: `$T` and `$$E`. `$T` matches any type other than arrays and functions (this
is because the old compiler doesn't accept arrays for these, but actually it seems like it would make more sense if it
did). `$$E` matches any enumerable type (booleans, integers, enums) - unlike the old compiler, this is a a properly
generic type parameter, and is not considered to a special `int`.

It's also possible to restrict `$T` to accept only types valid for array indices if it is used as `array [$T] of int`.
In this case only integers/enums or tuples of them would be accepted.

One complication is that the way type-inst variables are used looks different to most other languages. Since omitting
`var`/`opt` implies `par`, non-`opt`, a parameter declared to be `$T` is actually `par (non-opt) $T`. That is, the
parameter will accept the par, non-optional version of whatever type the type parameter `$T` is given as. `any $T` is
used to make the parameter accept the actual type of the type parameter `$T`.

Note that the language is currently missing `anyvar` and `anyopt`, which would be needed for expressing parameters
whose inst/optionality depend on the input type parameters (for type-inst variables used in parameters only, this is
not really a problem since using `var $T` or `opt $T` will still let you match the needed combinations - the problem
is that if you use the type-inst variable in the return type, specifying `opt $T` will force the return type to be
`opt`, even if `$T` is non-optional).

#### Determining the types substituted for type-inst variables

Since calls do not explicitly set the types to give for each type-inst variable, we have to use the argument types to
determine them. Consider the function:

```mzn
function var opt $T: foo(var $T: x, any $T: y);
var 1..3: p;
opt int: q;
any: v = foo(p, q);
```

Then for the call `foo(p, q)` we start by considering the first argument `p`, which is of type `var int`. The function
parameter is declared as `var $T`, so that means it applies its own inst (`var`) and optionality (non-`opt`) to the type
of `$T` - therefore we strip off the inst and optionality from the given argument type and use that for `$T`. So
`$T` for this argument is set to be (`par`, non-`opt`) `int`.

Next we look at the second argument `q`, which is of type `opt int`. Since the function parameter is declared as
`any $T`, it does not apply any modifiers to the type of `$T`, so we set the type of `$T` for this argument to be
`opt int`.

Then the final type for `$T` is the most specific supertype of `int` and `opt int`, meaning the final type is `opt int`.
From this, we can compute the actual instantiated function signature by substituting `$T` = `opt int` for the parameters
and return type.

- The first parameter `var $T` becomes `var int` (again, since `var $T` is really `var`, non-`opt` `$T`).
- The second parameter of `any $T` gives `opt int` (since `any` means don't apply any modifiers to `$T`).
- The return type of `var opt $T` gives `var opt int` (since it applies the `var` and `opt` modifiers to `$T`).

So the instantiated result is:

```mzn
function var opt int: foo(var int: x, opt int: y);
var 1..3: p;
opt int: q;
var opt int: v = foo(p, q);
```

## Output typing

The expressions of output items, and definitions of `:: output_only` declarations are type-checked in a special mode
where the types of top-level identifiers are assumed to be par. This alleviates the need to manually insert calls to
`fix()`.

In this example, if we don't consider `p` in the output statement to be `par`, then the if-then-else will cause will
trigger a type error as it will return the illegal type `var string`.

```mzn
var bool: p;
output [if p then "Yes" else "No" endif];
```
