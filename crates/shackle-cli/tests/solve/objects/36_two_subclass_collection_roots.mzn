% Two collection roots of a subclass: the superclass's set is the image of
% each root's block, whose identities start after the previous root's.
class A ( 1..9: v );
class B extends A ( 1..9: w );
set of new B: b1 = [(v: 1, w: 1)];
set of new B: b2 = [(v: 2, w: 1), (v: 3, w: 1)];
output ["\(A) \([a.v | a in A]) \([b.w | b in B])\n"];
