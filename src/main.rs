use logic_resolution::error::Res;
use logic_resolution::repl;
use std::env;

fn main() -> Res<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        1 => repl::repl()?,
        2 => repl::rep(&args[1])?,
        _ => println!("Usage: {} [file]", args[0]),
    }
    Ok(())
}

// @todo Rc<[str]>
