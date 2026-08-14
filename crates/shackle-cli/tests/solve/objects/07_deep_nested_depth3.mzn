% Depth-3: two chained deep object classes. C (depth 2, via A.bs -> B.cs)
% owns `ds: set of new D`; D (depth 3) owns `es: set of new E`. Both C and D are
% reconstructed by the deep builder (depth-agnostic). Reading each D's `es` back
% through the minted identities recovers per-D sums [2, 7]: D1.es=[2], D2.es=[3,4].
class E ( 2..9: w; );
class D ( set of new E: es; );
class C ( set of new D: ds; );
class B ( set of new C: cs; );
class A ( set of new B: bs; );

new A: a = (bs: [ (cs: [ (ds: [ (es: [(w: 2)]), (es: [(w: 3), (w: 4)]) ]) ]) ]);

solve satisfy;

output [ "per_D=" ++ show([ sum([e.w | e in d.es]) | d in D ]) ++ "\n" ];
