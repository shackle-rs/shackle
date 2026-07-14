class A (
  0..4: x;
);

set of new A: a1 = [(x: 3), (x: 4)];
var set(1..4) of new A: a2;

solve satisfy;
