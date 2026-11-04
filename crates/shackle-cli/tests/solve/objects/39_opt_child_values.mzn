% An `opt new` child given by any value of its `opt` record type: a literal, a
% conditional in a comprehension, a `let`, a function call, an identifier, or a
% whole input record. Optional children are output as a record or `<>`, and
% take their parents' positions in their ids (`ms.extra[2]`, not `[1]`).
class Leaf ( 1..9: v );
class Mid ( 1..9: w; opt new Leaf: extra );
class Top ( new Mid: main );
function opt record(int: v): pick(bool: p) = if p then (v: 4) else <> endif;
opt record(int: v): given = (v: 5);
record(int: w, opt record(int: v): extra): whole = (w: 6, extra: <>);
set of new Mid: ms :: output =
  [(w: k, extra: if k = 1 then <> else (v: k) endif) | k in 1..3];
new Top: t :: output = (main: let { int: n = 9 } in (w: 1, extra: (v: n)));
new Mid: f :: output = (w: 2, extra: pick(true));
new Mid: g :: output = (w: 3, extra: given);
new Mid: h :: output = whole;
