% Objects and references read from data, for the errors in the data that
% MiniZinc detects: each `18_data_errors_*` data file has one. `g` is created
% by the model, so its generated id "g" is taken.
class Leaf ( int: v; );
class Holder ( Leaf: pick; );
set of new Leaf: leaves;
set of new Holder: holders;
new Leaf: g = (v: 7);
Leaf: top;
var int: total :: output = sum (h in holders) (h.pick.v) + top.v;
