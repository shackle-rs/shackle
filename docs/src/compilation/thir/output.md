# Output item removal

The [`MIR`](../mir/mir.md) does not have output items. Instead, we create
`:: output_only` string declarations for each output section.

```mzn
output ["foo\n"];
output :: "section" ["bar\n"];
output :: "section" ["qux\n"];
```

Becomes:

```mzn
string: mzn_output :: output_only = concat(["foo\n"]);
string: mzn_output_section :: output_only = concat(["bar\n"] ++ ["qux\n"]);
```

We also make the `:: output` variables explicit, so `var` declarations with no
RHS definitions get `:: output` added unless they are already marked as
`:: output` or `:: no_output`.
