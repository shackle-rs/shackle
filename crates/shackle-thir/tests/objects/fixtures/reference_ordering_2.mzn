class A ();
class B extends A ();
A: a;
B: b;
constraint a < b;
constraint a <= b;
constraint a > b;
constraint a >= b;
solve satisfy;
