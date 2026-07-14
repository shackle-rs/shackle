class B (
  2..3: x;
);

class A (
  set(1..2) of new B: children;
  int: n = card(children);
);

set(1..2) of new A: as = [(children: [(x: 2)]), (children: [(x: 2), (x: 3)])];
output ["ns=", show([a.n | a in as]), "\n"];
solve satisfy;
