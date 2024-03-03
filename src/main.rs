use logic_resolution::repl;
use logic_resolution::Res;

fn main() -> Res<()> {
    repl::repl()?;
    Ok(())
}
