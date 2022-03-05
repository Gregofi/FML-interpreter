
mod interpreter;
mod ast;
mod heap;

use ast::AST;
use interpreter::interpret;
use std::fs;
use std::env;

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();
    if args[1] != "run" {
        ()
    }
    let program = fs::read_to_string(&args[2])?;
    let tree: AST = serde_json::from_str(&program).unwrap();
    interpret(tree);
    Ok(())
}
