% The objects of a singular child and of a superclass only depend on the
% structure of the data too, so the data's values and references may read them.
class Base ( 1..9: v );
class Leaf extends Base ( 1..9: w );
class Tree ( set of new Leaf: leaves; new Base: main; Base: favourite );
new Tree: t = (
	leaves: [(v: 1, w: 2), (v: 3, w: 4)],
	main: if card(Base) > 2 then (v: 5) else (v: 6) endif,
	favourite: max(Base),
);
output ["\([b.v | b in Base]) \(t.main.v) \(t.favourite.v)\n"];
solve satisfy;
