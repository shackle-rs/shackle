% The first half of a round trip: the objects of a solution, printed in the
% data form, are read back as data by `17_round_trip_data.mzn`.
class A ( 1..3: x; A: next; );
var new A: one;
var new A: two;
constraint one.x < two.x;
