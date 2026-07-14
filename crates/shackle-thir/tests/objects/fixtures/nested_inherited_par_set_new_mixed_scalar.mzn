class A (
  int: x;
);

class B extends A (
  int: y;
);

class Root (
  var 0..2: z;
  set of new A: as;
  set of new B: bs;
);

set of new Root: roots = [
  (as: [(x: 1)], bs: [(x: 2, y: 3), (x: 4, y: 5)]),
  (as: [(x: 6), (x: 7)], bs: [(x: 8, y: 9)])
];

solve satisfy;
