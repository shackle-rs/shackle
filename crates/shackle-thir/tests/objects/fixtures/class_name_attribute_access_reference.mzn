% `A.r` for a reference attribute is an array of references, indexed by the
% referring object.
class C ( 1..3: c );
class A ( int: x; C: r );
new C: c1 = (c: 1);
new A: a1 = (x: 1, r: c1);
array [A] of C: refs = A.r;
solve satisfy;
