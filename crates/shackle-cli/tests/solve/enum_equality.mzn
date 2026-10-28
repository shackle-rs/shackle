% Equality of two var enum values used to recurse forever in MiniZinc: there
% was no `=` for `var $$E`, so it resolved to the generated `=` on any type,
% whose body calls `=` on the same type again.
enum Colour = {Red, Green, Blue};
var Colour: c;
var Colour: d;
constraint c = d;
constraint c != Green;
solve satisfy;
