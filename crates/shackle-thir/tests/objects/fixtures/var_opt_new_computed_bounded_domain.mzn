% Computed attribute with a total RHS but a BINDING declared domain
% (`var 3..4: z = x1 + x2`) on the singular `var opt new` arm.
% The lowering relocates the domain instead of value-guarding: the alias
% stays unguarded (`z = x1 + x2` on every slot), the `<A>_objects` element
% record carries `var int: z` (relaxed), and the declared domain is
% re-imposed on realised objects only via
% `forall(this in A)(this.z in 3..4)`. With the slot unrealised
% (`absent(a)`), `z = 2` at the pinned frees is unconstrained and the model
% stays SAT.
class A(var 0..2: x1; var 0..2: x2; var 3..4: z = x1 + x2);
var opt new A: a;
constraint absent(a);
solve satisfy;
output ["absent ok\n"];
