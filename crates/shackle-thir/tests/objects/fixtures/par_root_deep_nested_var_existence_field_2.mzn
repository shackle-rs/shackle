class D ( 2..3: v; );
class C ( 1..3: ck; var set(1..2) of new D: ds; );
class A ( set(1..2) of new C: cs; );
new A: a = (cs: [(ck: 1), (ck: 2)]);
solve satisfy;
