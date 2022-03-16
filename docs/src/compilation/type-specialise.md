# Type specialisation

Calls to polymorphic functions are monomorphised into versions which use
concrete types. Name mangling is used to produce the new declarations (and
allow recovery of the original name).

## Notes

- If we want to pre-compile the standard library, we will have to monomorphise
  any polymorphic functions based on what the possible types for the call
  arguments are.
  - Actually this may not be possible because we need monomorphisation to
    specialise calls involving enums - and user enums which can be used with
    stdlib won't be available if precompiled
  - Could we compile the model + stdlib separately from the solver library?
    Probably not because decompositions in the solver library may use library
    functions which need to be specialised
- Alternatively we could leave polymorphic functions un-compiled and instead
  compile them on the fly.
- To start with, the initial implementation should probably just require
  compiling the library along with the model.
