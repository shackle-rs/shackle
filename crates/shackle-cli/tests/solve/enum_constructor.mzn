% A constructor that takes an argument ranges over that argument's domain,
% which is an arbitrary expression and so cannot yet be resolved from the model.
% Values of such a type are shown by their position instead of by constructor:
% MiniZinc prints `f = Bar(1);` where this prints `f = to_enum(Foo, 1);`.
enum Foo = Bar(1..3);
var Foo: f;
solve satisfy;
