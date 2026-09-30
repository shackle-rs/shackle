% Roots and references left uninitialised are read from data under their own
% names, in the data form: every object record has an `id`, and references are
% ids. The internal declarations are renamed `<name>_ref`.
class Leaf ( int: w; );
class Pole ( int: len; );
class Node (
  set of new Leaf: leaves;
  new Leaf: main;
  opt new Pole: pole;
  Leaf: fav;
  set of Leaf: favs;
);
class Tree ( new Leaf: top; );
set of new Node: nodes :: output;
new Tree: tree;
Leaf: pick;
opt Leaf: maybe;
array [1..2] of Leaf: pair;
