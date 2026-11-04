% A par `opt new` root introduces zero or one objects, depending on its value,
% which can be any `opt` record: a present one is identified like a singular
% root (`t`, `t.main`), and an absent one has no objects. Its block of objects
% is chained after the previous roots' (`a` after `ls`).
class Leaf ( 1..3: v );
class Sub extends Leaf ( 1..3: w );
class Tree ( set of new Leaf: leaves; new Leaf: main; opt new Leaf: spare );
bool: b = true;
opt new Tree: t :: output = (leaves: [(v: 1), (v: 2)], main: (v: 3), spare: <>);
opt new Tree: u :: output = <>;
opt new Tree: w :: output =
  if b then (leaves: [], main: (v: 1), spare: (v: 2)) else <> endif;
set of new Leaf: ls :: output = [(v: 1)];
opt new Leaf: a :: output = (v: 2);
opt new Sub: s :: output = (v: 3, w: 1);
var 1..3: x;
constraint occurs(a) -> x = deopt(a).v;
output [
  "\(Leaf) \(Sub) \(Tree) \(x)\n",
  "\(t) \(u) \(w) \(a) \(s)\n",
];
