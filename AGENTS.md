# Repository guidance

## MiniZinc standard library

The compiler test suite and compiler-backed CLI commands such as
`shackle compile` require the `MZN_STDLIB_DIR` environment variable.

Before running them, ensure that `MZN_STDLIB_DIR` points to MiniZinc's
`share/minizinc` directory. The directory must contain `std/stdlib.mzn` and
`std/solver_redefinitions.mzn`; do not set the variable to the `std`
directory itself. See the Development section of `README.md` for setup and
command examples.
