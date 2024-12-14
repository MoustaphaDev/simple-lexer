// simple lexer for a simple language
// should be able to lex something like
// value = 1 + 3 + 4;
// name = name + ' ' + "hey you!";

// The lexer is very clone and casting heavy
// I'll solve performance issues when they arise

mod character_helpers;
mod token;

use token::*;

enum StringState {
    InSingleQuote,
    InDoubleQuote,
}

enum State {
    Start,
    InNumber,
    InString(StringState),
    InIdentifier,
    InOperator,
}

#[derive(Debug, PartialEq)]
pub struct Span {
    start: usize,
    length: usize,
}

#[derive(Debug, PartialEq)]
pub struct LexerError {
    span: Span,
    message: String,
}

pub struct ErrorHandler {
    errors: Vec<LexerError>,
}

impl ErrorHandler {
    fn new() -> Self {
        Self { errors: Vec::new() }
    }

    fn add_error(&mut self, error: LexerError) {
        self.errors.push(error);
    }
}

pub struct Lexer {
    current_state: State,
    buffered_token: String,
    input: String,
    cursor: usize,
    tokens: Vec<Token>,
    handler: ErrorHandler,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            current_state: State::Start,
            buffered_token: String::new(),
            input: source,
            cursor: 0,
            tokens: Vec::new(),
            handler: ErrorHandler::new(),
        }
    }
}

impl Lexer {
    // something is an Identifier
    // until we're proven wrong and that the
    // identifier matches a keyword

    fn change_state(&mut self, state: State) {
        self.current_state = state;
    }

    fn reset_state(&mut self) {
        self.current_state = State::Start;
    }
}

// state handlers
impl Lexer {
    fn handle_start(&mut self, char: &char) {
        if character_helpers::is_digit(char) {
            self.change_state(State::InNumber);
        } else if character_helpers::is_letter(char) {
            self.change_state(State::InIdentifier);
        } else if character_helpers::is_single_quote(char) {
            // don't buffer the opening quote
            self.advance_cursor();
            self.change_state(State::InString(StringState::InSingleQuote));
        } else if character_helpers::is_double_quote(char) {
            // don't buffer the opening quote
            self.advance_cursor();
            self.change_state(State::InString(StringState::InDoubleQuote));
        } else if character_helpers::is_operator(char) {
            self.change_state(State::InOperator);
        } else if character_helpers::is_semicolon(char) {
            self.consume_token_explicit(Token::Semicolon);
            // the token was created and consumed on the spot
            // skip to the next character in the next iteration
            // of the state machine
            self.advance_cursor();
        } else if character_helpers::is_whitespace(char) {
            self.consume_token_explicit(Token::Whitespace(*char));
            self.advance_cursor();
        } else {
            // TODO: should I introduce an InError state
            // so its the state handler will take responsibility
            // on how to handle the errors?
            // meh idk 😅, I'll just handle it here for now
            self.consume_token_explicit(Token::Invalid(char.to_string()));
            self.advance_cursor();

            self.handler.add_error(LexerError {
                span: self.create_span(),
                message: format!("Invalid token: `{char}`"),
            });
        }
    }

    fn handle_in_number(&mut self, char: &char) {
        if character_helpers::is_digit(char) {
            self.buffer_token(*char);
        } else {
            self.consume_buffered_token();
            self.reset_state();
        }
    }

    fn handle_in_operator(&mut self, char: &char) {
        // operators can be at most 2 characters long
        // len < 2 because if this branch is reached
        // the token's buffer is gonna grow by 1
        if character_helpers::is_operator(char) && self.buffered_token.len() < 2 {
            self.buffer_token(*char)
        } else {
            self.consume_buffered_token();
            self.reset_state();
        }
    }

    fn handle_in_string(&mut self, char: &char) {
        let is_closing_quote;
        if let State::InString(string_state) = &self.current_state {
            is_closing_quote = match string_state {
                StringState::InSingleQuote => character_helpers::is_single_quote,
                StringState::InDoubleQuote => character_helpers::is_double_quote,
            };
        } else {
            return;
        }

        // as long as we don't reach the end of the quote
        if !is_closing_quote(char) {
            self.buffer_token(*char);
        } else {
            // don't reprocess the closing quote character
            // The closing quote character doesn't need to be
            // stored in the token
            // We already have information about the nature
            // of the string in the token itself
            self.advance_cursor();
            self.consume_buffered_token();
            self.reset_state();
        }
    }

    fn handle_in_identifier(&mut self, char: &char) {
        if character_helpers::is_in_identifier(char) {
            self.buffer_token(*char);
        } else {
            // consuming of keywords is hidden under this function
            self.consume_buffered_token();
            self.reset_state();
        }
    }
}

