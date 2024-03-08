use logic_resolution::repl;
use logic_resolution::error::Res;

fn main() -> Res<()> {
    repl::repl()
}
