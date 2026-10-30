% A var-existence field (`var set of new`, `var opt new`) of a par object
% whose class has no par attributes: each object gets a fresh decision, which
% must be a finite `var set of D_potential` in MiniZinc.
class D ( 1..3: x );
class C ( var set(0..2) of new D: ds; var opt new D: maybe );
class P ( 1..2: k; new C: c );
new P: p = (k: 1, c: ());
set of new C: cs = [(), ()];
constraint card(p.c.ds) = 2 /\ forall (d in p.c.ds) (d.x = 3);
constraint forall (c in cs) (card(c.ds) = 1 /\ occurs(c.maybe));
constraint absent(p.c.maybe);
constraint forall (d in D) (d.x >= 2);
output ["\(card(D)) \(sort([d.x | d in p.c.ds])) \([card(c.ds) | c in cs])\n"];
solve satisfy;
