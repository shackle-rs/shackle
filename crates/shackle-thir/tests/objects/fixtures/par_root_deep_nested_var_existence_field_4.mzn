class D ( 2..3: v; );
class C ( 1..3: ck; var set(1..2) of new D: ds; );
class B ( set(1..2) of new C: cs; );
class A ( set(1..2) of new B: bs; );
new A: a = (bs: [(cs: [(ck: 1), (ck: 2)])]);
solve satisfy;
