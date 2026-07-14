int: m;
int: n;

class B (
  0..2: y;
);

class A (
  var set(0..n) of new B: bs;
);

var set(0..m) of new A: as;
solve satisfy;
