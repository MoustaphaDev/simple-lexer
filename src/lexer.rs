// simple lexer for a simple language
// should be able to lex something like
// value = 1 + 3 + 4;
// name = name + ' ' + "hey you!";
mod character_helpers;

#[derive(Debug, PartialEq)]
pub enum Token {
    Number(String),
    String(StringType),
    Identifier(String),
    Operator(String),
    Keyword(String),
    Whitespace(char),
    Semicolon,
    Invalid,
}

#[derive(Debug, PartialEq)]
pub enum StringType {
    SingleQuote(String),
    DoubleQuote(String),
}

enum StringState {
    SingleQuote,
    DoubleQuote,
}

enum State {
    Start,
    InNumber,
    InString(StringState),
    InIdentifier,
    InOperator,
}

pub struct Lexer {
    current_state: State,
    buffered_token: String,
    input: String,
    cursor: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            current_state: State::Start,
            buffered_token: String::new(),
            input: source,
            cursor: 0,
            tokens: Vec::new(),
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
            self.cursor += 1;
            self.change_state(State::InString(StringState::SingleQuote));
        } else if character_helpers::is_double_quote(char) {
            // don't buffer the opening quote
            self.cursor += 1;
            self.change_state(State::InString(StringState::DoubleQuote));
        } else if character_helpers::is_operator(char) {
            self.change_state(State::InOperator);
        } else if character_helpers::is_semicolon(char) {
            self.consume_token_explicit(Token::Semicolon);
            // the token was created and consumed on the spot
            // skip to the next character in the next iteration
            // of the state machine
            self.skip_current_char();
        } else if character_helpers::is_whitespace(char) {
            self.consume_token_explicit(Token::Whitespace(*char));
            self.skip_current_char();
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
        if character_helpers::is_operator(char) {
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
                StringState::SingleQuote => character_helpers::is_single_quote,
                StringState::DoubleQuote => character_helpers::is_double_quote,
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
            self.skip_current_char();
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

        &self.tokens
    }

    fn skip_current_char(&mut self) {
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
                StringState::SingleQuote => {
                    Token::String(StringType::SingleQuote(self.buffered_token.clone()))
                }
                StringState::DoubleQuote => {
                    Token::String(StringType::DoubleQuote(self.buffered_token.clone()))
                }
            },
            State::InNumber => Token::Number(self.buffered_token.clone()),
            State::InOperator => Token::Operator(self.buffered_token.clone()),
            // this should never be reached
            State::Start => Token::Invalid,
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
        self.cursor += 1;
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
                Token::Operator("=".to_string()),
                Token::Whitespace(' '),
                Token::Number("1".to_string()),
                Token::Semicolon,
            ]
        );
    }

    #[test]
    fn it_tokenizes_number_assignment_increment_correctly() {
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
                Token::Operator("+=".to_string()),
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
                Token::Operator("=".to_string()),
                Token::Whitespace(' '),
                Token::Number("1".to_string()),
                Token::Semicolon,
                Token::Whitespace('\n'),
                Token::Identifier("value".to_string()),
                Token::Operator("++".to_string()),
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
                Token::Operator("=".to_string()),
                Token::Whitespace(' '),
                Token::String(StringType::DoubleQuote("Hello".to_string())),
                Token::Whitespace(' '),
                Token::Operator("+".to_string()),
                Token::Whitespace(' '),
                Token::String(StringType::DoubleQuote(" ".to_string())),
                Token::Whitespace(' '),
                Token::Operator("+".to_string()),
                Token::Whitespace(' '),
                Token::String(StringType::DoubleQuote("world!".to_string())),
                Token::Semicolon,
                Token::Whitespace(' ')
            ]
        );
    }
}
