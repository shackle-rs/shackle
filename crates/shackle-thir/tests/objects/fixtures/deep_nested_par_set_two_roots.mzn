class A (
  int: x;
);

class B (
  set of new A: a1;
  set of new A: a2;
);

class C (
  set of new B: b1;
  set of new B: b2;
);

set of new C: c1 = [
  (
    b1: [
      (a1: [(x: 1)], a2: [(x: 2), (x: 3)])
    ],
    b2: [
      (a1: [(x: 4), (x: 5)], a2: [])
    ]
  )
];

set of new C: c2 = [
  (
    b1: [
      (a1: [], a2: [(x: 6)])
    ],
    b2: [
      (a1: [(x: 7)], a2: [(x: 8)]),
      (a1: [(x: 9)], a2: [(x: 10), (x: 11)])
    ]
  )
];

solve satisfy;
