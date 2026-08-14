% Depth-1 singular object nesting that IS var-reached: a par root `root`
% transitively owns a singular `new A: kid` whose class A is object-carrying
% (`children: set(1..1) of new B`) and var-reached from `var new A: a2`. The
% singular nested contribution has to mint `<B>_potential` identities for
% `kid.children` rather than store the inline input record, so that it matches
% A's varified identity storage.
% A = {root.kid, a2}; root.kid has one child (x=2), a2 has one child (x free 2..3).
class B ( 2..3: x; );
class A ( set(1..1) of new B: children; );
class P ( new A: kid; );

new P: root = (kid: (children: [(x: 2)]));
var new A: a2;

solve satisfy;

output ["xs=", show([ sum([b.x | b in o.children]) | o in A ]), "\n"];
