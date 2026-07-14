class A (
  0..2: c;
);

int: n;
var set(1..n) of new A: as1;

int: m;
var set(1..m) of new A: as2;

solve satisfy;