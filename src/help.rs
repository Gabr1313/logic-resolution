pub fn help() -> &'static str {
    "\
You can insert a formula using the following operators:
    `~a;`       means \"not a\"
    `a & b;`    means \"a and b\"
    `a | b;`    means \"a or b\"
    `a => b;`   means \"a then b\"
    `a <=> b;`  means \"a if and only if b\"
Tip: don't forget the `;`
Paranthesis are valid syntax:
    `a & (b <=> c)`
The precedence of the operators is in decreasing order:
    `!` `&` `|` `=>` `<=>`
There also exists some special operators and keywords:
    `!`         means \"find box\" 
    `?`         means \"print formulas in use\" 
    `-1;`       means \"delete formula_1\" 
    `0 <=> ~1;` means \"formula_0 if and only if not formula_1\"
    `exit;`     means \"exit the program\"
    `help;`     means \"print this menu\"
Finally you can call the program followed by file_name\
"
}
