# Rewriting of domains to constraints

This transform rewrites domains into constraints where necessary.

The [MIR](../mir/mir.md) only allows domains in the type-insts of fresh variables without RHS definitions
which do not contain structured types (or nested arrays).

For example, `var 1..3: x`, `var set of 1..3`, `array [1..3] of var 1..3`, `array [1..3] of var set of 1..3`.

Other type-inst domains, such as function parameter domains, function return type-inst domains, domains of `par`
declarations, and domains of declarations with RHS definitions, are transformed into
constraints which enforce the domains (or assertions if the declaration is `par`).

```mzn
1..3: x;
```

Becomes:

```mzn
int: x;
constraint assert(x in 1..3, "Value out of range");
```

Structured types and arrays need to check domains for each element:

```mzn
tuple(1..2, 1..3): x;
array [int] of 1..3: y;
```

Becomes:

```mzn
tuple(int, int): x;
constraint assert(x.1 in 1..2 /\ x.2 in 1..3, "Value out of range");
array [int] of int: y;
constraint forall (y_i in 1..3) (y_i in y);
```

The actual implementation also has to ensure that the semantics of when the domain gets evaluated remains consistent
with what's expected, so some extra `let` expressions are often needed to prevent re-evaluation.

Since we also would like to present the user with a useful error/warning message, we also keep track of how to display
the variable accessor (e.g. `x.1.foo` for `tuple(record(1..3: foo)): x`) and use the `mzn_domain_constraint` functions
to perform the checks.

## Unpacking of structured type declarations

Consider the declaration:

```mzn
tuple(var 1..2, var 1..3): x;
```

Since the type-inst is for a tuple type, we must rewrite this to not have a domain.

However, since this has no RHS, we do not have to introduce constraints - instead we can unpack `x`
into its constituent variables:

```mzn
tuple(var int, var int): x = let {
  var 1..2: a;
  var 1..3: b;
} in (a, b);
```

In the case of an array of structured types, a comprehension can be used. For example:

```mzn
array [1..3] of tuple(var 1..2, var 1..3): x;
```

Can be transformed into:

```mzn
array [1..3] of tuple(var int, var int): x = [
  let {
    var 1..2: a;
    var 1..2: b;
  } in (a, b)
| _ in 1..3];
```

## Index set checking

For arrays, each declared index set needs to be checked against the true index set of the array.

```mzn
array [1..3, int] of int: x;
```

Becomes:

```mzn
array [int, int] of int: x;
constraint assert(index_set_1of2(x) = 1..3, "Index set doesn't match");
```

Since we again need to produce a useful error message, we perform the index set checks using the `mzn_check_index_set`
function, which can produce an error message mentioning which dimension has the incorrect index set.
