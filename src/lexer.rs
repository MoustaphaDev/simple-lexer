// simple lexer for a simple language
// should be able to lex something like
// let value = 1 + 3 + 4;
// let name = name + ' ' + "hey you!";

mod character_helpers;
mod token;

use token::*;

#[derive(Debug, PartialEq)]
enum StringState {
    InSingleQuote,
    InDoubleQuote,
}

#[derive(Debug, PartialEq)]
enum State {
    Start,
    InNumber,
    InString(StringState),
    InIdentifier,
    InOperator,
}

#[derive(Debug, PartialEq)]
enum LexerErrorKind {
    InvalidToken,
    InvalidOperator,
}

#[derive(Debug, PartialEq)]
pub struct LexerError {
    span: Span,
    kind: LexerErrorKind,
}

pub struct ErrorHandler {
    errors: Vec<LexerError>,
}

pub struct Lexer<'a> {
    current_state: State,
    // byte index of the first character of the token being buffered
    buffered_token_start: usize,
    input: &'a String,
    /**
     * This is the index of the current character being processed
     * in the vector of characters, not the byte index of the
     * character in the input string
     * If you want the byte index of the character in the input string
     * use the current_code_point_byte_index value
     */
    cursor: usize,
    current_character_byte_index: usize,
    tokens: Vec<Token>,
    handler: &'a mut ErrorHandler,
}

impl ErrorHandler {
    pub fn new() -> Self {
        Self { errors: Vec::new() }
    }

    fn add_error(&mut self, error: LexerError) {
        self.errors.push(error);
    }
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a String, handler: &'a mut ErrorHandler) -> Self {
        Self {
            current_state: State::Start,
            buffered_token_start: 0,
            current_character_byte_index: 0,
            input: source,
            cursor: 0,
            tokens: Vec::new(),
            handler,
        }
    }
}

impl Lexer<'_> {
    fn change_state(&mut self, state: State) {
        self.current_state = state;
    }

    fn reset_state(&mut self) {
        self.current_state = State::Start;
    }
}

// state handlers
impl Lexer<'_> {
    fn handle_start(&mut self, character: char) {
        self.buffered_token_start = self.current_character_byte_index;

        if character_helpers::is_digit(character) {
            self.change_state(State::InNumber);
        } else if character_helpers::is_letter(character) {
            self.change_state(State::InIdentifier);
        } else if character_helpers::is_single_quote(character) {
            // don't buffer the opening quote
            self.advance_cursor();
            self.change_state(State::InString(StringState::InSingleQuote));
        } else if character_helpers::is_double_quote(character) {
            // don't buffer the opening quote
            self.advance_cursor();
            self.change_state(State::InString(StringState::InDoubleQuote));
        } else if character_helpers::is_operator(character) {
            self.change_state(State::InOperator);
        } else if character_helpers::is_semicolon(character) {
            let token = token::create_token(TokenKind::Semicolon, self.buffered_token_start, 1);
            self.consume_token_explicit(token);
            // the token was created and consumed on the spot
            // skip to the next character in the next iteration
            // of the state machine
            self.advance_cursor();
        } else if character_helpers::is_whitespace(character) {
            let token = token::create_token(TokenKind::Whitespace, self.buffered_token_start, 1);

            self.consume_token_explicit(token);
            self.advance_cursor();
        } else {
            // TODO: should I introduce an InError state
            // so its the state handler will take responsibility
            // on how to handle the errors?
            // meh idk 😅, I'll just handle it here for now
            let token = token::create_token(TokenKind::Invalid, self.buffered_token_start, 1);

            self.consume_token_explicit(token);
            self.advance_cursor();

            self.handler.add_error(LexerError {
                span: self.create_current_token_span(),
                kind: LexerErrorKind::InvalidToken,
            });
        }
    }

    fn handle_in_number(&mut self, character: char) {
        if character_helpers::is_digit(character) {
            self.advance_cursor();
        } else {
            self.consume_buffered_token();
            self.reset_state();
        }
    }

    fn handle_in_operator(&mut self, character: char) {
        // operators can be at most 2 characters long
        // len < 2 because the token's buffer is gonna grow by 1
        // in this code path
        if character_helpers::is_operator(character) && self.get_buffered_token().len() < 2 {
            self.advance_cursor();
        } else {
            self.consume_buffered_token();
            self.reset_state();
        }
    }

    fn handle_in_string(&mut self, character: char) {
        let is_closing_quote = if let State::InString(string_state) = &self.current_state {
            match string_state {
                StringState::InSingleQuote => character_helpers::is_single_quote,
                StringState::InDoubleQuote => character_helpers::is_double_quote,
            }
        } else {
            // if this handler is called, the current state
            // is without a doubt InString
            // if not, it's a bug, and the program should panic
            unreachable!();
        };

        if !is_closing_quote(character) {
            self.advance_cursor();
        } else {
            // don't reprocess the closing quote character
            self.advance_cursor();

            self.consume_buffered_token();
            self.reset_state();
        }
    }

    fn handle_in_identifier(&mut self, character: char) {
        if character_helpers::is_in_identifier(character) {
            self.advance_cursor();
        } else {
            // Consuming of keywords is hidden under this function
            // Something is an Identifier unless that
            // identifier matches a keyword
            self.consume_buffered_token();
            self.reset_state();
        }
    }
}

