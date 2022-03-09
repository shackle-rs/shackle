# Type checking

The types for each expression will be determined in a bottom-up fashion. That
is, the types of child nodes are used to determine the type of parent nodes (and
check for correctness). This stage also matches calls with their declarations.

## Unification

For literals which have a bottom type (e.g. `<>`, `_`, `[]` and `{}`), a special
unification step needs to be performed as bottom up typing is not sufficient.
Note that this unification does not perform inference across calls.

### Variable assignment example

```mzn
array [int] of var opt int: x = [_];
```

Bottom up typing will be unable to determine a concrete type for the `[<>, _]`
literal (although it is known to be `var`). Instead, the left-hand side type
should be pushed into it, making it an `array [int] of var opt int`.

### Function call example

```mzn
test foo(array [int] of var opt int: x, int: y) = true;
test foo(array [int] of var int: x, set of int: y) = false;
constraint foo([_, _], 1);
```

Bottom up typing will be unable to determine a type for the `[_, _]` literal.
In this case, since second argument of the call is an `int`, only the first
overload of `foo` is possible. Therefore the type should be
`array [int] of var opt int`.

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
used. Otherwise, new syntax for annotating a literal with its type will be
required.
