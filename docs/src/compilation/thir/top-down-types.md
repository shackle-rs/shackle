# Top-down typing

Top-down typing of the model involves pushing types from enclosing expressions into their subexpressions.
This allows us to determine a real type for every type found to be `bottom` during bottom-up type-checking.

For example, in the expression `[1, <>]`, bottom-up typing would give:

- `1` is an `int`
- `<>` is an `opt bottom`
- `[1, <>]`: is an `array [int] of opt int`

In top-down typing, we start from the outer-most expression and work inwards:

- `[1, <>]`: is an `array [int] of opt int` (found from bottom-up typing)
- `1`: is an `opt int` (because the element type of the enclosing array is `opt int`)
- `<>`: is an `opt int` (because the element type of the enclosing array is `opt int`)

This allows us to remove the `bottom` from the [MIR](../mir/mir.md) entirely.

For a declaration item, the top-down type of the outer-most RHS expression is taken to be the declared
LHS type from the declaration.

For example, in `tuple(opt int, float): x = (3, 5)`:

- `(3, 5)` is a `tuple(opt int, float)` (taken from LHS)
- `3` is an `opt int` (first field of tuple)
- `5` is a `float` (second field of tuple)

The top-down type of a call argument is taken to be the declared type of the parameter.

For example:

```mzn
function int: foo(opt int: x);
int: y = foo(3);
```

- `foo(3)` is an `int` (taken from the LHS of `y`)
- `3` is an `opt int` (taken from the parameter `x` of the function `foo`)

## Polymorphic types

For a polymorphic function which with a type-inst variable in the return type, we may need to use the return type to
determine the type of the parameters:

For example;

```mzn
function array [$X] of any $T: foo(array [$X] of any $T: x);
array [int] of int: y = foo([]);
```

- `foo([])` is an `array [int] of int` (taken from the LHS of `y`)
- For this call, the return type `array [$X] of any $T` is therefore `array [int] of int`
  - So `$X = int` and `$T = int`
  - Therefore the parameter `x` is `array [int] of int`
- `[]` is an `array [int] of int` (taken from the parameter `x` of the function `foo`)

## Making coercion to option types explicit

Since option types are going to be [transformed into tuples](./option-types.md), we must keep track of values which are
known to occur, but are used as optional values. These coercions must be made explicit since one the option types become
tuples, the non-tuple occurring type will no longer be a subtype of the optional one.
