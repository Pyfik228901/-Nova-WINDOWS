class TokenType:
    LET = 'LET'
    PRINT = 'PRINT'
    IF = 'IF'
    ELSE = 'ELSE'
    WHILE = 'WHILE'
    FUNC = 'FUNC'
    RETURN = 'RETURN'
    IDENT = 'IDENT'
    NUMBER = 'NUMBER'
    STRING = 'STRING'
    PLUS = '+'
    MINUS = '-'
    MUL = '*'
    DIV = '/'
    ASSIGN = '='
    EQ = '=='
    NEQ = '!='
    LT = '<'
    GT = '>'
    LPAREN = '('
    RPAREN = ')'
    LBRACE = '{'
    RBRACE = '}'
    LBRACKET = '['
    RBRACKET = ']'
    COMMA = ','
    EOF = 'EOF'

class Token:
    def __init__(self, type_, value, line):
        self.type = type_
        self.value = value
        self.line = line

    def __repr__(self):
        return f"Token({self.type}, {repr(self.value)}, line={self.line})"

class Lexer:
    def __init__(self, text):
        self.text = text
        self.pos = 0
        self.line = 1
        self.current_char = self.text[self.pos] if self.text else None

    def error(self):
        raise SyntaxError(f"Лексическая ошибка: неожиданный символ '{self.current_char}' в строке {self.line}")

    def advance(self):
        if self.current_char == '\n':
            self.line += 1
        self.pos += 1
        if self.pos < len(self.text):
            self.current_char = self.text[self.pos]
        else:
            self.current_char = None

    def peek(self):
        pos = self.pos + 1
        if pos < len(self.text):
            return self.text[pos]
        return None

    def skip_whitespace(self):
        while self.current_char is not None and self.current_char.isspace():
            self.advance()

    def skip_comment(self):
        while self.current_char is not None and self.current_char != '\n':
            self.advance()

    def number(self):
        result = ''
        is_float = False
        while self.current_char is not None and (self.current_char.isdigit() or self.current_char == '.'):
            if self.current_char == '.':
                if is_float:
                    self.error()
                is_float = True
            result += self.current_char
            self.advance()
        return float(result) if is_float else int(result)

    def string(self):
        quote = self.current_char
        self.advance()
        result = ''
        while self.current_char is not None and self.current_char != quote:
            result += self.current_char
            self.advance()
        if self.current_char is None:
            raise SyntaxError(f"Незакрытая строка в строке {self.line}")
        self.advance()
        return result

    def tokenize(self):
        tokens = []
        while self.current_char is not None:
            if self.current_char.isspace():
                self.skip_whitespace()
                continue

            if self.current_char == '/' and self.peek() == '/':
                self.advance()
                self.advance()
                self.skip_comment()
                continue

            if self.current_char.isalpha() or self.current_char == '_':
                start_line = self.line
                val = ''
                while self.current_char is not None and (self.current_char.isalnum() or self.current_char == '_'):
                    val += self.current_char
                    self.advance()
                
                keywords = {
                    'let': TokenType.LET,
                    'print': TokenType.PRINT,
                    'if': TokenType.IF,
                    'else': TokenType.ELSE,
                    'while': TokenType.WHILE,
                    'func': TokenType.FUNC,
                    'return': TokenType.RETURN,
                }
                t_type = keywords.get(val, TokenType.IDENT)
                tokens.append(Token(t_type, val if t_type == TokenType.IDENT else None, start_line))
                continue

            if self.current_char.isdigit():
                start_line = self.line
                val = self.number()
                tokens.append(Token(TokenType.NUMBER, val, start_line))
                continue

            if self.current_char in ('"', "'"):
                start_line = self.line
                val = self.string()
                tokens.append(Token(TokenType.STRING, val, start_line))
                continue

            if self.current_char == '=':
                start_line = self.line
                self.advance()
                if self.current_char == '=':
                    self.advance()
                    tokens.append(Token(TokenType.EQ, '==', start_line))
                else:
                    tokens.append(Token(TokenType.ASSIGN, '=', start_line))
                continue

            if self.current_char == '!':
                start_line = self.line
                self.advance()
                if self.current_char == '=':
                    self.advance()
                    tokens.append(Token(TokenType.NEQ, '!=', start_line))
                else:
                    self.error()
                continue

            if self.current_char == '<':
                start_line = self.line
                self.advance()
                tokens.append(Token(TokenType.LT, '<', start_line))
                continue

            if self.current_char == '>':
                start_line = self.line
                self.advance()
                tokens.append(Token(TokenType.GT, '>', start_line))
                continue

            single_char_tokens = {
                '+': TokenType.PLUS,
                '-': TokenType.MINUS,
                '*': TokenType.MUL,
                '/': TokenType.DIV,
                '(': TokenType.LPAREN,
                ')': TokenType.RPAREN,
                '{': TokenType.LBRACE,
                '}': TokenType.RBRACE,
                '[': TokenType.LBRACKET,
                ']': TokenType.RBRACKET,
                ',': TokenType.COMMA,
            }

            if self.current_char in single_char_tokens:
                start_line = self.line
                t_type = single_char_tokens[self.current_char]
                self.advance()
                tokens.append(Token(t_type, None, start_line))
                continue

            self.error()

        tokens.append(Token(TokenType.EOF, None, self.line))
        return tokens