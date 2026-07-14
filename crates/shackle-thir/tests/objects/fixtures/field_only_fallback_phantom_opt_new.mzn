% Family (a) of the field-only universe fallback: a singular `opt new`
% field introduction. H's actual set falls back to `H = H_potential`, so
% card(H) reports 1 even though the only introduction is forced to not
% occur. Correct semantics: H = {} and nH=0.
class W (
  opt new H: h;
);
class H ( var 0..1: x; );
constraint forall(w in W)(not occurs(w.h));
var new W: w;
output ["nH=", show(card(H))];
solve satisfy;
