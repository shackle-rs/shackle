class Leaf (
  int: value;
);

class Node (
  set of new Leaf: leaves;
);

set of new Node: left = [
  (leaves: [(value: 1), (value: 2)]),
  (leaves: [(value: 3)])
];

set of new Node: right = [
  (leaves: [(value: 4)]),
  (leaves: [(value: 5), (value: 6)])
];

solve satisfy;
