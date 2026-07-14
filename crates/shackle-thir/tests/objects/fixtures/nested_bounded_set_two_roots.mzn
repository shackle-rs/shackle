int: n;

class B (
  0..2: y;
);

class A (
  var set(0..n) of new B: bs;
);

var new A: a1;
var new A: a2;
solve satisfy;