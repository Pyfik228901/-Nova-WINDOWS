use super::token::{Token, TokenType};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct LexerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Lexer error at line {}, col {}: {}", self.line, self.column, self.message)
    }
}

impl std::error::Error for LexerError {}

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<(usize, char)>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<(usize, char)> = source.char_indices().collect();
        Self {
            source,
            chars,
            pos: 0,
            line: 1,
            column: 1,
        }
    }

    fn current_byte_offset(&self) -> usize {
        self.chars.get(self.pos).map(|&(b, _)| b).unwrap_or(self.source.len())
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.advance();
                continue;
            }

            let start_byte = self.current_byte_offset();
            let start_line = self.line;
            let start_col = self.column;

            // Single-line or Multi-line comments
            if ch == '/' {
                if let Some(next) = self.peek_next_char() {
                    if next == '/' {
                        self.advance();
                        self.advance();
                        while let Some(c) = self.peek_char() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                        continue;
                    } else if next == '*' {
                        self.advance();
                        self.advance();
                        let mut depth = 1;
                        while depth > 0 {
                            if let Some(c) = self.peek_char() {
                                if c == '/' && self.peek_next_char() == Some('*') {
                                    self.advance();
                                    self.advance();
                                    depth += 1;
                                } else if c == '*' && self.peek_next_char() == Some('/') {
                                    self.advance();
                                    self.advance();
                                    depth -= 1;
                                } else {
                                    self.advance();
                                }
                            } else {
                                return Err(LexerError {
                                    message: "Unterminated multi-line comment /* ... */".to_string(),
                                    line: start_line,
                                    column: start_col,
                                });
                            }
                        }
                        continue;
                    }
                }
            }

            // String literals: "..." or '...'
            if ch == '"' || ch == '\'' {
                let s = self.read_string(ch, start_line, start_col)?;
                let end_byte = self.current_byte_offset();
                tokens.push(Token::new(
                    TokenType::String(s.clone()),
                    format!("\"{}\"", s),
                    start_line,
                    start_col,
                    (start_byte, end_byte),
                ));
                continue;
            }

            // Number literals: Int or Float
            if ch.is_ascii_digit() {
                let (token_type, lexeme) = self.read_number(start_line, start_col)?;
                let end_byte = self.current_byte_offset();
                tokens.push(Token::new(token_type, lexeme, start_line, start_col, (start_byte, end_byte)));
                continue;
            }

            // Identifiers and keywords
            if ch.is_alphabetic() || ch == '_' {
                let ident = self.read_ident();
                let end_byte = self.current_byte_offset();
                let token_type = match ident.as_str() {
                    "let" => TokenType::Let,
                    "print" => TokenType::Print,
                    "if" => TokenType::If,
                    "else" => TokenType::Else,
                    "while" => TokenType::While,
                    "for" => TokenType::For,
                    "in" => TokenType::In,
                    "func" => TokenType::Func,
                    "return" => TokenType::Return,
                    "import" => TokenType::Import,
                    "as" => TokenType::As,
                    "extern" => TokenType::Extern,
                    "true" => TokenType::True,
                    "false" => TokenType::False,
                    "null" => TokenType::Null,
                    _ => TokenType::Ident(ident.clone()),
                };
                tokens.push(Token::new(token_type, ident, start_line, start_col, (start_byte, end_byte)));
                continue;
            }

            // Two-character operators
            if let Some(next) = self.peek_next_char() {
                let two_op = match (ch, next) {
                    ('=', '=') => Some(TokenType::Eq),
                    ('!', '=') => Some(TokenType::NotEq),
                    ('<', '=') => Some(TokenType::LtEq),
                    ('>', '=') => Some(TokenType::GtEq),
                    ('&', '&') => Some(TokenType::And),
                    ('|', '|') => Some(TokenType::Or),
                    ('+', '=') => Some(TokenType::PlusAssign),
                    ('-', '=') => Some(TokenType::MinusAssign),
                    ('*', '=') => Some(TokenType::MulAssign),
                    ('/', '=') => Some(TokenType::DivAssign),
                    _ => None,
                };

                if let Some(tt) = two_op {
                    let lexeme = format!("{}{}", ch, next);
                    self.advance();
                    self.advance();
                    let end_byte = self.current_byte_offset();
                    tokens.push(Token::new(tt, lexeme, start_line, start_col, (start_byte, end_byte)));
                    continue;
                }
            }

            // Single-character tokens
            let single_op = match ch {
                '+' => Some(TokenType::Plus),
                '-' => Some(TokenType::Minus),
                '*' => Some(TokenType::Star),
                '/' => Some(TokenType::Slash),
                '%' => Some(TokenType::Percent),
                '=' => Some(TokenType::Assign),
                '<' => Some(TokenType::Lt),
                '>' => Some(TokenType::Gt),
                '!' => Some(TokenType::Not),
                '(' => Some(TokenType::LParen),
                ')' => Some(TokenType::RParen),
                '{' => Some(TokenType::LBrace),
                '}' => Some(TokenType::RBrace),
                '[' => Some(TokenType::LBracket),
                ']' => Some(TokenType::RBracket),
                ',' => Some(TokenType::Comma),
                '.' => Some(TokenType::Dot),
                ':' => Some(TokenType::Colon),
                ';' => Some(TokenType::Semicolon),
                _ => None,
            };

            if let Some(tt) = single_op {
                self.advance();
                let end_byte = self.current_byte_offset();
                tokens.push(Token::new(tt, ch.to_string(), start_line, start_col, (start_byte, end_byte)));
                continue;
            }

            return Err(LexerError {
                message: format!("Unexpected character '{}'", ch),
                line: start_line,
                column: start_col,
            });
        }

        let eof_byte = self.source.len();
        tokens.push(Token::new(TokenType::Eof, "", self.line, self.column, (eof_byte, eof_byte)));
        Ok(tokens)
    }

    fn peek_char(&self) -> Option<char> {
        self.chars.get(self.pos).map(|&(_, c)| c)
    }

    fn peek_next_char(&self) -> Option<char> {
        self.chars.get(self.pos + 1).map(|&(_, c)| c)
    }

    fn advance(&mut self) -> Option<char> {
        if let Some(&(_, c)) = self.chars.get(self.pos) {
            self.pos += 1;
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(c)
        } else {
            None
        }
    }

    fn read_string(&mut self, quote: char, start_line: usize, start_col: usize) -> Result<String, LexerError> {
        self.advance(); // skip opening quote
        let mut result = String::new();

        while let Some(ch) = self.peek_char() {
            if ch == quote {
                self.advance(); // skip closing quote
                return Ok(result);
            }

            if ch == '\\' {
                self.advance();
                match self.advance() {
                    Some('n') => result.push('\n'),
                    Some('r') => result.push('\r'),
                    Some('t') => result.push('\t'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('\'') => result.push('\''),
                    Some('0') => result.push('\0'),
                    Some(other) => {
                        result.push('\\');
                        result.push(other);
                    }
                    None => {
                        return Err(LexerError {
                            message: "Unterminated escape sequence in string literal".to_string(),
                            line: start_line,
                            column: start_col,
                        });
                    }
                }
            } else {
                result.push(ch);
                self.advance();
            }
        }

        Err(LexerError {
            message: format!("Unterminated string literal started with {}", quote),
            line: start_line,
            column: start_col,
        })
    }

    fn read_number(&mut self, _start_line: usize, _start_col: usize) -> Result<(TokenType, String), LexerError> {
        let mut s = String::new();
        let mut is_float = false;

        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                s.push(ch);
                self.advance();
            } else if ch == '.' && !is_float {
                if let Some(next) = self.peek_next_char() {
                    if next.is_ascii_digit() {
                        is_float = true;
                        s.push(ch);
                        self.advance();
                        continue;
                    }
                }
                break;
            } else {
                break;
            }
        }

        if is_float {
            let f = s.parse::<f64>().map_err(|e| LexerError {
                message: format!("Invalid float '{}': {}", s, e),
                line: self.line,
                column: self.column,
            })?;
            Ok((TokenType::Float(f), s))
        } else {
            let i = s.parse::<i64>().map_err(|e| LexerError {
                message: format!("Invalid integer '{}': {}", s, e),
                line: self.line,
                column: self.column,
            })?;
            Ok((TokenType::Int(i), s))
        }
    }

    fn read_ident(&mut self) -> String {
        let mut s = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_alphanumeric() || ch == '_' {
                s.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        s
    }
}
