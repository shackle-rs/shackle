% Data and solutions in both targets. The MicroZinc target erases records,
% optional values and enums, so they are written and read in the erased form;
% the MiniZinc target leaves them to MiniZinc. Both give the same solution.
enum Colour = {Red, Green, Blue};
record(int: a, opt int: b): r;
opt int: o;
opt int: n;
Colour: c;
set of Colour: cs;
array [Colour] of int: w;
tuple(int, bool): t;
var record(var 1..5: x, var opt 1..3: y): v;
var opt Colour: vc;
array [Colour] of var 0..9: vw;
constraint v.x = r.a + deopt(o) /\ absent(v.y) /\ absent(n) /\ absent(r.b);
constraint vc = c /\ Blue in cs;
constraint forall (i in Colour) (vw[i] = w[i] + t.1);
solve satisfy;
