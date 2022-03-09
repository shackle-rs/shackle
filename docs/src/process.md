# The Compilation Process

The compilation process for Shackle is divided into several phases.
Each phase is described in more detail in its respective section.
The overall process is divided into two main stages representing development
milestones.

{{#include images/compilation-process.svg}}

## Compilation phases

1. [Parsing](./compilation/parsing.md)  
   `tree-sitter` is used to generate a CST from a MiniZinc model.
2. [MiniZinc AST generation](./compilation/ast-gen.md)  
   A simple desugaring phase which generates an AST linked to CST.
3. [Type checking](./compilation/typecheck.md)  
   Bottom up typechecking, with an additional unification step for dealing with
   any remaining unresolved types due to `_` or `<>`.
4. [Type specialisation](./compilation/type-specialise.md)  
   Monomorphisation of polymorphic functions.
5. [MiniZinc AST-to-AST transformations](./compilation/transforms.md)  
   Main desugaring transformation phase.
6. [Compilation to MicroZinc](./compilation/microzinc-gen.md)  
   The MiniZinc AST is transformed into MicroZinc AST, which involves
   removal of function overloading, rewriting operators into calls,
   replacement of variable conditions with an appropriate decomposition, and
   removing partiality.
7. [Bytecode generation](./compilation/bytecode-gen.md)  
   Code generation of a program which will be interpreted to generate the final
   FlatZinc.
8. [Bytecode interpretation](./compilation/interpreter.md)  
   The bytecode along with the data is interpreted to produce NanoZinc and later
   FlatZinc or any other format for solver backends.

## Notes

- Compilation to MicroZinc happens without data
- Models are compiled with only declarations of standard library functions.
  Calls are resolved later by the interpreter, allowing the library to be
  swapped out without recompilation.
- The standard library, and solver redefinition libraries can be distributed
  pre-compiled into MicroZinc.
- The new compiler will use [Salsa](https://github.com/salsa-rs/salsa) so that
  computed results can be memoised and only updated as needed.

## Project notes

- Steps 1-6 will be the starting point for the compiler. The current MiniZinc
  interpreter will be extended to handle MicroZinc (e.g. support for tuples)
  to allow for testing of the new frontend.
- Steps 7-8 require more research/discussion.
