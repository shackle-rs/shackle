% Output of owned children: a root's record contains its `new` children's
% records, a `set of new` field as an array of them. The par root `nodes` is
% output because it is annotated `::output`. The `var set(..) of new` root `ws`
% is rendered as an array of the realised objects only.
class Leaf ( 1..9: value; );
class Node ( set of new Leaf: leaves; new Leaf: first; );
class Seat ( var 1..2: comfort; );
class Wagon ( set(1..2) of new Seat: seats; );
set of new Node: nodes :: output = [
  (leaves: [(value: 1), (value: 2)], first: (value: 3))
];
var set(0..2) of new Wagon: ws;
solve satisfy;
