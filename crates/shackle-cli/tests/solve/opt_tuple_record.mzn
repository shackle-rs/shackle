% Opt tuples and records: declared, inferred, given in data, read through
% occurs/deopt and their fields, with bounded and opt fields, and shown
opt record(1..3: v, opt 2..3: w): r = (v: 2, w: <>);
opt record(1..3: v, opt 2..3: w): s = <>;
opt tuple(int, int): t = (1, 2);
bool: b = true;
any: x = if b then (v: 1) else <> endif;
array [1..2] of opt tuple(5..6): ts = [<>, (5,)];
function opt record(int: v): pick(bool: p) = if p then (v: 3) else <> endif;
opt record(1..3: v): d;
opt record(1..3: v): e;
opt record(opt int: x, int: y): f;
var 1..3: z;
constraint z = deopt(r).v;
output [
  "\(r) \(s) \(t) \(x) \(ts) \(pick(false)) \(d) \(e) \(f)\n",
  "\(occurs(r)) \(r.v) \(r.w) \(s.v) \(t.2) \(deopt(x).v) \([occurs(u) | u in ts] == [false, true]) \(z)\n",
];
