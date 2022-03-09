# Compilation to MicroZinc

Compilation of the MiniZinc AST to MicroZinc will involve transformation to
a new AST for MicroZinc.

## Replacing operators by function calls

Operators need to be replaced by calls.

## Decomposition of variable conditionals

`if-then-else` expressions with a variable condition need to be rewritten into
function calls.

## Lifting partiality

Partial functions need to transformed into total functions.

## Subtype based overloading

As functions arguments also accept their subtypes, these should be transformed
to dispatch to their specialised counterparts.

## Pass values from outer scopes of functions through parameters

Functions which refer to variables outside the scope of the function need to be
transformed such that these variables are passed as arguments to the function.

## Generation of the main entrypoint

MicroZinc uses a top-level function `main` as its entrypoint for the interpreter.

The top-level decision variable and constraints are added as a `let` expression
in this function, taking the model parameters as arguments.
