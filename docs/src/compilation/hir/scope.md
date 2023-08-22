# Scope collection

Scope collection gives the identifiers in scope for each expression in an item. Note that at this point, we don't
perform name resolution, as resolving call identifiers requires the types of the arguments to perform function
overloading resolution, and resolving record field access requires knowledge of the accessed record's type.

It would be possible to only keep track of this for expressions which are identifiers, rather than all expressions
(and we could even perform name resolution for variables and non-overloaded function calls). However, having the scopes
computed for every expression is useful for providing code completion information in the language server.

## Scope collection results

Scope collection results in a data structure containing an arena of `Scope`s which contain the identifiers defined in
each scope in an item. Each `Scope` keeps track of its parent `Scope` index, so that the chain of scopes can be followed
to resolve an identifier. Each expression index in an item is mapped to a scope index and generation (see below for
details on generations).

The `db.lookup_item_scope(item_ref)` query produces a `ScopeResult` which allows for easy lookup of functions and
variables in scope for an expression. It automatically traverses the item scopes and will bubble up to global scope
to find an identifier. Note that the result of looking up identifiers is a `PatternRef` (or list of them for overloaded
functions). These point to the declaring identifier pattern of the identifier (as there may be several identifiers
declared in a single item, via destructuring or in a let expression, so it is not sufficient to give only the item).

## Scopes and generations

A `Scope` keeps track of the variables and functions declared in a particular scope, along with the the parent scope (if
any). For example:

```mzn
function foo(int: a) =
    let {
        int: x = 3 * a;
        int: y = x + 2;
    } in a + y;
```

Has two scopes: one created by the function item for its body, and one created by the let item.

The first scope contains a declaration for the identifier `a` and its parent scope (assuming this is a top-level item)
is the global scope. The second scope has declarations for the identifiers `x` and `y`, and its parent scope is the
first scope.

Then, the scope for the outer let would be computed to be the first scope, while the scope for the RHS of `x`, the
RHS of `y`, and the `in` expression `a + y` would be the second scope (and all of their sub-expressions would also
be assigned to be the second scope).

Note that items in let expressions are such that they only enter into scope once they appear, so an item cannot refer to
an item later than itself. For example:

```mzn
let {
    int: y = 10;
    int: z = let {
        int: x = y;
        int: y = 1;
    } in x;
} in z
```

Will give `z = 10` as the let item `int: y = 1` is not in scope for the declaration of `x`.

This is achieved by keeping track of the 'generation' where a let-item identifier has been declared. The generation is
incremented every time we visit an identifier pattern which creates a new variable. It should also be noted that the
patterns for a declaration in a let are always visited after processing their RHS. Expressions are then assigned both
a scope, along with a generation defining the minimum generation an identifier must be to be considered available in
the expression. Exiting a scope restores the generation the value it had upon entering the scope.

Following the example:

- The declaration `int: y = 10;` is visited
  - The RHS expression (`10`) is assigned scope 0, gen 0
  - The LHS identifier `y` is assigned scope 0, gen 1
- The declaration `int: z = let { int: x = y; int: y = 1; } in x` is visited
  - The RHS `let` expression is assigned scope 0, gen 1
    - The declaration `int: x = y;` is visited
      - The RHS expression `y` is assigned scope 1, gen 1
      - The LHS identifier `x` is assigned scope 1, gen 2
    - The declaration `int: y = 1` is visited
      - The RHS expression `1` is assigned scope 1, gen 2
      - The LHS identifier `y` is assigned scope 1, gen 3
    - The `in` expression `x` is assigned scope 1, gen 3
  - The LHS identifier `z` is assigned scope 0, gen 1 (restored because exited scope 1)
- The `in` expression `z` is assigned scope 0, gen 1

From this, we can see that while the RHS of `int: x = y` is scope 1, since its generation is 1, the `y` does not refer
to the `y` in scope 1 (which is generation 3), and so instead refers to the `y` in scope 0 (which is generation 1).

It should be noted that the 'generation' values are an implementation detail - what actually matters in the end is to
be able to determine whether or not an identifier is in scope or not for a given expression.

## Patterns

Patterns used for variable or parameter declarations are only allowed to contain tuples, records and identifiers.
Otherwise, the pattern is refutable and may not match all values. Identifiers in variable declarations always create
new variables, and do not refer to enumeration/annotation atoms (instead they get shadowed).

```mzn
enum Foo = {A, B};

% OK: creates variables `x`, `y` and `z`.
any: (x, (foo: y, bar: z)) = (1, (foo: 2, bar: 3));

% OK: shadows enum values `A` and `B`
any: z = let {
    int: A = 1;
    int: B = 2;
} in A + B;

% ERROR: refutable pattern may not match all cases.
any: (m, 1) = (3, 2);
```

In comprehension generators and case expressions, all patterns are allowed, and identifiers can either refer to
enumeration/annotation atoms, or if none match, they create new variables. A problem with this approach is that
since we don't have namespacing, a user could create an atom with the same name as a variable binding in a case
expression or generator in another part of the code, and then that variable binding would instead match that atom,
changing the behaviour (likely causing the model to fail type checking).
