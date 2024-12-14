mod lexer;
use lexer::{ErrorHandler, Lexer};
fn main() {
    let source = String::from("let word = \"Hello\" + \" \" + \"world!\"; ");
    let mut handler = ErrorHandler::new();
    let mut lexer = Lexer::new(source, &mut handler);

    let _tokens = lexer.lex();
    todo!();
}
