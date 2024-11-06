# Solver for Propositional Calculus
The program functions as a REPL (Read-Eval-Print Loop) designed to determine
whether a given set of clauses is contradictory.  
It implements the resolution method for mathematical logics and displays the
step-by-step process used to discover the contraddiction.

## Syntax
```
Identifiers begin with a letter or an `_` and can also contain digits.
A formula can be inserted using the following operators:
    `~a`       -> "not a"
    `a & b`    -> "a and b"
    `a | b`    -> "a or b"
    `a => b`   -> "a then b"
    `a <=> b`  -> "a if and only if b"
    `(`, `)`   -> "parenthesis"
The `;` is optional:
    `~a; a & b` are 2 formulas
The precedences of the operators are in decreasing order:
    `!` `&` `|` `=>` `<=>`
There are some special operators and keywords:
    `!`        -> "find box"
    `?`        -> "print formulas currently in use ()"
    `-1`       -> "delete formula_1"
    `0 <=> ~1` -> "formula_0 if and only if not formula_1"
    `exit`     -> "exit the program"
    `help`     -> "print this menu"
The program can be called followed by an input file.
```

## Example
```
$ cargo run -r
...
>> ~(A&B&C)
(~((A & B) & C))
>> A|(B|C)&~C
(A | ((B | C) & (~C)))
>> ?
0: (~((A & B) & C)) --> {{~A, ~B, ~C}}
1: (A | ((B | C) & (~C))) --> {{A, B, C}, {A, ~C}}
>> 1 & (~B|C) & ~(A&~B)
(((A | ((B | C) & (~C))) & ((~B) | C)) & (~(A & (~B))))
>> -1
Formula 1 removed.
>> !
Box found:
0: (~((A & B) & C)) --> {{~A, ~B, ~C}}
1: (((A | ((B | C) & (~C))) & ((~B) | C)) & (~(A & (~B)))) --> {{A, B, C}, {A, ~C}, {B, ~A}, {C, ~B}}
Proof:
{B, ~A}, {~A, ~B, ~C} -> {~A, ~C}
{C, ~B}, {B, ~A} -> {C, ~A}
{~A, ~C}, {C, ~A} -> {~A}
{C, ~B}, {A, B, C} -> {A, C}
{A, ~C}, {A, C} -> {A}
{~A}, {A} -> {}
>> exit
```
