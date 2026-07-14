% A class mixing par computed attributes with a var attribute whose declared
% domain depends on a computed attribute (`var 1..z: s`). The storage record
% element type carries `var int: s` (the per-object bound `1..z` can't live in
% a shared record type); the reconstruction comprehension mints `s` as a fresh
% decision with its real per-object domain via `let { var 1..z: .. } in ..`.
class A (
  int: x;
  int: y = x + 1;
  int: z = y + 4;
  var 1..z: s;
);

set of new A: a = [(x: 3), (x: 5)];
constraint forall(p in a)(p.s >= p.x);
solve satisfy;
output [
  "xs=", show([p.x | p in a]), "\n",
  "ys=", show([p.y | p in a]), "\n",
  "zs=", show([p.z | p in a]), "\n",
  "ss=", show([p.s | p in a]), "\n",
];
