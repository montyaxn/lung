mod lexer;
mod parser;
mod syntax;
mod type_def;
mod typing;

fn main() {
    let lexer = lexer::Lexer::from_file("test.txt");
}
