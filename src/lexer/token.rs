#[derive(Debug, PartialEq)]
pub enum Token {
    Number(String),
    String(StringType),
    Identifier(String),
    Operator(OperatorType),
    Keyword(String),
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
    // +=
    CompoundAdd,
    CompoundSubstract,
    CompoundMultiply,
    CompoundDivide,
    // ++
    Increment,
    Decrement,

    // =
    Equal,

    // ==
    DoubleEqual,
}

#[derive(Debug, PartialEq)]
pub enum StringType {
    SingleQuoted(String),
    DoubleQuoted(String),
}

pub fn match_operator_to_token(operator: String) -> Token {
    if operator.len() == 1 {
        // can only be a simple operator
        match operator.as_str() {
            "+" => Token::Operator(OperatorType::Add),
            "-" => Token::Operator(OperatorType::Substract),
            "*" => Token::Operator(OperatorType::Multiply),
            "/" => Token::Operator(OperatorType::Divide),
            "=" => Token::Operator(OperatorType::Equal),
            _ => Token::Invalid(operator),
        }
    } else {
        // can be a compound operator
        match operator.as_str() {
            "+=" => Token::Operator(OperatorType::CompoundAdd),
            "++" => Token::Operator(OperatorType::Increment),
            "*=" => Token::Operator(OperatorType::CompoundMultiply),
            "/=" => Token::Operator(OperatorType::CompoundDivide),
            "-=" => Token::Operator(OperatorType::CompoundSubstract),
            "--" => Token::Operator(OperatorType::Decrement),
            "==" => Token::Operator(OperatorType::DoubleEqual),
            _ => Token::Invalid(operator),
        }
    }
}
