% A `var set of new A` root whose class has a *class-typed* field
% (`set of new B: children`) plus a computed attribute over it
% (`int: nChildren = card(children)`). The singular `var new` path already
% reconstructs the full storage record (computed field minted as a fresh
% decision + pinned by the class-body forall), but the `var set of new` path
% used its free `_storage` array *directly* as `<A>_objects` — and the computed
% field is excluded from that free array (`free_storage_record_ty`), so
% `<A>_objects` was missing `nChildren` entirely and every downstream
% `<A>_objects[x].nChildren` access (e.g. the symmetry-break default loop)
% panicked in `RecordAccess::build`. The set root now reconstructs the full
% record when a storage field is missing, symmetric with the singular path.
class B(0..3: x);
class A(
  set(0..3) of new B: children;
  int: nChildren = card(children);
  constraint card(children) = 2;
);

var set(1..1) of new A: as;
constraint card(as) = 1;
solve satisfy;
output ["ns=", show([fix(a.nChildren) | a in as]), "\n"];
