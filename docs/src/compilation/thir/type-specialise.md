# Type specialisation

Type specialisation involves generating concrete versions of polymorphic functions for each call to
such a function. This is needed because the [MIR](../mir/mir.md) does not have polymorphic functions.

We start by finding the top-level polymorphic calls, and then create the concrete versions
if we haven't already. We then traverse the bodies of the functions and recursively specialise
polymorphic calls in them.

The end result is that all reachable calls are now non-polymorphic.

We can then remove all polymorphic which have bodies.

Calls to polymorphic functions without bodies are left as-is and not specialised since they would
be interpreter builtins.

## Example

```mzn
function any $T: foo(any $T: x) = bar(x);
function any $T: bar(any $T: x) = x;
any: a = foo(1);
any: b = bar(2);
```

First we process the call `foo(1)` and generate

```mzn
function int: foo(int: x) = bar(x);
```

Then we process the call in the body of the newly generated `foo`, which is `bar(x)`, and generate

```mzn
function int: bar(int: x) = x;
```

Then we process the call `bar(2)`, but don't need to generate anything since we already generated an
integer version.

The final result is

```mzn
function int: foo(int: x) = bar(x);
function int: bar(int: x) = x;
any: a = foo(1);
any: b = bar(2);
```

## Dynamic dispatch

In order to facilitate [dynamic dispatch](./dispatch.md), we need to ensure that when there are
multiple compatible polymorphic overloads of a function, we instantiate all versions which are
more specific that the one that has been called. Otherwise, the more specific versions won't be in
the model which we create the function dispatch preambles.

For example, consider:

```mzn
function int: foo(any $T: x) = 1;
function int: foo(var $$E: x) = 2;
function int: foo($U: x) = 3;
var opt 1..1: x;
constraint foo(x);
```

If we only instantiated the `var opt int` version of `foo(any $T)`, then we would not be able to
dispatch to the other versions at runtime.

To do this, we find the type-inst var instantiations for each polymorphic overload (other than the
original) given the argument types, and then instantiate calls using those types.

In this case, we have two other overloads and an argument of `var opt int`:

- `foo(var $$E)` gives `$$E = int`, and so the signature becomes `foo(var int)`.
- `foo($U)` gives `$U = int`, and so the signature becomes `foo(int)`.

We then instantiate `foo(var int)` and `foo(int)` and will be able to generate function dispatch
preambles as required.

Note that it's possible for multiple overloads to give the same signature - so we actually have to
re-lookup the best matches to prevent instantiating the wrong versions. For example, if the second
overload was `foo($$E)`, then both `foo($$E)` and `foo($U)` would give `foo(int)` to instantiate,
and in that case, we need to use the more specific `$$E` version and not the `$U` one.

Finally, we also have keep track of which polymorphic function each specialised function comes from,
because dynamic dispatch should not happen between different instantiations of the same polymorphic
function.

## Generation of `show` for erased types

Enums, option types and records (and types containing them) need to have specialised versions of
`show` generated. The definition of `show` for plain enums is actually generated during
[enum erasure](./enums.md), but the others are generated at this stage.

## Optimisation for enum functions

In the future, it would be possible to add an optimisation for functions which are polymorphic over
enum types. Functions which take a `$$E` can actually just specialise into the type-erased `int`
version if the call does not lead to a `show($$E)` call somewhere down the line.

However, it should be noted that since the `fzn_...` functions are never polymorphic, there probably
aren't that many layers of depth to most calls.

## Looking up function calls after specialisation

Once specialisation has occurred, we it's no longer valid to perform function lookups on non-builtin
polymorphic functions (as we would have had to specialise them in order to use them correctly).

So all calls introduced after this stage must either be to builtins, or to concrete functions (or
exactly matching an existing specialised version).
