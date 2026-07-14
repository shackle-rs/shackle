class A (
  int: x;
  constraint this in A;
);

set of new A: as = [(x: 1), (x: 2)];
solve satisfy;