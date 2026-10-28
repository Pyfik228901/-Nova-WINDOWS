from nova.lexer import Token, TokenType

class AST:
    pass

class Program(AST):
    def __init__(self, statements):
        self.statements = statements

class VarAssign(AST):
    def __init__(self, name, expr):
        self.name = name
        self.expr = expr

class PrintNode(AST):
    def __init__(self, expr):
        self.expr = expr

class IfNode(AST):
    def __init__(self, condition, then_block, else_block=None):
        self.condition = condition
        self.then_block = then_block
        self.else_block = else_block

class WhileNode(AST):
    def __init__(self, condition, block):
        self.condition = condition
        self.block = block

class FuncDef(AST):
    def __init__(self, name, params, body):
        self.name = name
        self.params = params
        self.body = body

class ReturnNode(AST):
    def __init__(self, expr):
        self.expr = expr

class FunctionCall(AST):
    def __init__(self, name, args):
        self.name = name
        self.args = args

class ListNode(AST):
    def __init__(self, elements):
        self.elements = elements

class BinOp(AST):
    def __init__(self, left, op, right):
        self.left = left
        self.op = op
        self.right = right

class Num(AST):
    def __init__(self, token):
        self.token = token
        self.value = token.value

class Str(AST):
    def __init__(self, token):
        self.token = token
        self.value = token.value

class Variable(AST):
    def __init__(self, token):
        self.token = token
        self.name = token.value

