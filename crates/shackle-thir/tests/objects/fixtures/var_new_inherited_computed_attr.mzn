% A singular `var new` root of a SUBCLASS whose superclass declares a
% computed attribute. The superclass inheritance projection previously read
% the raw root inputs, fresh-minted `c` as an unbounded free decision, and
% registered NOT determined — keeping S's class-body forall and the symmetry
% pins load-bearing. The projection is symmetric with the collection-root
% path: project from the already-reconstructed direct-class objects array
% (T_t_objects), whose `c` column is alias-defined, and inherit its
% determined flag — S's forall is dropped.
class S(var 0..3: b; var int: c = b + 1);
class T extends S(var 0..2: d);
var new T: t;
constraint t.b = 2;
constraint t.d = 1;
solve satisfy;
output ["cs=", show([fix(s.c) | s in S]), " d=", show(fix(t.d)), "\n"];
