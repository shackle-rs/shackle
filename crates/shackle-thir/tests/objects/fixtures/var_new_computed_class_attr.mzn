% A singular `var new A` root whose class has a *class-typed* field
% (`set of new B: children`) plus a computed attribute over it
% (`int: nChildren = card(children)`). The class-typed field is a
% `<B>_potential` identity held in A's free `_storage`, so the
% storage-reconstruction alias chain reads it straight through and *defines*
% the computed field as `nChildren = card(children)` — a genuine alias, not a
% free `var int` decision pinned by the class-body forall. `card(children) = 2`
% pins `nChildren = 2`.
class B(0..3: x);
class A(
  set(0..3) of new B: children;
  int: nChildren = card(children);
  constraint card(children) = 2;
);

var new A: a;
solve satisfy;
output ["n=", show(fix(a.nChildren)), "\n"];
