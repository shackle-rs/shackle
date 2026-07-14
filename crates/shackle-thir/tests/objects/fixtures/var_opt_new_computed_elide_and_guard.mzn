% Singular `var opt new` root — the other root shape with an unrealisable
% slot. Pins BOTH per-field outcomes of the 2.5(a) elision analysis on the
% singular arm: `s = 5 - x` (total RHS, no declared domain) elides its guard
% entirely (no witness, no if-then-else), while `y = 6 div (x + 1)` (`div`
% is not on the totality whitelist) keeps its guard — so the shared per-slot
% `realised` alias must still be emitted for `y` alone. `absent(a)` is SAT.
class A(var 0..5: x; var int: s = 5 - x; var 1..6: y = 6 div (x + 1));
var opt new A: a;
constraint absent(a);
solve satisfy;
output ["absent ok\n"];
