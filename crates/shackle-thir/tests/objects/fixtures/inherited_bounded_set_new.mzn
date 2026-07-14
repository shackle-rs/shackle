class A (
  0..2: x;
);

class B extends A (
  0..2: y;
);

int: n;
var set(0..n) of new B: bs;
solve satisfy;