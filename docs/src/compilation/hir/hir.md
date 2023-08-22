# High-level intermediate representation (HIR)

The HIR is the representation used for validating the input MiniZinc program. Name resolution and type checking results
are used by the language server to provide type information and diagnostics. This phase of the compiler is incremental
with respect to changing model code. When an item hasn't changed (e.g. white-space changed, or the change happened in a
different file), then results of relevant analyses can be reused.

## Structure and IDs

HIR nodes are assigned 'IDs' which are generally based on numeric indices which can be used to access the node itself.
The HIR is structured such that each `.mzn` file gets its own `Model`, each of which which gets a `ModelRef` for its ID.
Models contain arenas to store each kind of top-level item. Local to each model, items can be referred to using their
arena index, and the `LocalItemRef` type can be used to store the index of any item. The `ItemRef` type includes both
the `ModelRef` and the `LocalItemRef` for an item, meaning it can be used to globally identify any item.

Expressions, user-ascribed types, and patterns (called HIR 'entities') are stored in arenas inside each item, and can be
locally referred to using their arena indices. To refer to one of these entities globally, the `ExpressionRef`,
`TypeRef` and `PatternRef` IDs can be used. `EntityRef` can be used to store a reference to any of these entities.
It should be noted that local items inside let expressions use their parent's expression storage and do not themselves
store their expressions internally like top-level items.

There is also a `NodeRef` type for referencing an arbitrary HIR node (i.e. models, items, entities).
This is mainly useful for tracking origins of nodes.

## Types in the HIR

The `Type`s used in the HIR represent user-written types, and may be incomplete. This is in contrast
to the `Ty` type, which is type computed by the type-checker, and is always complete.

Consider:

```mzn
any: x = 1;
```

In this case, the `Type` for the declaration of `x` will be `any` (set during lowering), but the `Ty` will be `par int`
(computed during type checking).

## Lowering from the AST

Lowering from AST is done on a per-model basis, accessed using the `db.lookup_model(model_ref)` query.
The actual HIR trees are kept separate from the source-mapping back to AST, which allows analysis queries to ignore the
source-mapping (unless they are emitting diagnostics) and avoid unnecessary recomputation when source locations change.

Lowering AST expressions is done by walking the AST and allocating the the HIR expressions bottom up. We also apply some
syntactic desugarings during this process. Patterns and types are also lowered in a similar way.

As the HIR nodes are built up, a `SourceMap` containing the original AST nodes and the type of desugaring which occurred
(if any) is populated. This source mapping can be accessed with `db.lookup_source_map(model_ref)`. This should only
be done if a diagnostic needs to be produced.

One complication in lowering is that we use different representations for enum assignment items and parameter assignment
items, even though we cannot distinguish them syntactically. To deal with this, we keep track of the names of the enum
items in the AST, so that we can detect what kind of assignment item is being used.

A further complication is that type-inst identifiers in MiniZinc do not get 'pre-declared', and may simply appear in a
function signature. When lowering function items, we have to keep track of the type-inst identifiers and what positions
they are used in to obtain the implied type-inst identifier 'declarations'.

### Desugarings

The HIR involves several syntactic desugarings from the AST.

> Note that only desugarings which cannot cause future errors referring to non-user written constructs to occur are
> permitted, in order to avoid emitting confusing error messages, or needing to track desugarings and specialise error
> messages for them.
>
> For example, we could desugar 2D array literals into calls to `array2d` at this point, however, if the user were to
> use an invalid index set type, as in `[|"a": "b": | 2: x, 3: y |]`, we would immediately rewrite that into
> `array2d([2, 3], ["a", "b"], [x, y])`, which would give a type error indicating that no overload of `array2d` exists
> for the given argument types, even though the user has not called `array2d` at all.

Predicate/test items are rewritten into function items:

<table style="width:100%">

<tr><th>MiniZinc syntax</th><th>Desugaring</th></tr>

<tr>
<td>

```mzn
predicate foo();
test bar();
```

</td>
<td>

```mzn
function var bool: foo();
function bool: bar();
```

</td>
</tr>

</table>

Unary/binary operators are rewritten as calls:

<table style="width:100%">

<tr><th>MiniZinc syntax</th><th>Desugaring</th></tr>

<tr>
<td>

```mzn
a + b
```

</td>
<td>

```mzn
'+'(a, b)
```

</td>
</tr>

<tr>
<td>

```mzn
-a
```

</td>
<td>

```mzn
'-'(a)
```

</td>
</tr>

<tr>
<td>

```mzn
a..
```

</td>
<td>

```mzn
'..o'(a)
```

