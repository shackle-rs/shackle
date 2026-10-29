% Inline data whose reference field points at objects that the same data
% creates: the objects are sized by `Tree_t_structure`, not the inputs.
class Leaf ( 1..3: v );
class Tree ( set of new Leaf: leaves; Leaf: favourite );
new Tree: t = (leaves: [(v: 1), (v: 2)], favourite: min(Leaf));
solve satisfy;
