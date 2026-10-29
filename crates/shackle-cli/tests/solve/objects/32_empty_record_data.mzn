% A class with only var attributes gets the empty record `()` as its data, as
% a child (singular, opt and in a set) and as a root. MiniZinc has no empty
% tuple, so `()` is written as `(true,)`.
class C ( var 1..3: v );
class P ( 1..2: k; new C: c; opt new C: o; set of new C: cs );
new P: p = (k: 1, c: (), o: (), cs: [(), ()]);
new C: one = ();
set of new C: more = [(), (), ()];
any: unit = (k: 1, u: ());
constraint p.c.v = 3;
constraint forall (x in p.cs) (x.v = 2);
output ["\(card(C)) \(p.c.v) \([x.v | x in p.cs]) \(unit)\n"];
solve satisfy;
