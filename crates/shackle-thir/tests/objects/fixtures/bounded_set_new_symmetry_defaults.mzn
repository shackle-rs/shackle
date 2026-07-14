int: n;

class C (
  var 5..10: i;
  var bool: b;
  var opt int: o;
  var set of 0..3: s;
);

var set(0..n) of new C: cs;
solve satisfy;
