% Family (c) of the field-only universe fallback: a multi-hop nested
% introduction. P (one hop) derives correctly from the root field; N (two
% hops) records only the joined-path introduction ("ps_ns" on the root),
% fails the direct-field check, and falls back to `N = N_potential` —
% claiming a phantom member although every realised P has an empty ns.
% Correct semantics: N = {} and nN=0.
class N ( var 0..1: v; );
class P ( var set(0..1) of new N: ns; );
class C ( var set(1..1) of new P: ps; );
var new C: c;
constraint forall(cc in C, p in cc.ps)(card(p.ns) = 0);
output ["nN=", show(card(N)), " nP=", show(card(P))];
solve satisfy;
