pub fn is_keyword(str: &str) -> bool {
    // mmk is a special keyword, it's my name abbreviated
    // not sure what to do with it rn
    matches!(
        str,
        "let" | "const" | "if" | "else" | "while" | "for" | "function" | "mmk"
    )
}

pub fn is_digit(char: &char) -> bool {
    char.is_ascii_digit()
}

pub fn is_letter(char: &char) -> bool {
    char.is_ascii_alphabetic()
}

// no bitwise or logical stuff for now
pub fn is_operator(char: &char) -> bool {
    matches!(char, '+' | '-' | '*' | '/' | '=' | '!' | '<' | '>' | '%')
}

pub fn is_single_quote(char: &char) -> bool {
    *char == '\''
}

pub fn is_double_quote(char: &char) -> bool {
    *char == '\"'
}

pub fn is_semicolon(char: &char) -> bool {
    *char == ';'
}

pub fn is_whitespace(char: &char) -> bool {
    char.is_whitespace()
}

// pub fn is_quote(char: &char) -> bool {
//     is_double_quote(char) || is_single_quote(char)
// }
//
// pub fn is_newline(char: &char) -> bool {
//     *char == '\n'
// }

pub fn is_in_identifier(char: &char) -> bool {
    char.is_ascii_alphanumeric() || *char == '_'
}
