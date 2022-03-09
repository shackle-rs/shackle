# Type checking

The types for each expression will be determined in a bottom-up fashion. That
is, the types of child nodes are used to determine the type of parent nodes (and
check for correctness). This stage also matches calls with their declarations.

```mzn
constraint let {
    var 1..10: x;
} in x == 5;
```

## Unification

For literals which have a bottom type (e.g. `<>` and `_`), a special unification
step needs to be performed. Note that this unification does not perform
inference across items. Instead, one of the types to be unified must be a
concrete type.

```mzn
array [int] of var opt int: x = [<>, _];
```

Bottom up typing will be unable to determine a concrete type for the `[<>, _]`
literal (although it is known to be `var opt`). Instead, the left-hand side type should be pushed into it.

```mzn
test foo(array [int] of var opt int: x) = true;
constraint foo([_, _]);
```

Bottom up typing will be unable to determine a type for the `[_, _]` literal.
The type from the parameter `x` in the declaration of `foo` should be pushed
into it (since there is only one declaration of `foo`, this is unambiguous).

### Ambiguous types

When there is not a single unambiguous type for a value, this is considered a
type error.

```mzn
test foo(array [int] of var opt $T: x) = true;
constraint foo([_]);
```

The type of `[_]` could be `array [int] of var opt int` or
`array [int] of var opt float`, or many others. Therefore, this would be a
type error. To get around this, a variable declaration for `[_]` could be
used. Otherwise, new syntax for annotating a literal with its type is required.
