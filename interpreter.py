import random
import webbrowser
from nova.lexer import TokenType

class Environment:
    def __init__(self, parent=None):
        self.values = {}
        self.parent = parent

    def get(self, name):
        if name in self.values:
            return self.values[name]
        if self.parent is not None:
            return self.parent.get(name)
        raise NameError(f"Неизвестная переменная '{name}'")

    def assign(self, name, value):
        if name in self.values:
            self.values[name] = value
            return
        if self.parent is not None:
            try:
                self.parent.assign(name, value)
                return
            except NameError:
                pass
        self.values[name] = value

class ReturnSignal(Exception):
    def __init__(self, value):
        self.value = value

class Interpreter:
    def __init__(self):
        self.global_env = Environment()
        self.functions = {}

    def visit(self, node, env):
        method_name = f'visit_{type(node).__name__}'
        visitor = getattr(self, method_name, self.generic_visit)
        return visitor(node, env)

    def generic_visit(self, node, env):
        raise NotImplementedError(f'Отсутствует метод visit_{type(node).__name__}')

    def visit_Program(self, node, env):
        result = None
        for statement in node.statements:
            result = self.visit(statement, env)
        return result

    def visit_VarAssign(self, node, env):
        val = self.visit(node.expr, env)
        env.assign(node.name, val)
        return val

    def visit_PrintNode(self, node, env):
        val = self.visit(node.expr, env)
        print(val)
        return val

    def visit_IfNode(self, node, env):
        cond_val = self.visit(node.condition, env)
        if bool(cond_val):
            sub_env = Environment(env)
            for stmt in node.then_block:
                res = self.visit(stmt, sub_env)
                if isinstance(res, ReturnSignal):
                    return res
        elif node.else_block:
            sub_env = Environment(env)
            for stmt in node.else_block:
                res = self.visit(stmt, sub_env)
                if isinstance(res, ReturnSignal):
                    return res

    def visit_WhileNode(self, node, env):
        while bool(self.visit(node.condition, env)):
            sub_env = Environment(env)
            for stmt in node.block:
                res = self.visit(stmt, sub_env)
                if isinstance(res, ReturnSignal):
                    return res

    def visit_FuncDef(self, node, env):
        self.functions[node.name] = node

    def visit_ReturnNode(self, node, env):
        val = self.visit(node.expr, env)
        raise ReturnSignal(val)

    def visit_ListNode(self, node, env):
        return [self.visit(el, env) for el in node.elements]

    def visit_FunctionCall(self, node, env):
        name = node.name
        
        # Встроенные функции
        if name == 'input':
            prompt = self.visit(node.args[0], env) if node.args else ""
            val = input(str(prompt))
            if val.isdigit():
                return int(val)
            try:
                return float(val)
            except ValueError:
                pass
            if val.lower() == 'true':
                return True
            if val.lower() == 'false':
                return False
            return val
        elif name == 'random':
            min_val = int(self.visit(node.args[0], env))
            max_val = int(self.visit(node.args[1], env))
            return random.randint(min_val, max_val)
        elif name == 'browse':
            url = str(self.visit(node.args[0], env))
            print(f"🌐 Открываю в браузере: {url}")
            webbrowser.open(url)
            return None

        if name not in self.functions:
            raise NameError(f"Вызов несуществующей функции '{name}'")

        func = self.functions[name]
        if len(node.args) != len(func.params):
            raise TypeError(f"Функция '{name}' ожидает {len(func.params)} аргументов, передано {len(node.args)}")

        arg_vals = [self.visit(arg, env) for arg in node.args]
        func_env = Environment(self.global_env)
        for param, val in zip(func.params, arg_vals):
            func_env.assign(param, val)

        try:
            for stmt in func.body:
                self.visit(stmt, func_env)
        except ReturnSignal as sig:
            return sig.value
        return None

    def visit_BinOp(self, node, env):
        left = self.visit(node.left, env)
        right = self.visit(node.right, env)
        op = node.op

        if op == TokenType.PLUS:
            return left + right
        elif op == TokenType.MINUS:
            return left - right
        elif op == TokenType.MUL:
            return left * right
        elif op == TokenType.DIV:
            return left / right
        elif op == TokenType.EQ:
            return left == right
        elif op == TokenType.NEQ:
            return left != right
        elif op == TokenType.LT:
            return left < right
        elif op == TokenType.GT:
            return left > right
        else:
            raise ValueError(f"Неизвестный оператор {op}")

    def visit_Num(self, node, env):
        return node.value

    def visit_Str(self, node, env):
        return node.value

    def visit_Variable(self, node, env):
        return env.get(node.name)

    def interpret(self, program_ast):
        return self.visit(program_ast, self.global_env)