// lexer utilities
impl<'a> Lexer<'a> {
    pub fn lex(&'a mut self) -> &'a Vec<self::Token> {
        // TODO: could have a better data structure?
        let mut characters = self.input.char_indices().peekable();

        let mut advancement = 0;
        let mut current_group = characters.next();

        while current_group.is_some() {
            let (current_character_byte_index, current_character) =
                current_group.expect("This should never be None");
            self.current_character_byte_index = current_character_byte_index;

            match self.current_state {
                State::Start => self.handle_start(current_character),
                State::InIdentifier => self.handle_in_identifier(current_character),
                State::InString(_) => self.handle_in_string(current_character),
                State::InNumber => self.handle_in_number(current_character),
                State::InOperator => self.handle_in_operator(current_character),
            }

            let delta = self.cursor - advancement;
            for _ in 0..delta {
                current_group = characters.next();
                advancement += 1;
            }
        }

        // consume the last buffered token
        // if the state machine is still in a non-start state
        if self.current_state != State::Start {
            // advance the character index so that the last
            // character is included in the buffered token
            self.current_character_byte_index = self.input.len();
            self.consume_buffered_token()
        }

        &self.tokens
    }

    /**
     * Creates a span from buffered_token_start, which is the
     * byte index of the first character of the token being buffered
     */
    fn create_current_token_span(&self) -> Span {
        // if the buffered token is empty
        // we're only processing a single character
        let token_length = {
            let l = self.get_buffered_token().len();
            if l == 0 {
                1
            } else {
                l
            }
        };

        Span::new(self.buffered_token_start, token_length)
    }

    /**
     * Gets a slice from the input string that represents the buffered token
     * On a defined character index, the buffered token is the slice
     * from the buffered_token_start to the preceding character indice
     * (slices are exclusive on the end index)
     */
    fn get_buffered_token(&self) -> &str {
        // TODO: consider if you should introduce caching here
        // Could be a bigger of a concern when you just want to get the length of the buffered token
        // Will see, maybe I just don't understand enough how string slices work, and I'm overthinking it 🤷
        &self.input[self.buffered_token_start..self.current_character_byte_index]
    }

    fn advance_cursor(&mut self) {
        self.cursor += 1;
    }

    fn consume_buffered_token(&mut self) {
        let token_kind = match &self.current_state {
            State::InIdentifier => {
                // if the identifier matches a keyword,
                // consume the token as a keyword
                let buffered_token = self.get_buffered_token();
                if character_helpers::is_keyword(buffered_token) {
                    TokenKind::Keyword
                } else {
                    TokenKind::Identifier
                }
            }
            State::InString(string_state) => {

                // advance the character byte index so that the closing
                // quote is included in the buffered token
                self.current_character_byte_index += 1;

                match string_state {
                    StringState::InSingleQuote => {
                        TokenKind::String(StringKind::SingleQuoted)
                    }
                    StringState::InDoubleQuote => {
                        TokenKind::String(StringKind::DoubleQuoted)
                    }
                }
            },
            State::InNumber => TokenKind::Number,
            State::InOperator => {
                let buffered_token = self.get_buffered_token();
                let operator_kind = token::match_operator_slice_to_operator_kind(buffered_token);
                // if it's doesn't match any valid operator, it's a compound-like operator
                // We should split the operator in two, consume the first
                // part and then reprocess the second part
                match operator_kind {
                    OperatorKind::Invalid => {
                        self.handler.add_error(LexerError {
                            span: self.create_current_token_span(),
                            kind: LexerErrorKind::InvalidOperator,
                        });
                        let buffered_token= self.get_buffered_token();
                        let first_operator_slice = &buffered_token[0..1];
                        let first_operator_kind = token::match_operator_slice_to_operator_kind(first_operator_slice);

                        let first_token = token::create_token( TokenKind::Operator(first_operator_kind), self.buffered_token_start, 1);
                        self.consume_token_explicit(first_token);

                        self.buffered_token_start += 1;

                        let buffered_token= self.get_buffered_token();
                        let second_operator_slice = &buffered_token[0..1];
                        let second_operator_kind = token::match_operator_slice_to_operator_kind(second_operator_slice);
                        TokenKind::Operator(second_operator_kind)
                    },
                    _ => TokenKind::Operator(operator_kind),
                }
            },
            // NOTE: this arm will never be matched
            // it's a bug if it does
            State::Start => unreachable!("This function should never be called to buffer tokens when the lexer is in a `Start` state. Use `consume_token_explicit`"),
        };

        let token = Token {
            kind: token_kind,
            span: self.create_current_token_span(),
        };

        // the cursor is one character ahead of the last character
        // of the token
        // so the the start of the next token is the current cursor position
        self.tokens.push(token);
    }

    /**
     * Helper method to consume a token created on the fly
     * The cursor would likely need to to incremented
     * as the character/string would've been used to
     * create the token
     */
    fn consume_token_explicit(&mut self, token: Token) {
        self.tokens.push(token);
    }
}

// TODO: consider snapshot testing instead of fixtures
#[cfg(test)]
mod tests {
    use super::*;
    use similar_asserts::assert_eq;

