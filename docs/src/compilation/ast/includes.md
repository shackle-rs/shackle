# Include resolution

Include resolution involves recursively going through the `include` items in a MiniZinc model, and finding the linked
model files. There are also two 'implied' includes which make the standard library available. These are `stdlib.mzn`
and `solver_redefinitions.mzn`. While this process is described here as part of the AST module, as the include items
are extracted from the AST, it actually produces `ModelRef` IDs, which are part of the HIR module.

Include resolution gives a list of `ModelRef`s which comprises all of the model files to be lowered into
[HIR](../hir/hir.md). Logically these model files get concatenated together, however we don't actually do this until
the [THIR](../thir/thir.md) stage. The list of models produced is in the order that the models appear.

## Resolution rules

The resolution method used depends on the kind of path used in the include item. Note that relative includes are never
resolved relative to the process's current directory, only to the current model file (so string models without files
cannot use `./path/to/model.mzn` includes).

<table style="width:100%">
<tr><th>Include</th><th>Resolution method</th></tr>
<tr>
<td>

```mzn
include "foo.mzn";
```

</td>
<td>

- Resolved relative to include search directories
- If not found, resolved relative to current file

</td>
</tr>
<tr>
<td>

```mzn
include "./foo.mzn";
```

</td>
<td>

- Resolved relative to current file

</td>
<tr>
<td>

```mzn
include "/path/to/foo.mzn";
```

</td>
<td>

- Absolute path used

</td>
</tr>
</table>

## Cyclic includes

As all models are locally concatenated together, it actually does not matter if there are includes which are cyclic. We
only ever add any particular model file to the list of models once, so cycles are simply ignored.

## Error handling

Errors during include resolution prevent lowering to HIR from occurring (as without having all of the included files
available, it is highly likely that uninformative name resolution and type errors will occur). Note that when we fail
to resolve an include, we actually don't abort include resolution. This way, we can produce errors for as many missing
files as possible.
