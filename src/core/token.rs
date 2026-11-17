use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Let,
    Print,
    If,
    Else,
    While,
    For,
    In,
    Func,
    Return,
    Import,
    As,
    Extern,
    True,
    False,
    Null,

    // Literals
    Ident(String),
    Int(i64),
    Float(f64),
    String(String),

    // Operators
    Plus,       // +
    Minus,      // -
    Star,       // *
    Slash,      // /
    Percent,    // %
    Assign,     // =
    PlusAssign, // +=
    MinusAssign,// -=
    MulAssign,  // *=
    DivAssign,  // /=
    Eq,         // ==
    NotEq,      // !=
    Lt,         // <
    Gt,         // >
    LtEq,       // <=
    GtEq,       // >=
    And,        // &&
    Or,         // ||
    Not,        // !

    // Punctuation
    LParen,     // (
    RParen,     // )
    LBrace,     // {
    RBrace,     // }
    LBracket,   // [
    RBracket,   // ]
    Comma,      // ,
    Dot,        // .
    Colon,      // :
    Semicolon,  // ;

    Eof,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
    pub span: (usize, usize),
}

impl Token {
    pub fn new(
        token_type: TokenType,
        lexeme: impl Into<String>,
        line: usize,
        column: usize,
        span: (usize, usize),
    ) -> Self {
        Self {
            token_type,
            lexeme: lexeme.into(),
            line,
            column,
            span,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Token({:?}, '{}', line={}, col={}, span={:?})",
            self.token_type, self.lexeme, self.line, self.column, self.span
        )
    }
}
