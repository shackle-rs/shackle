% A var-reached class (`var new C`) with a computed (RHS) attribute
% (`c = b + 1`). `var new` varifies every attribute, so both `b` and the
% computed `c` become var. The computed attribute should be *defined* by the
% storage reconstruction (an alias), not left as a free decision pinned only by
% the class-body forall.
class C (
  0..9: b;
  0..10: c = b + 1;
);

var new C: obj;
constraint obj.b = 4;
solve satisfy;
output ["b=", show(obj.b), " c=", show(obj.c), "\n"];
