# Essence' Compiler
In addition to the MiniZinc compiler, Shackle also offers language support for Essence'. 

## Limitations
The compiler lacks support for certain operations currently. These include:
- Multiple where statements
- Array parameters with placeholder variables to represent length
- Multidimensional arrays

## Extra Features
The compiler adds some extra features found within MiniZinc to Essence'. These include:
- Output Statements
- String Literals
- Infinity Literal
- Any Type (not extensively tested)

Semantic Changes:
- In normal Essence' if a `letting` statement is 