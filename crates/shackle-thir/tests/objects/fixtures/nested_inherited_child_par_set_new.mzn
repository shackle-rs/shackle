class Leaf (
  int: value;
);

class A (
  int: x;
);

class B extends A (
  int: y;
  set of new Leaf: leaves;
);

class Root (
  set of new B: bs;
);

set of new Root: roots = [
  (bs: [(x: 1, y: 2, leaves: [(value: 3), (value: 4)])]),
  (bs: [(x: 5, y: 6, leaves: [(value: 7)])])
];

solve satisfy;
