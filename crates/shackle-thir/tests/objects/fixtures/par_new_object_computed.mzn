class B (
  2..3: x;
);

class A (
  set(1..1) of new B: children;
  int: n = card(children);
);

new A: a = (children: [(x: 2)]);
constraint a.n = 1;
output ["n=", show(a.n), "\n"];
solve satisfy;
