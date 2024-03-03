use resolution_logic::repl;
use resolution_logic::Res;

fn main() -> Res<()> {
    repl::repl()?;
    Ok(())
}
