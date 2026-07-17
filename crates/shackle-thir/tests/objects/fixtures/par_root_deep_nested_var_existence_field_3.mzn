class D ( 2..3: v; );
class C ( 1..3: ck; var set(1..2) of new D: ds; );
class A ( new C: c; );
set of new A: as = [(c: (ck: 1)), (c: (ck: 2))];
solve satisfy;
