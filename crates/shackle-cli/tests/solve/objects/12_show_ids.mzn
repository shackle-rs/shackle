% `show` and string interpolation of objects print their ids, quoted like any
% string; a set of references prints as an array of ids.
class Leaf ( 1..9: value; );
class Node ( set of new Leaf: leaves; );
class Seat ( var 1..2: comfort; );
class Wagon ( set(1..2) of new Seat: seats; );
set of new Node: nodes = [
  (leaves: [(value: 1), (value: 2)]),
  (leaves: [(value: 3)])
];
var new Wagon: w;
var Leaf: pick;
output ["pick=\(pick) w=", show(w), " seats=\(w.seats) leaves=\(Leaf)\n"];
solve satisfy;
