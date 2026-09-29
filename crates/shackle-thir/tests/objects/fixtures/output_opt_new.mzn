% Output of an optional owned child: MiniZinc has no `opt record`, so the
% `opt new` field is rendered as an array of zero or one records.
class Handrail ( 1..3: len; );
class Pole ( opt new Handrail: handrail; );
var new Pole: pole;
solve satisfy;
