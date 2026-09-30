% Objects of two classes referring to each other, read from data, and a
% singular root read from data as a single record. The reference of the
% decision object `na` can only point at objects given in the data.
class A ( B: b; int: x; );
class B ( A: a; bool: y; );

set of new A: as;
set of new B: bs;
new A: first;
var new A: na;
var int: via_references :: output = first.b.a.x + sum (o in as) (o.b.a.x);
constraint na.x = first.x + sum (o in as) (o.x);