</td>
</tr>

</table>

Generator calls are rewritten as calls with comprehension arguments:

<table style="width:100%">

<tr><th>MiniZinc syntax</th><th>Desugaring</th></tr>

<tr>
<td>

```mzn
forall (i in 1..3) (foo(i))
```

</td>
<td>

```mzn
forall([foo(i) | i in 1..3])
```

</td>
</tr>

</table>

String interpolation is rewritten using `concat` and `show`:

<table style="width:100%">

<tr><th>MiniZinc syntax</th><th>Desugaring</th></tr>

<tr>
<td>

```mzn
"foo\(value)bar"
```

</td>
<td>

```mzn
concat(["foo", show(value), "bar"])
```

</td>
</tr>

</table>

## Analysis

Analysis of the HIR is generally done on a per-item basis. This allows us to avoid recomputation if an item hasn't
changed (since even after lowering a model again, many items may still be the same). Additionally, analysis at this
stage is designed to be as robust to errors as possible. For example, scope collection errors do not prevent type
checking, and the type-checker tries to continue as much as possible in the presence of type errors.

### Scope collection

Scope collection determines the identifiers in scope for each expression in an item (and also the identifiers in the
global scope). This can be accessed using the `db.lookup_item_scope(item_ref)` query.
Note that this does not perform name resolution - that is done in the type-checker as identifiers in call expressions
cannot be resolved until the argument types are known (due to overloading).

We also check to ensure that variable declarations and function parameters only use irrefutable patterns (i.e. those
which always match every value of the type).

See [Scope collection](./scope.md) for more detail about the proces.

### Type checking

Type checking is done for each HIR item, accessed through the `db.lookup_item_types(item_ref)` query. It produces:

- A mapping from expression indices to their computed types
- A mapping from pattern indices to their computed types
  - Note that these types are augmented to include more information about
    the pattern (e.g. so you can tell the difference between a pattern forming
    a variable declaration and a pattern that forms a type alias).
- A mapping from identifier expression indices to their declaring `PatternRef`
- A mapping from identifier pattern indices to their declaring `PatternRef`
  (e.g. for enum atoms/constructor functions)

See [Type checking](./typecheck.md) for more detail about the process.

### Case expression exhaustiveness checking

Case expressions inside items are checked to ensure that they are exhaustive.

For example

```mzn
enum Foo = {A, B, C} ++ D(Bar);
enum Bar = {E, F};
var Foo: x;
any: y = case x of
    A => 1,
    B => 2,
    D(E) => 3,
endcase;
```

Is missing the case for the pattern `C` and the pattern `D(F)`.

The algorithm is based on testing whether or not a pattern is 'useful' given a list of other patterns. If the wildcard
pattern `_` is still useful given all the patterns that appear in the case arms, then the case expression is
non-exhaustive. See the documentation in `pattern_matching.rs` for details.

### Topological sorting

The topological sorter sorts the items so the declarations are not used in definitions/domains (other than function
bodies) before they appear. This is where cyclic definitions are detected.

For example:

```mzn
int: y = x + 1;
int: z = x - 1;
int: x = 3;
```

Will be reordered into:

```mzn
int: x = 3;
int: y = x + 1;
int: z = x - 1;
```

Whereas

```mzn
int: x = y;
int: y = x;
```

Has no ordering that allows definitions to appear before they are used. Therefore, this is an error.

However, this is allowed:

```mzn
constraint y = x;
var int: x = y;
var int: y;
```

As it can be reordered into:

```mzn
var int: y;
var int: x = y;
constraint y = x;
```

Which does allow every variable declaration to appear before being used.

### Validation

Some final validation of the HIR is done at the whole-program level, since some problems may be spread across multiple
files.

- Check for duplicate function definitions
  - For each pair of function items `f1` and `f2` with the same name, we consider it to be an error if the parameters of
    `f1` can be used to call `f2` and vice-versa, and either:
    - Both have bodies, or
    - They have different return types
- Check for duplicate constructors
  - Defining two constructor functions with the same name is not allowed
  - E.g. `enum Foo = A(B) ++ A(C)` is illegal since there would be two constructors named `A`.
  - Note that it is legal to overload a constructor function using a normal function. In fact, the function may have
    exactly the same signature as the constructor, and will be used in preference to it.
- Check for multiple assignments to the same variable (if allowing multiple assignments to the same variable is not
  enabled)
- Check for multiple solve items

If no errors were emitted at all for the HIR module (or the syntax model), then the program is valid and we proceed to
lower the program into the [typed high-level intermediate representation (THIR)](./thir.md).
