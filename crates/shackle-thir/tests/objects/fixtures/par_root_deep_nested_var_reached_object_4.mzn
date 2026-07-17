int: n = 2;
class D ( 2..3: v; );
class C ( set(1..2) of new D: ds; );
class B ( set(1..2) of new C: cs; );
class A ( set(1..2) of new B: bs; );
new A: a = (bs: [(cs: [(ds: [(v: 2)])])]);
var set(0..n) of new C: cs2;
solve satisfy;
