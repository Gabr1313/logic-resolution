# Resolutor for Propositional Calculus
The program functions as a REPL (Read-Eval-Print Loop) designed to determine
whether a given set of clauses is contradictory.  
It employs the resolution method and displays the step-by-step process used
to uncover the proof.
### Example
```
❯ cargo run -r
Type `help`
>> ~(A&B&C)
 (~((A & B) & C))
>> A|(B|C)&~C
 (A | ((B | C) & (~C)))
>> ?
 0: (~((A & B) & C)) -> {{~A, ~B, ~C}}
 1: (A | ((B | C) & (~C))) -> {{A, B, C}, {A, ~C}}
>> (~B|C) & ~(A&~B) & 1
 ((((~B) | C) & (~(A & (~B)))) & (A | ((B | C) & (~C))))
>> -1
 Formula 1 removed.
>> !
 Box found:
 0: (~((A & B) & C)) -> {{~A, ~B, ~C}}
 1: ((((~B) | C) & (~(A & (~B)))) & (A | ((B | C) & (~C)))) -> {{A, B, C}, {A, ~C}, {B, ~A}, {C, ~B}}
 {B, ~A}, {~A, ~B, ~C} -> {~A, ~C}
 {C, ~B}, {B, ~A} -> {C, ~A}
 {~A, ~C}, {C, ~A} -> {~A}
 {C, ~B}, {A, B, C} -> {A, C}
 {A, ~C}, {A, C} -> {A}
 {~A}, {A} -> {}
>> exit
```
