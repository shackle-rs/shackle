class A (
  int: x;
  int: y = x + 1;
);

new A: a = (x: 1);
solve satisfy;
