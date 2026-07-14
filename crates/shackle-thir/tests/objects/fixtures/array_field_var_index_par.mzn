% Array-typed attribute read back through a var index on a PAR class.
% `chosen.xs` decomposes via the per-position fast path
% (`decompose_array_field_var_access`): one scalar column per position j,
% reassembled with arrayXd. The instance forces the id=2 object's xs to
% all-zero and the total to 1, so `sum(chosen.xs) = 1` pins chosen to the
% id=1 object: cid=1, sums=[1, 0].
class A ( 1..2: id; array[1..3] of var 0..2: xs; );
set of new A: as = [(id: 1), (id: 2)];
var A: chosen;
var 1..2: cid;
constraint sum(a in as)(sum(a.xs)) = 1;
constraint forall(a in as)(a.id = 2 -> sum(a.xs) = 0);
constraint sum(chosen.xs) = 1;
constraint cid = chosen.id;
output ["cid=", show(cid), " sums=", show([sum(a.xs) | a in as])];
solve satisfy;
