# Removal of overloading

Since [`MIR`](../mir/mir.md) does not support overloading, we perform name-mangling to give each
overloaded variant of a function a distinct name.
