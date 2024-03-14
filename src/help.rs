pub fn help() -> &'static str {
    "\
You can insert a formula using the following operators:
    `~a`       -> \"not a\"
    `a & b`    -> \"a and b\"
    `a | b`    -> \"a or b\"
    `a => b`   -> \"a then b\"
    `a <=> b`  -> \"a if and only if b\"
The `;` is optional:
    `~a; a & b` are 2 formulas
Paranthesis are valid syntax:
    `a & (b <=> c)`
The precedence of the operators are in decreasing order:
    `!` `&` `|` `=>` `<=>`
There exists some special operators and keywords:
    `!`        -> \"find box\"
    `?`        -> \"print formulas currently in use\"
    `-1`       -> \"delete formula_1\"
    `0 <=> ~1` -> \"formula_0 if and only if not formula_1\"
    `exit`     -> \"exit the program\"
    `help`     -> \"print this menu\"
You can call the program followed by the input file.\
"
}