// lexer utilities
impl Lexer {
    pub fn lex(&mut self) -> &Vec<self::Token> {
        // TODO: could have a better data structure?
        let chars: Vec<char> = self.input.chars().collect();
        while self.cursor < chars.len() {
            let current_char = chars.get(self.cursor).unwrap();

            match self.current_state {
                State::Start => self.handle_start(current_char),
                State::InIdentifier => self.handle_in_identifier(current_char),
                State::InString(_) => self.handle_in_string(current_char),
                State::InNumber => self.handle_in_number(current_char),
                State::InOperator => self.handle_in_operator(current_char),
            }
        }

        // consume the last buffered token
        // if the state machine is still in a non-start state
        match self.current_state {
            State::Start => {}
            _ => self.consume_buffered_token(),
        }

        &self.tokens
    }

    /**
     * Creates a Span from the current cursor position
     * Assumes the cursor is one character ahead
     * of the last character of the token
     */
    fn create_span(&self) -> Span {
        // if the buffered token is empty
        // we're only processing a single character
        let token_length = if self.buffered_token.is_empty() {
            1
        } else {
            self.buffered_token.len()
        };

        let end = self.cursor - 1;

        Span {
            start: end - token_length + 1,
            length: token_length,
        }
    }

    fn advance_cursor(&mut self) {
        self.cursor += 1;
    }

    fn consume_buffered_token(&mut self) {
        let token = match &self.current_state {
            State::InIdentifier => {
                // if the identifier matches a keyword,
                // consume the token as a keyword
                if character_helpers::is_keyword(&self.buffered_token) {
                    Token::Keyword(self.buffered_token.clone())
                } else {
                    Token::Identifier(self.buffered_token.clone())
                }
            }
            State::InString(string_state) => match string_state {
                StringState::InSingleQuote => {
                    Token::String(StringType::SingleQuoted(self.buffered_token.clone()))
                }
                StringState::InDoubleQuote => {
                    Token::String(StringType::DoubleQuoted(self.buffered_token.clone()))
                }
            },
            State::InNumber => Token::Number(self.buffered_token.clone()),
            State::InOperator => {
                let token = token::match_operator_to_token(self.buffered_token.clone());
                // if it's doesn't match any valid operator, it's a compound-like operator
                // We should split the operator in two, consume the first
                // part and then reprocess the second part
                if let Token::Invalid(operator) = token {
                    self.handler.add_error(LexerError {
                        span: self.create_span(),
                        message: format!("Invalid operator: `{}`", operator),
                    });

                    let mut operators_split = operator.chars();
                    self.consume_token_explicit(match_operator_to_token(operators_split.next().unwrap().to_string()));
                    match_operator_to_token(operators_split.next().unwrap().to_string())

                } else {
                    token
                }
            },
            // TODO: this should never be reached
            // not sure if panicking is right thought
            // I'l leave it as is for now
            State::Start => unreachable!("This function should never be called to buffered tokens in the Start state. Use `consume_token_explicit`"),
        };

        self.tokens.push(token);
        self.buffered_token.clear();
    }

    /**
     * Helper function to consume a token created on the fly
     * The cursor would likely need to to incremented
     * as the character/string would've been used to
     * create the token
     */
    fn consume_token_explicit(&mut self, token: Token) {
        self.tokens.push(token);
    }

