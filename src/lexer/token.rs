#[derive(Debug, PartialEq)]
pub enum Token {
    // NOTE: consider refactoring to add concrete tokens
    // instead of nesting information about the token in
    // its enum value
    String(StringType),
    Operator(OperatorType),
    Keyword(String),
    Number(String),
    Identifier(String),
    Whitespace(char),
    Semicolon,
    Invalid(String),
}

#[derive(Debug, PartialEq)]
pub enum OperatorType {
    // +
    Add,
    Substract,
    Multiply,
    Divide,
    Modulo,

    // +=
    CompoundAdd,
    CompoundSubstract,
    CompoundMultiply,
    CompoundDivide,
    CompoundModulo,

    // ++
    Increment,
    Decrement,

    // ==
    DoubleEqual,
    Equal,
    NotEqual,

    // !
    Not,

    // >
    GreaterThan,
    LessThan,
}

#[derive(Debug, PartialEq)]
pub enum StringType {
    SingleQuoted(String),
    DoubleQuoted(String),
}

pub fn match_operator_to_token(operator: String) -> Token {
    match operator.as_str() {
        // can be a simple operator
        "+" => Token::Operator(OperatorType::Add),
        "-" => Token::Operator(OperatorType::Substract),
        "*" => Token::Operator(OperatorType::Multiply),
        "/" => Token::Operator(OperatorType::Divide),
        "=" => Token::Operator(OperatorType::Equal),
        "%" => Token::Operator(OperatorType::Modulo),

        // can be a comparison operator
        "!=" => Token::Operator(OperatorType::NotEqual),
        "!" => Token::Operator(OperatorType::Not),
        ">" => Token::Operator(OperatorType::GreaterThan),
        "<" => Token::Operator(OperatorType::LessThan),

        // can be a compound operator
        "+=" => Token::Operator(OperatorType::CompoundAdd),
        "-=" => Token::Operator(OperatorType::CompoundSubstract),
        "*=" => Token::Operator(OperatorType::CompoundMultiply),
        "/=" => Token::Operator(OperatorType::CompoundDivide),
        "%=" => Token::Operator(OperatorType::CompoundModulo),
        "==" => Token::Operator(OperatorType::DoubleEqual),
        "++" => Token::Operator(OperatorType::Increment),
        "--" => Token::Operator(OperatorType::Decrement),

        // if it's doesn't match any of the above it's a compound-like operator
        // We should split the operator in two, consume the first
        // part and the reprocess the second part
        _ => Token::Invalid(operator),
    }
}
