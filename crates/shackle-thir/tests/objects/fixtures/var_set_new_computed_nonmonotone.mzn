% A NON-monotone computed attribute (`y = 5 - x`) on a `var set of new` root.
% The symmetry-break default wave must not pin alias-defined fields: on an
% unrealised slot the free `x` is pinned to `lb(x) = 0`, so the alias gives
% `y = 5`, while a pin would demand `y = mzn_safe_default(y) = lb(y) = 0`.
% The failed disjunct then forces every potential slot into the class set,
% silently removing all solutions with fewer than two objects — this model
% (`card(as) = 0`) was UNSATISFIABLE before the wave learned to skip
% functionally-determined fields. The whole prior corpus was monotone
% (`b + 1`, `card(...)` over lb-`{}` sets), where alias value and pin happen
% to coincide, so it could not witness the conflict.
class A(var 0..5: x; var int: y = 5 - x);
var set(0..2) of new A: as;
constraint card(as) = 0;
solve satisfy;
output ["card=", show(card(as)), "\n"];
