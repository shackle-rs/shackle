% `var` on a tuple or record type makes every field var, nested ones and
% `par` ones too, and keeps their domains
var tuple(1..2, opt 1..2): t;
var record(set of 1..2: s, par 1..2: n, tuple(1..2): u): r;
constraint t.1 = 2 /\ occurs(t.2) /\ t.2 < t.1;
constraint card(r.s) = r.n /\ r.n = 2 /\ r.u.1 = t.1;
