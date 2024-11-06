# Resolutor for Propositional Calculus
The program functions as a REPL (Read-Eval-Print Loop) designed to determine
whether a given set of clauses is contradictory.  
It implements the resolution method for mathematical logics and displays the
step-by-step process used to discover the contraddiction.
## Syntax
A `formula` is written using the following symbols (ordered by priority):
- `~` is a `not`
- `&` is an `and`
- `|` is an `or`
- `=>` is an `imply`
- `<=>` is an `if and only if`
In the REPL you can:
- Insert a formula: type it. It will be displayed how the formula is recognized
  by the system.  
  Example: `a <=> ~b`
- If perhaps you would like to insert more than one formula on the same line
  use `;`.  
  Example: `a <=> b; c`
- Print informations about inserted formulae: type `?`.  
  Example: `?`
- Combine formulae: you can alias formulae with the number they are associated 
  with.  
  Example: `1 & d | ~e`
- Remove a formula: type `-x` to remove the formula number x.  
  Example: `-2`
- Try to find box (a contraddiction): type `!`. If it is found, a backtrace
  will be displayed.  
  Example: `!`

### Example
```
❯ cargo run -r
...
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
