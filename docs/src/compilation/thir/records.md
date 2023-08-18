# Erasure of records

Erasing records into tuples involves transforming record literals into tuples,
and record access into tuple access, ensuring that the order of the fields
remains consistent.

```mzn
record(int: foo, float: bar): x = (bar: 1.5, foo: 2);
int: y = x.foo;
```

Transforms into

```mzn
tuple(float, int): x = (1.5, 2);
int: y = x.2;
```
