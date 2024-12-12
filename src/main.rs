mod lexer;
use lexer::Lexer;
fn main() {
    let source = String::from("let word = \"Hello\" + \" \" + \"world!\"; ");
    let mut lexer = Lexer::new(source);

    let _tokens = lexer.lex();
    todo!();
}
