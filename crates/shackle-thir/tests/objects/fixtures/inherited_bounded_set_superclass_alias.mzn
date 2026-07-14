class A (
  0..4: x;
  constraint this.x >= 0;
);

class B extends A (
  0..4: y;
  constraint this.x <= this.y;
);

int: n;
var set(0..n) of new B: bs;
solve satisfy;