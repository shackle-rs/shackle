# Type specialisation

Type specialisation involves generating concrete versions of polymorphic functions for each call to
such a function. This is needed because the [MIR](../mir/mir.md) does not have polymorphic functions.

We start by finding the top-level polymorphic calls, and then create the concrete versions
if we haven't already. We then traverse the bodies of the functions and recursively specialise
polymorphic calls in them.

The end result is that all reachable calls are now non-polymorphic.

We can then remove all polymorphic which have bodies.

Calls to polymorphic functions without bodies are left as-is and not specialised

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

## Generation of `show` for erased types

Enums, option types and records (and types containing them) need to have specialised versions of
`show` generated. The definition of `show` for plain enums is actually generated during
[enum erasure](./enums.md), but the others are generated at this stage.

## Optimisation for enum functions

Functions which take a `$$E` can actually just specialise into the type-erased `int` version if
the call does not lead to a `show($$E)` call somewhere down the line.

## Looking up function calls after specialisation

Once specialisation has occurred, we it's no longer valid to perform function lookups on non-builtin
polymorphic functions (as we would have had to specialise them in order to use them correctly).

So all calls introduced after this stage must either be to builtins, or to concrete functions (or
exactly matching an existing specialised version).
