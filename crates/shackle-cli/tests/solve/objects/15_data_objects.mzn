% Objects and references read from a data file. Every object record carries its
% id, nested ones included, and a reference is written as the id of the object
% it refers to: a set of references as an array of ids, an `opt new` child as a
% list of zero or one records. `nodes` is output under `nodes_output`, as its
% own name is taken by the data declaration.
class Leaf ( int: w; );
class Pole ( int: len; );
class Node (
  set of new Leaf: leaves;
  new Leaf: main;
  opt new Pole: pole;
  Leaf: fav;
  set of Leaf: favs;
);
set of new Node: nodes :: output;
Leaf: pick;
opt Leaf: maybe;
array [1..2] of Leaf: pair;
var int: total :: output =
  sum (n in nodes) (
    n.fav.w + sum (l in n.favs) (l.w) + if occurs(n.pole) then deopt(n.pole).len else 0 endif
  ) +
  pick.w +
  sum (l in pair) (l.w) +
  if occurs(maybe) then 1000 else 0 endif;
