% References into the objects that the same data creates. Which objects exist
% only depends on the data's owned children, so the references may point at
% them, whether the children are written as literals or not.
class Leaf ( 1..9: v );
class Tree ( set of new Leaf: leaves; opt new Leaf: spare; Leaf: favourite );
class Node ( 1..9: v; opt Node: next );
array [int] of record(1..9: v): more = [(v: 5), (v: 6)];
new Tree: t = (leaves: [(v: i) | i in 1..2], spare: (v: 3), favourite: max(Leaf));
set of new Tree: ts = [(leaves: more, spare: <>, favourite: min(Leaf))];
set of new Node: ns = [(v: i, next: if i = 1 then <> else min(Node) endif) | i in 1..3];
output [
	"\([l.v | l in Leaf]) \(t.favourite.v) \([u.favourite.v | u in ts])\n",
	"\([if occurs(n.next) then (deopt(n.next)).v else 0 endif | n in ns])\n",
];
solve satisfy;
