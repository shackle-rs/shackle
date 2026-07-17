class D ( 2..3: v; );
class C ( set(1..1) of new D: ds; );
class B ( set(1..1) of new C: cs; );
class A ( set(1..1) of new B: bs; );
new A: a = (bs: [(cs: [(ds: [(v: 2)])])]);
var new C: c2;
solve satisfy;
