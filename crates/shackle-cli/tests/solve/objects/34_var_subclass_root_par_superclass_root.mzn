% A var root of a subclass makes the superclass's set of objects a decision,
% also when the superclass has a par root of its own. `Base` holds `bs`, the
% leaf of `t`, and `more`'s object when there is one.
class Base ( 1..3: v );
class Leaf extends Base ();
class Tree ( set of new Leaf: leaves );
set of new Base: bs = [(v: 3)];
new Tree: t = (leaves: [(v: 1)]);
var set(0..1) of new Leaf: more;
output ["\(Base) \([b.v | b in Base]) \(Leaf) \(more)\n"];
