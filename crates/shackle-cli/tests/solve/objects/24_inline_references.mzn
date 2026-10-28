% Objects given in the model can refer to other objects in their attributes.
% The references lower to the referenced objects' identities, so the inputs
% they are read from must be typed by those identities rather than the class.
class C ( 1..3: c );
class A ( 1..2: x; opt C: o; set of C: rs; array [1..2] of C: ps; opt A: next );
new C: c2 = (c: 2);
new C: c3 = (c: 3);
new A: a1 = (x: 1, o: c3, rs: {c2, c3, c3}, ps: [c3, c2], next: <>);
set of new A: as = [(x: 2, o: <>, rs: {c2}, ps: [c2, c2], next: a1)];
constraint card(a1.rs) = 2;
constraint sum(r in a1.rs) (r.c) + a1.ps[1].c + a1.o.c = 11;
output [
	show((a1.o, a1.rs, a1.ps, a1.next)), "\n",
	show([(a.x, a.o, a.rs, a.ps, a.next) | a in as]), "\n",
];
solve satisfy;
