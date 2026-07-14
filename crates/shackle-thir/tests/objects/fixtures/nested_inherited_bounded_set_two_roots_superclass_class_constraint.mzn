int: n;

class A (
  0..4: x;
  constraint this.x >= 0;
);

class B extends A (
  0..4: y;
);

class Root (
  var set(0..n) of new B: bs;
);

var new Root: r1;
var new Root: r2;
solve satisfy;