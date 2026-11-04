% Par objects of classes with both par and var attributes: each object's var
% attributes are decisions, read beside its par ones (also a par array).
class A ( 1..5: x; var 1..6: v );
class B ( array [1..2] of int: ys; var 1..3: w );
set of new A: as = [(x: 1), (x: 2), (x: 3)];
new B: b = (ys: [1, 2]);
constraint forall(a in A) (a.v = a.x + 1);
constraint b.w = sum(b.ys);
output ["\([a.v | a in A]) \(b.ys) \(b.w)\n"];
