% Two par roots of one class, each object with a nested chain of objects. Each
% root's nested objects are numbered within that root, so they line up with
% the objects created from its own inputs.
class Leaf ( 1..9: v );
class Mid ( 1..9: m; new Leaf: leaf );
class Node ( new Mid: mid );
set of new Node: nodes = [(mid: (m: 1, leaf: (v: 1))), (mid: (m: 2, leaf: (v: 2)))];
new Node: root = (mid: (m: 3, leaf: (v: 3)));
output [
	"\([(n.mid.m, n.mid.leaf.v) | n in nodes]) \((root.mid.m, root.mid.leaf.v))\n",
	"\([l.v | l in Leaf])\n",
];
solve satisfy;
