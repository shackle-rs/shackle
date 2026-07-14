class A (
  int: x;
  var 0..2: y;
);

set of new A: as = [(x: 1), (x: 2)];
solve satisfy;