class Parser:
    def __init__(self, tokens):
        self.tokens = tokens
        self.pos = 0
        self.current_token = self.tokens[self.pos] if self.tokens else Token(TokenType.EOF, None, 1)

    def error(self, message="Синтаксическая ошибка"):
        raise SyntaxError(f"{message} в строке {self.current_token.line}")

    def eat(self, token_type):
        if self.current_token.type == token_type:
            self.pos += 1
            if self.pos < len(self.tokens):
                self.current_token = self.tokens[self.pos]
        else:
            self.error(f"Ожидался токен {token_type}, получен {self.current_token.type}")

    def parse(self):
        statements = []
        while self.current_token.type != TokenType.EOF:
            statements.append(self.statement())
        return Program(statements)

    def statement(self):
        if self.current_token.type == TokenType.LET:
            return self.let_statement()
        elif self.current_token.type == TokenType.PRINT:
            return self.print_statement()
        elif self.current_token.type == TokenType.IF:
            return self.if_statement()
        elif self.current_token.type == TokenType.WHILE:
            return self.while_statement()
        elif self.current_token.type == TokenType.FUNC:
            return self.func_statement()
        elif self.current_token.type == TokenType.RETURN:
            return self.return_statement()
        elif self.current_token.type == TokenType.IDENT:
            name = self.current_token.value
            if self.pos + 1 < len(self.tokens) and self.tokens[self.pos + 1].type == TokenType.ASSIGN:
                self.eat(TokenType.IDENT)
                self.eat(TokenType.ASSIGN)
                expr = self.expr()
                return VarAssign(name, expr)
            else:
                return self.expr()
        else:
            self.error(f"Неожиданный токен {self.current_token.type}")

    def let_statement(self):
        self.eat(TokenType.LET)
        if self.current_token.type != TokenType.IDENT:
            self.error("Ожидалось имя переменной после 'let'")
        name = self.current_token.value
        self.eat(TokenType.IDENT)
        self.eat(TokenType.ASSIGN)
        expr = self.expr()
        return VarAssign(name, expr)

    def print_statement(self):
        self.eat(TokenType.PRINT)
        self.eat(TokenType.LPAREN)
        expr = self.expr()
        self.eat(TokenType.RPAREN)
        return PrintNode(expr)

    def if_statement(self):
        self.eat(TokenType.IF)
        condition = self.expr()
        self.eat(TokenType.LBRACE)
        then_block = self.block()
        else_block = None
        if self.current_token.type == TokenType.ELSE:
            self.eat(TokenType.ELSE)
            self.eat(TokenType.LBRACE)
            else_block = self.block()
        return IfNode(condition, then_block, else_block)

    def while_statement(self):
        self.eat(TokenType.WHILE)
        condition = self.expr()
        self.eat(TokenType.LBRACE)
        block = self.block()
        return WhileNode(condition, block)

    def func_statement(self):
        self.eat(TokenType.FUNC)
        if self.current_token.type != TokenType.IDENT:
            self.error("Ожидалось имя функции")
        name = self.current_token.value
        self.eat(TokenType.IDENT)
        self.eat(TokenType.LPAREN)
        params = []
        if self.current_token.type == TokenType.IDENT:
            params.append(self.current_token.value)
            self.eat(TokenType.IDENT)
            while self.current_token.type == TokenType.COMMA:
                self.eat(TokenType.COMMA)
                params.append(self.current_token.value)
                self.eat(TokenType.IDENT)
        self.eat(TokenType.RPAREN)
        self.eat(TokenType.LBRACE)
        body = self.block()
        return FuncDef(name, params, body)

    def return_statement(self):
        self.eat(TokenType.RETURN)
        expr = self.expr()
        return ReturnNode(expr)

    def block(self):
        statements = []
        while self.current_token.type != TokenType.RBRACE and self.current_token.type != TokenType.EOF:
            statements.append(self.statement())
        self.eat(TokenType.RBRACE)
        return statements

    def expr(self):
        return self.equality()

    def equality(self):
        node = self.comparison()
        while self.current_token.type in (TokenType.EQ, TokenType.NEQ):
            token = self.current_token
            self.eat(token.type)
            node = BinOp(node, token.type, self.comparison())
        return node

    def comparison(self):
        node = self.term()
        while self.current_token.type in (TokenType.GT, TokenType.LT):
            token = self.current_token
            self.eat(token.type)
            node = BinOp(node, token.type, self.term())
        return node

    def term(self):
        node = self.factor()
        while self.current_token.type in (TokenType.PLUS, TokenType.MINUS):
            token = self.current_token
            self.eat(token.type)
            node = BinOp(node, token.type, self.factor())
        return node

    def factor(self):
        node = self.unary()
        while self.current_token.type in (TokenType.MUL, TokenType.DIV):
            token = self.current_token
            self.eat(token.type)
            node = BinOp(node, token.type, self.unary())
        return node

    def unary(self):
        token = self.current_token
        if token.type == TokenType.PLUS:
            self.eat(TokenType.PLUS)
            return BinOp(Num(Token(TokenType.NUMBER, 0, token.line)), TokenType.PLUS, self.unary())
        elif token.type == TokenType.MINUS:
            self.eat(TokenType.MINUS)
            return BinOp(Num(Token(TokenType.NUMBER, 0, token.line)), TokenType.MINUS, self.unary())
        return self.primary()

    def primary(self):
        token = self.current_token
        if token.type == TokenType.NUMBER:
            self.eat(TokenType.NUMBER)
            return Num(token)
        elif token.type == TokenType.STRING:
            self.eat(TokenType.STRING)
            return Str(token)
        elif token.type == TokenType.LBRACKET:
            self.eat(TokenType.LBRACKET)
            elements = []
            if self.current_token.type != TokenType.RBRACKET:
                elements.append(self.expr())
                while self.current_token.type == TokenType.COMMA:
                    self.eat(TokenType.COMMA)
                    elements.append(self.expr())
            self.eat(TokenType.RBRACKET)
            return ListNode(elements)
        elif token.type == TokenType.IDENT:
            name = token.value
            self.eat(TokenType.IDENT)
            if self.current_token.type == TokenType.LPAREN:
                self.eat(TokenType.LPAREN)
                args = []
                if self.current_token.type != TokenType.RPAREN:
                    args.append(self.expr())
                    while self.current_token.type == TokenType.COMMA:
                        self.eat(TokenType.COMMA)
                        args.append(self.expr())
                self.eat(TokenType.RPAREN)
                return FunctionCall(name, args)
            return Variable(token)
        elif token.type == TokenType.LPAREN:
            self.eat(TokenType.LPAREN)
            node = self.expr()
            self.eat(TokenType.RPAREN)
            return node
        else:
            self.error(f"Некорректное выражение около токена {token.type}")