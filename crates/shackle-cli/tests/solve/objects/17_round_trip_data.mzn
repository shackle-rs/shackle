% The second half of a round trip: reads the solution of
% `17_round_trip_solution.mzn` as data.
class A ( 1..3: x; A: next; );
new A: one;
new A: two;
int: total :: output = 100 * one.x + 10 * one.next.x + two.x;
