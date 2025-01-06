#[derive(Debug, PartialEq)]
pub struct Span {
    pub length: usize,
    pub start: usize,
}

#[derive(Debug, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, PartialEq)]
pub enum TokenKind {
    // NOTE: consider refactoring to add concrete tokens
    // instead of nesting information about the token in
    // its enum value
    String(StringKind),
    Operator(OperatorKind),
    Keyword,
    Number,
    Identifier,
    Whitespace,
    Semicolon,
    Invalid,
}

#[derive(Debug, PartialEq)]
pub enum StringKind {
    SingleQuoted,
    DoubleQuoted,
}

#[derive(Debug, PartialEq)]
pub enum OperatorKind {
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

    // Invalid operator
    Invalid,
}

impl Span {
    pub fn new(start: usize, length: usize) -> Self {
        Self { start, length }
    }
}

pub fn create_token(kind: TokenKind, start: usize, length: usize) -> Token {
    Token {
        kind,
        span: Span::new(start, length),
    }
}

pub fn match_operator_slice_to_operator_kind(operator: &str) -> OperatorKind {
    match operator {
        // can be a simple operator
        "+" => OperatorKind::Add,
        "-" => OperatorKind::Substract,
        "*" => OperatorKind::Multiply,
        "/" => OperatorKind::Divide,
        "=" => OperatorKind::Equal,
        "%" => OperatorKind::Modulo,

        // can be a comparison operator
        "!=" => OperatorKind::NotEqual,
        "!" => OperatorKind::Not,
        ">" => OperatorKind::GreaterThan,
        "<" => OperatorKind::LessThan,

        // can be a compound operator
        "+=" => OperatorKind::CompoundAdd,
        "-=" => OperatorKind::CompoundSubstract,
        "*=" => OperatorKind::CompoundMultiply,
        "/=" => OperatorKind::CompoundDivide,
        "%=" => OperatorKind::CompoundModulo,
        "==" => OperatorKind::DoubleEqual,
        "++" => OperatorKind::Increment,
        "--" => OperatorKind::Decrement,

        // if it's doesn't match any of the above it's a compound-like operator
        // We should split the operator in two, consume the first
        // part and the reprocess the second part
        _ => OperatorKind::Invalid,
    }
}
