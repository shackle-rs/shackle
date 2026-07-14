int: maxSeats = 2;

class Wagon(
  set(1..maxSeats) of Seat: seats;
);

class Seat(
  Wagon: wagon;
);

constraint forall(w in Wagon)(forall(s in w.seats)(s in s.wagon.seats));
solve satisfy;
