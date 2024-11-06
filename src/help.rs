pub fn help() -> &'static str {
    "\
Identifiers begin with letter or `_` and can also contain digits.
A formula can be inserted using the following operators:
    `~a`       -> \"not a\"
    `a & b`    -> \"a and b\"
    `a | b`    -> \"a or b\"
    `a => b`   -> \"a then b\"
    `a <=> b`  -> \"a if and only if b\"
    `a & (b <=> c)`
The `;` is optional:
    `~a; a & b` are 2 formulas
The precedences of the operators are in decreasing order:
    `!` `&` `|` `=>` `<=>`
There are some special operators and keywords:
    `!`        -> \"find box\"
    `?`        -> \"print formulas currently in use\"
    `-1`       -> \"delete formula_1\"
    `0 <=> ~1` -> \"formula_0 if and only if not formula_1\"
    `exit`     -> \"exit the program\"
    `help`     -> \"print this menu\"
The program can be called followed by an input file.\
"
}
