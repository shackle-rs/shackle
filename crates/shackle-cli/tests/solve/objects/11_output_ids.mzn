% No output item: the output declarations are printed, objects and references
% by their string ids. Owned children are nested records, `set of new`
% fields arrays of them; the reference `pick` is the id of a leaf.
% The `Seat_seats_objects` line in the expected output is lowering-internal
% storage leaking into the output (a known issue): it disappears once internal
% declarations are kept out of the output.
class Leaf ( 1..9: value; );
class Node ( set of new Leaf: leaves; );
class Seat ( var 1..2: comfort; );
class Wagon ( set(1..2) of new Seat: seats; );
set of new Node: nodes :: output = [
  (leaves: [(value: 1), (value: 2)]),
  (leaves: [(value: 3)])
];
var new Wagon: w;
var Leaf: pick;
solve satisfy;
