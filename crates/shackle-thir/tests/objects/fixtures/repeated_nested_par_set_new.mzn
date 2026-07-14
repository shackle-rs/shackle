class Leaf (
  int: value;
);

class Node (
  set of new Leaf: left;
  set of new Leaf: right;
);

set of new Node: roots = [
  (left: [(value: 1), (value: 2)], right: [(value: 3)]),
  (left: [(value: 4)], right: [(value: 5), (value: 6)])
];

solve satisfy;
