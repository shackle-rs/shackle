class A (
  int: x;
  constraint this.x >= 0;
);

A: a;
constraint a.x >= 0;
solve satisfy;
