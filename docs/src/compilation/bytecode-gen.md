# Bytecode generation

## Output generation

Some kind of code which maps from FlatZinc output variables to user model
output should be generated similarly to how we generate `.ozn` output models
currently.

This functionality could be extended to allow for mapping of other solver
information such as statistics, duals, etc to report these in the context of
the user model.
