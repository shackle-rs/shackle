# Error handling

Good error reporting should be integral to the compiler.

We need to think about what errors could be encountered and what information is
needed to generate a useful message.

## MiniZinc <-> MicroZinc

- Could keep a stack of the transformations, so we always know the origin of any
  new constraints
- Transformations generate explanations of why they were done, so the logic can
  be followed for debugging

## Bytecode

- Could generate bytecode with debugging symbols giving locations for instructions
- Interpreter could have a debugging runtime mode enabling full tracing
