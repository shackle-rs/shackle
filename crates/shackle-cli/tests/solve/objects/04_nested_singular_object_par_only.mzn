% Depth-1 singular object nesting with NO var root: the singular nested
% `kid.children` field must be stored as `<B>_potential` identities rather
% than as the inline input record. The depth-1 lowering mints identities
% uniformly (par and var), so this par-only model lowers and solves.
% A = {root.kid}; its single child has x=2 -> one solution, xs=[2].
class B ( 2..3: x; );
class A ( set(1..1) of new B: children; );
class P ( new A: kid; );

new P: root = (kid: (children: [(x: 2)]));

solve satisfy;

output ["xs=", show([ sum([b.x | b in o.children]) | o in A ]), "\n"];
