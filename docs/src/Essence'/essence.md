# Essence' Compiler

In addition to the MiniZinc compiler, Shackle also offers language support for Essence'.

## Limitations

The compiler lacks support for certain operations currently. These include:

- Multiple where statements
- Array parameters with placeholder variables to represent length
- Matrix comprehensions with inner matrix literal with indexes, don't use the index set `[[i+j | j : 1..n ; int(2..n+1)] | i : 1..n]`, won't use the declared index set.
- Matrix comprehensions with `[[[i, i+1]] | i : 1..10]` not supported as MZN doesn't support `[([i, i+1]) | i in 1..10]`.

## Extra Features

The compiler adds some extra features found within MiniZinc to Essence'. These include:

- Output Statements
- String Literals
- Infinity Literal
- Any Type (not extensively tested)

Semantic Changes:

- In normal Essence' if a `letting` statement is called in a model file it uses a declaration, while if used in a parameter file it will be an assignment. This is changed to be if a letting statement is called on a variable which hasn't been declared already it will become a declaration of type any.
