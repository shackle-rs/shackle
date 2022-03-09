# Type specialisation

Calls to polymorphic functions are monomorphised into versions which use
concrete types. Name mangling is used to produce the new declarations (and
allow recovery of the original name).

## Notes

Since we intend to pre-compile the standard library, we will have to
monomorphise any polymorphic functions based on what the possible types for
the call arguments are.

Alternatively we could leave polymorphic functions un-compiled and instead
compile them on the fly.
