pub fn is_keyword(str: &str) -> bool {
    // mmk is a special keyword, it's my name abbreviated
    // not sure what to do with it rn
    matches!(
        str,
        "let" | "const" | "if" | "else" | "while" | "for" | "function" | "mmk"
    )
}

pub fn is_digit(grapheme: &str) -> bool {
    grapheme.chars().all(|char| char.is_ascii_digit())
}

pub fn is_letter(grapheme: &str) -> bool {
    grapheme.chars().all(|char| char.is_ascii_alphabetic())
}

// no bitwise or logical stuff for now
pub fn is_operator(grapheme: &str) -> bool {
    grapheme
        .chars()
        .all(|char| matches!(char, '+' | '-' | '*' | '/' | '=' | '!' | '<' | '>' | '%'))
}

pub fn is_single_quote(grapheme: &str) -> bool {
    grapheme.chars().all(|char| char == '\'')
}

pub fn is_double_quote(grapheme: &str) -> bool {
    grapheme.chars().all(|char| char == '\"')
}

pub fn is_semicolon(grapheme: &str) -> bool {
    grapheme.chars().all(|char| char == ';')
}

pub fn is_whitespace(grapheme: &str) -> bool {
    grapheme.chars().all(|char| char.is_whitespace())
}

pub fn is_in_identifier(grapheme: &str) -> bool {
    grapheme
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || char == '_')
}
