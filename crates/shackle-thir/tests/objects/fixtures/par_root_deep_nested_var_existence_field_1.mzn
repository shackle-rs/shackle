class D ( 2..3: v; );
class C ( 1..3: ck; var set(1..2) of new D: ds; );
new C: c = (ck: 1);
solve satisfy;
