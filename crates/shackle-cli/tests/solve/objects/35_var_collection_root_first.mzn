% A var collection root before other roots of its class: the later blocks of
% `Leaf` start after the var root's block, a nested one (`t.leaves`) and a
% top-level one (`ls`).
class Leaf ( 1..3: v );
class Tree ( set of new Leaf: leaves );
var set(0..1) of new Leaf: extra;
new Tree: t = (leaves: [(v: 1), (v: 2)]);
set of new Leaf: ls = [(v: 1)];
constraint forall (l in extra) (l.v = 3);
output ["\(Leaf) \([l.v | l in Leaf]) \(extra) \([l.v | l in t.leaves])\n"];
