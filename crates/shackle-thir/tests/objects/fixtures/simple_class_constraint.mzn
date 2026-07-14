class A (
  int: x;
  constraint this.x >= 0;
);

new A: a = (x: 1);
solve satisfy;