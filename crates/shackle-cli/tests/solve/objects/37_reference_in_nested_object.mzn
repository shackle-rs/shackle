% A reference attribute of a nested object, beside a `new` one: only the `new`
% child gets an identity minted, the references are read as given. `Mid` is
% also created at the top level, so both contributions must have one shape.
class Leaf ( 1..9: v );
class Mid ( new Leaf: main; Leaf: pick; set of Leaf: picks );
class Tree ( set of new Mid: ms );
set of new Leaf: ls = [(v: 1), (v: 4)];
new Tree: t = (ms: [(main: (v: 2), pick: min(ls), picks: ls)]);
set of new Mid: extra = [(main: (v: 7), pick: max(ls), picks: {})];
output [
  "\([(m.main.v, m.pick.v, [l.v | l in m.picks]) | m in Mid]) \([l.v | l in Leaf])\n"
];