    fn buffer_token(&mut self, char: char) {
        self.buffered_token.push(char);
        self.advance_cursor();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_tokenizes_basic_number_assignment_correctly() {
        let source = String::from("let value = 1;");
        let mut lexer = Lexer::new(source);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 8);
        assert_eq!(
            tokens,
            &vec![
                Token::Keyword("let".to_string()),
                Token::Whitespace(' '),
                Token::Identifier("value".to_string()),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::Equal),
                Token::Whitespace(' '),
                Token::Number("1".to_string()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn it_tokenizes_number_compound_assignment_correctly() {
        let source = String::from("let value += 1;");
        let mut lexer = Lexer::new(source);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 8);
        assert_eq!(
            tokens,
            &vec![
                Token::Keyword("let".to_string()),
                Token::Whitespace(' '),
                Token::Identifier("value".to_string()),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::CompoundAdd),
                Token::Whitespace(' '),
                Token::Number("1".to_string()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn it_tokenizes_invalid_operator_correctly_1() {
        let source = String::from("let value =+ 1;");
        let mut lexer = Lexer::new(source);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 9);
        assert_eq!(
            tokens,
            &vec![
                Token::Keyword("let".to_string()),
                Token::Whitespace(' '),
                Token::Identifier("value".to_string()),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::Equal),
                Token::Operator(OperatorType::Add),
                Token::Whitespace(' '),
                Token::Number("1".to_string()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn it_tokenizes_invalid_operator_correctly_2() {
        let source = String::from("let value %=+ 1;");
        let mut lexer = Lexer::new(source);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 9);
        assert_eq!(
            tokens,
            &vec![
                Token::Keyword("let".to_string()),
                Token::Whitespace(' '),
                Token::Identifier("value".to_string()),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::CompoundModulo),
                Token::Operator(OperatorType::Add),
                Token::Whitespace(' '),
                Token::Number("1".to_string()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn it_tokenizes_invalid_operator_correctly_3() {
        let source = String::from("let value ++++ 1;");
        let mut lexer = Lexer::new(source);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 9);
        assert_eq!(
            tokens,
            &vec![
                Token::Keyword("let".to_string()),
                Token::Whitespace(' '),
                Token::Identifier("value".to_string()),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::Increment),
                Token::Operator(OperatorType::Increment),
                Token::Whitespace(' '),
                Token::Number("1".to_string()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn it_tokenizes_number_post_increment_correctly() {
        let source = String::from("let value = 1;\nvalue++;");
        let mut lexer = Lexer::new(source);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 12);
        assert_eq!(
            tokens,
            &vec![
                Token::Keyword("let".to_string()),
                Token::Whitespace(' '),
                Token::Identifier("value".to_string()),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::Equal),
                Token::Whitespace(' '),
                Token::Number("1".to_string()),
                Token::Semicolon,
                Token::Whitespace('\n'),
                Token::Identifier("value".to_string()),
                Token::Operator(OperatorType::Increment),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn it_tokenizes_cyrillic_strings_correctly() {
        let source = String::from("let greetings = 'привет мой друг';");
        let mut lexer = Lexer::new(source);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 8);
        assert_eq!(
            tokens,
            &vec![
                Token::Keyword("let".to_string()),
                Token::Whitespace(' '),
                Token::Identifier("greetings".to_string()),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::Equal),
                Token::Whitespace(' '),
                Token::String(StringType::SingleQuoted("привет мой друг".to_string())),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn it_tokenizes_source_with_string_concat_correctly() {
        let source = String::from("let word = \"Hello\" + \" \" + \"world!\"; ");
        let mut lexer = Lexer::new(source);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 17);
        assert_eq!(
            tokens,
            &vec![
                Token::Keyword("let".to_string()),
                Token::Whitespace(' '),
                Token::Identifier("word".to_string()),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::Equal),
                Token::Whitespace(' '),
                Token::String(StringType::DoubleQuoted("Hello".to_string())),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::Add),
                Token::Whitespace(' '),
                Token::String(StringType::DoubleQuoted(" ".to_string())),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::Add),
                Token::Whitespace(' '),
                Token::String(StringType::DoubleQuoted("world!".to_string())),
                Token::Semicolon,
                Token::Whitespace(' ')
            ]
        );
    }

    #[test]
    fn it_correctly_tokenizes_source_with_invalid_tokens() {
        let source = String::from("let @$` = &&| something something;");
        let mut lexer = Lexer::new(source);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 16);

        assert_eq!(
            tokens,
            &vec![
                Token::Keyword("let".to_string()),
                Token::Whitespace(' '),
                Token::Invalid('@'.to_string()),
                Token::Invalid('$'.to_string()),
                Token::Invalid('`'.to_string()),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::Equal),
                Token::Whitespace(' '),
                Token::Invalid('&'.to_string()),
                Token::Invalid('&'.to_string()),
                Token::Invalid('|'.to_string()),
                Token::Whitespace(' '),
                Token::Identifier("something".to_string()),
                Token::Whitespace(' '),
                Token::Identifier("something".to_string()),
                Token::Semicolon,
            ]
        )
    }

    #[test]
    fn it_correctly_tokenizes_source_when_lexer_state_machine_ends_in_a_non_start_state() {
        // Here the lexer's state machine will end in a non-start state
        // more precisely in the InIdentifier state
        // That's because the identifier is at the end of the source
        // and its corresponding handler will only consume the buffered
        // token when it encounters a non-identifier character,
        // which doesn't happen in this case
        // It requires special handling (to consume the buffered token when
        // lexing ends in a non-start state), but this makes the
        // handlers' code much simpler
        let source = String::from("let value = another_value");
        let mut lexer = Lexer::new(source);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 7);

        assert_eq!(
            tokens,
            &vec![
                Token::Keyword("let".to_string()),
                Token::Whitespace(' '),
                Token::Identifier("value".to_string()),
                Token::Whitespace(' '),
                Token::Operator(OperatorType::Equal),
                Token::Whitespace(' '),
                Token::Identifier("another_value".to_string()),
            ]
        )
    }
}
