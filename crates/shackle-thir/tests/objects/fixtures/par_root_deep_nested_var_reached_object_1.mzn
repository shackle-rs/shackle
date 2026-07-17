class D ( 2..3: v; );
class C ( set(1..1) of new D: ds; );
class B ( new C: c; );
class A ( new B: b; );
new A: a = (b: (c: (ds: [(v: 2)])));
var new C: c2;
solve satisfy;
