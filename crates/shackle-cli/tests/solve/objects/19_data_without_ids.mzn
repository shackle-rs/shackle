% Objects read from a data file without ids. An object given without an id gets
% the id generated for its slot (`nodes[1]`, `nodes.leaves[2]`, `solo`), and an
% optional child its parent's position (`nodes.pole[2]`), while the others keep
% the id given in the data, which references use.
class Leaf ( int: w; );
class Pole ( int: len; );
class Node (
  set of new Leaf: leaves;
  opt new Pole: pole;
  Leaf: fav;
);
set of new Node: nodes :: output;
new Leaf: solo :: output;
Leaf: pick;
var int: total :: output = sum (n in nodes) (n.fav.w) + pick.w + solo.w;
