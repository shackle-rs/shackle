int: n;

class B (
  0..4: y;
  constraint this.y >= 0;
);

class A (
  var set(0..n) of new B: bs;
);

var new A: a;
solve satisfy;