    #[test]
    fn it_tokenizes_basic_number_assignment_correctly() {
        let source = String::from("let value = 1;");
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();

        // assert_eq!(tokens.len(), 8);
        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Identifier, 4, 5),
                token::create_token(TokenKind::Whitespace, 9, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Equal), 10, 1),
                token::create_token(TokenKind::Whitespace, 11, 1),
                token::create_token(TokenKind::Number, 12, 1),
                token::create_token(TokenKind::Semicolon, 13, 1),
            ]
        );
    }

    #[test]
    fn it_tokenizes_number_compound_assignment_correctly() {
        let source = String::from("let value += 1;");
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 8);
        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Identifier, 4, 5),
                token::create_token(TokenKind::Whitespace, 9, 1),
                token::create_token(TokenKind::Operator(OperatorKind::CompoundAdd), 10, 2),
                token::create_token(TokenKind::Whitespace, 12, 1),
                token::create_token(TokenKind::Number, 13, 1),
                token::create_token(TokenKind::Semicolon, 14, 1),
            ]
        );
    }

    #[test]
    fn it_tokenizes_invalid_operator_correctly_1() {
        let source = String::from("let value =+ 1;");
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 9);
        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Identifier, 4, 5),
                token::create_token(TokenKind::Whitespace, 9, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Equal), 10, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Add), 11, 1),
                token::create_token(TokenKind::Whitespace, 12, 1),
                token::create_token(TokenKind::Number, 13, 1),
                token::create_token(TokenKind::Semicolon, 14, 1),
            ]
        );
    }

    #[test]
    fn it_tokenizes_invalid_operator_correctly_2() {
        let source = String::from("let value %=+ 1;");
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 9);
        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Identifier, 4, 5),
                token::create_token(TokenKind::Whitespace, 9, 1),
                token::create_token(TokenKind::Operator(OperatorKind::CompoundModulo), 10, 2),
                token::create_token(TokenKind::Operator(OperatorKind::Add), 12, 1),
                token::create_token(TokenKind::Whitespace, 13, 1),
                token::create_token(TokenKind::Number, 14, 1),
                token::create_token(TokenKind::Semicolon, 15, 1),
            ]
        );
    }

    #[test]
    fn it_tokenizes_invalid_operator_correctly_3() {
        let source = String::from("let value ++++ 1;");
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 9);
        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Identifier, 4, 5),
                token::create_token(TokenKind::Whitespace, 9, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Increment), 10, 2),
                token::create_token(TokenKind::Operator(OperatorKind::Increment), 12, 2),
                token::create_token(TokenKind::Whitespace, 14, 1),
                token::create_token(TokenKind::Number, 15, 1),
                token::create_token(TokenKind::Semicolon, 16, 1),
            ]
        );
    }

    #[test]
    fn it_tokenizes_number_post_increment_correctly() {
        let source = String::from("let value = 1;\nvalue++;");
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 12);
        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Identifier, 4, 5),
                token::create_token(TokenKind::Whitespace, 9, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Equal), 10, 1),
                token::create_token(TokenKind::Whitespace, 11, 1),
                token::create_token(TokenKind::Number, 12, 1),
                token::create_token(TokenKind::Semicolon, 13, 1),
                token::create_token(TokenKind::Whitespace, 14, 1),
                token::create_token(TokenKind::Identifier, 15, 5),
                token::create_token(TokenKind::Operator(OperatorKind::Increment), 20, 2),
                token::create_token(TokenKind::Semicolon, 22, 1),
            ]
        );
    }

    #[test]
    fn it_tokenizes_cyrillic_strings_correctly() {
        let source = String::from("let greetings = 'привет мой друг';");
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 8);
        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Identifier, 4, 9),
                token::create_token(TokenKind::Whitespace, 13, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Equal), 14, 1),
                token::create_token(TokenKind::Whitespace, 15, 1),
                token::create_token(TokenKind::String(StringKind::SingleQuoted), 16, 30),
                token::create_token(TokenKind::Semicolon, 46, 1),
            ]
        );
    }

    #[test]
    fn it_tokenizes_source_with_string_concat_correctly() {
        let source = String::from("let word = \"Hello\" + \" \" + \"world!\"; ");
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 17);
        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Identifier, 4, 4),
                token::create_token(TokenKind::Whitespace, 8, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Equal), 9, 1),
                token::create_token(TokenKind::Whitespace, 10, 1),
                token::create_token(TokenKind::String(StringKind::DoubleQuoted), 11, 7),
                token::create_token(TokenKind::Whitespace, 18, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Add), 19, 1),
                token::create_token(TokenKind::Whitespace, 20, 1),
                token::create_token(TokenKind::String(StringKind::DoubleQuoted), 21, 3),
                token::create_token(TokenKind::Whitespace, 24, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Add), 25, 1),
                token::create_token(TokenKind::Whitespace, 26, 1),
                token::create_token(TokenKind::String(StringKind::DoubleQuoted), 27, 8),
                token::create_token(TokenKind::Semicolon, 35, 1),
                token::create_token(TokenKind::Whitespace, 36, 1),
            ]
        );
    }

    #[test]
    fn it_correctly_tokenizes_source_with_invalid_tokens() {
        let source = String::from("let @$` = &&| something something;");
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 16);

        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Invalid, 4, 1),
                token::create_token(TokenKind::Invalid, 5, 1),
                token::create_token(TokenKind::Invalid, 6, 1),
                token::create_token(TokenKind::Whitespace, 7, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Equal), 8, 1),
                token::create_token(TokenKind::Whitespace, 9, 1),
                token::create_token(TokenKind::Invalid, 10, 1),
                token::create_token(TokenKind::Invalid, 11, 1),
                token::create_token(TokenKind::Invalid, 12, 1),
                token::create_token(TokenKind::Whitespace, 13, 1),
                token::create_token(TokenKind::Identifier, 14, 9),
                token::create_token(TokenKind::Whitespace, 23, 1),
                token::create_token(TokenKind::Identifier, 24, 9),
                token::create_token(TokenKind::Semicolon, 33, 1),
            ]
        )
    }

    #[test]
    fn it_collects_expected_errors() {
        let source = String::from("let value =+ 1;\nlet @$` = &&| something something;");
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();
        assert_eq!(tokens.len(), 26);

        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Identifier, 4, 5),
                token::create_token(TokenKind::Whitespace, 9, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Equal), 10, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Add), 11, 1),
                token::create_token(TokenKind::Whitespace, 12, 1),
                token::create_token(TokenKind::Number, 13, 1),
                token::create_token(TokenKind::Semicolon, 14, 1),
                token::create_token(TokenKind::Whitespace, 15, 1),
                token::create_token(TokenKind::Keyword, 16, 3),
                token::create_token(TokenKind::Whitespace, 19, 1),
                token::create_token(TokenKind::Invalid, 20, 1),
                token::create_token(TokenKind::Invalid, 21, 1),
                token::create_token(TokenKind::Invalid, 22, 1),
                token::create_token(TokenKind::Whitespace, 23, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Equal), 24, 1),
                token::create_token(TokenKind::Whitespace, 25, 1),
                token::create_token(TokenKind::Invalid, 26, 1),
                token::create_token(TokenKind::Invalid, 27, 1),
                token::create_token(TokenKind::Invalid, 28, 1),
                token::create_token(TokenKind::Whitespace, 29, 1),
                token::create_token(TokenKind::Identifier, 30, 9),
                token::create_token(TokenKind::Whitespace, 39, 1),
                token::create_token(TokenKind::Identifier, 40, 9),
                token::create_token(TokenKind::Semicolon, 49, 1),
            ]
        );

        assert_eq!(handler.errors.len(), 7);
        assert_eq!(
            LexerError {
                span: Span {
                    start: 10,
                    length: 2,
                },
                kind: LexerErrorKind::InvalidOperator,
            },
            handler.errors[0]
        );

        assert_eq!(
            LexerError {
                span: Span {
                    start: 20,
                    length: 1,
                },
                kind: LexerErrorKind::InvalidToken,
            },
            handler.errors[1]
        );

        assert_eq!(
            LexerError {
                span: Span {
                    start: 21,
                    length: 1,
                },
                kind: LexerErrorKind::InvalidToken,
            },
            handler.errors[2]
        );

        assert_eq!(
            LexerError {
                span: Span {
                    start: 22,
                    length: 1,
                },
                kind: LexerErrorKind::InvalidToken,
            },
            handler.errors[3]
        );

        assert_eq!(
            LexerError {
                span: Span {
                    start: 26,
                    length: 1,
                },
                kind: LexerErrorKind::InvalidToken,
            },
            handler.errors[4]
        );

        assert_eq!(
            LexerError {
                span: Span {
                    start: 27,
                    length: 1,
                },
                kind: LexerErrorKind::InvalidToken,
            },
            handler.errors[5]
        );

        assert_eq!(
            LexerError {
                span: Span {
                    start: 28,
                    length: 1,
                },
                kind: LexerErrorKind::InvalidToken,
            },
            handler.errors[6]
        );
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
        let mut handler = ErrorHandler::new();
        let mut lexer = Lexer::new(&source, &mut handler);

        let tokens = lexer.lex();

        assert_eq!(tokens.len(), 7);

        assert_eq!(
            tokens,
            &vec![
                token::create_token(TokenKind::Keyword, 0, 3),
                token::create_token(TokenKind::Whitespace, 3, 1),
                token::create_token(TokenKind::Identifier, 4, 5),
                token::create_token(TokenKind::Whitespace, 9, 1),
                token::create_token(TokenKind::Operator(OperatorKind::Equal), 10, 1),
                token::create_token(TokenKind::Whitespace, 11, 1),
                token::create_token(TokenKind::Identifier, 12, 13),
            ]
        )
    }

    #[bench]
    fn test_bench(b: &mut test::Bencher) {
        b.iter(|| {
            let source = String::from("let value = 1;let value = 1;let value = 1;let value = 1;");
            let mut handler = ErrorHandler::new();
            let mut lexer = Lexer::new(&source, &mut handler);
            let _tokens = lexer.lex();
        });
    }
}
