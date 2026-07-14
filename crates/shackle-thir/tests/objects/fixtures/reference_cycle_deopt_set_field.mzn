% Reference cycle between two var-reached classes: Seat references Handrail
% (`opt Handrail:`) while Handrail's reference-set field contains Seats
% (`set(1..1) of Seat:`). Classes are predeclared in topological item order,
% but a reference cycle has no valid order, so whichever class predeclares
% first used to keep an unsubstituted `Class<X>`-typed storage field —
% frozen into `<C>_objects` and panicking `lowered_ty_matches` when the
% top-level forall read `deopt(w.handrail).attached`.
% `repair_predeclared_class_objects_domains` rebuilds the storage domains
% once every class is registered.
class Wagon (
  set(1..1) of new Seat: seats;
  opt new Handrail: handrail;
);
class Handrail (
  set(1..1) of Seat: attached;
);
class Seat (
  opt Handrail: handrail;
  var 1..2: comfort;
);

constraint forall(w in Wagon)(occurs(w.handrail));
constraint forall(w in Wagon where occurs(w.handrail))(
  forall(s in deopt(w.handrail).attached)(s in w.seats /\ s.comfort = 2)
);

var new Wagon: w;
output ["comforts=", show([s.comfort | s in w.seats])];
solve satisfy;
