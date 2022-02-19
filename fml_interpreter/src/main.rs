#![feature(map_try_insert)]

mod interpreter;
mod ast;
use ast::AST;
use interpreter::interpret;

fn main() {
    let tree: AST = AST::variable(String::from("x"), AST::Integer(1));
    interpret(tree);
}
