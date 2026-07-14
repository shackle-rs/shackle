class A (
  int: a;
  set of new B: bs;
);

class B (
  int: b;
);

set of new A: as = [
  (a: 1, bs: [(b: 1), (b: 2)]),
  (a: 2, bs: [(b: 3)])
];

solve satisfy;