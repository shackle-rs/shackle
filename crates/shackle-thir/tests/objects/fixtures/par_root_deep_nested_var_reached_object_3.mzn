class D ( 2..3: v; );
class C ( var set(1..2) of new D: ds; );
class B ( new C: c; );
class A ( new B: b; );
new A: a = (b: (c: ( )));
var new C: c2;
solve satisfy;
