import re
import random
import webbrowser

class NovaInterpreter:
    def __init__(self, parent=None):
        self.variables = {}
        if parent:
            self.functions = parent.functions
        else:
            self.functions = {}

    def run(self, code_text):
        lines = code_text.strip().split('\n')
        i = 0
        while i < len(lines):
            line = lines[i].strip()
            
            # Пропускаем пустые строки и комментарии
            if not line or line.startswith('//'):
                i += 1
                continue

            # 1. Возврат значения из функции
            return_match = re.match(r'^return\s+(.+)$', line)
            if return_match:
                expr = return_match.group(1).strip()
                return self.evaluate(expr)

            # 2. Присвоение переменной (включая списки: let items = [1, 2, 3])
            let_match = re.match(r'^let\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+)$', line)
            if let_match:
                var_name = let_match.group(1)
                var_expr = let_match.group(2).strip()
                self.variables[var_name] = self.evaluate(var_expr)
                i += 1
                continue

            # 3. Вывод текста (print(...))
            print_match = re.match(r'^print\s*\((.+)\)$', line)
            if print_match:
                expr = print_match.group(1).strip()
                value = self.evaluate(expr)
                print(value)
                i += 1
                continue

            # 4. Открытие сайта в браузере (browse("url"))
            browse_match = re.match(r'^browse\s*\((.+)\)$', line)
            if browse_match:
                expr = browse_match.group(1).strip()
                url = self.evaluate(expr)
                print(f"Открываю в браузере: {url}")
                webbrowser.open(url)
                i += 1
                continue

            # 5. Условия (if condition { ... })
            if_match = re.match(r'^if\s+(.+)\s*\{\s*$', line)
            if if_match:
                condition = if_match.group(1).strip()
                condition_result = self.evaluate(condition)
                
                i += 1
                block_lines = []
                while i < len(lines) and lines[i].strip() != "}":
                    block_lines.append(lines[i])
                    i += 1
                
                if condition_result:
                    sub_interp = NovaInterpreter(parent=self)
                    sub_interp.variables = self.variables.copy()
                    res = sub_interp.run("\n".join(block_lines))
                    self.variables.update(sub_interp.variables)
                    if res is not None:
                        return res
                
                i += 1
                continue

            # 6. Циклы (while condition { ... })
            while_match = re.match(r'^while\s+(.+)\s*\{\s*$', line)
            if while_match:
                condition = while_match.group(1).strip()
                
                i += 1
                block_lines = []
                while i < len(lines) and lines[i].strip() != "}":
                    block_lines.append(lines[i])
                    i += 1
                
                while self.evaluate(condition):
                    sub_interp = NovaInterpreter(parent=self)
                    sub_interp.variables = self.variables.copy()
                    res = sub_interp.run("\n".join(block_lines))
                    self.variables.update(sub_interp.variables)
                    if res is not None:
                        return res
                
                i += 1
                continue

            # 7. Определение функции (func name(args) { ... })
            func_match = re.match(r'^func\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*?)\)\s*\{\s*$', line)
            if func_match:
                func_name = func_match.group(1)
                params_str = func_match.group(2).strip()
                params = [p.strip() for p in params_str.split(',')] if params_str else []
                
                i += 1
                block_lines = []
                while i < len(lines) and lines[i].strip() != "}":
                    block_lines.append(lines[i])
                    i += 1
                
                self.functions[func_name] = {'params': params, 'body': block_lines}
                i += 1
                continue

            # 8. Вызов функции
            func_call_match = re.match(r'^([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*)\)$', line)
            if func_call_match:
                func_name = func_call_match.group(1)
                if func_name in self.functions:
                    args_str = func_call_match.group(2).strip()
                    self.call_function(func_name, args_str)
                    i += 1
                    continue

            raise SyntaxError(f"Неизвестная команда: {line}")
        
        return None

    def call_function(self, func_name, args_str):
        func_info = self.functions[func_name]
        params = func_info['params']
        
        args = []
        if args_str:
            raw_args = args_str.split(',')
            for arg in raw_args:
                args.append(self.evaluate(arg.strip()))

        sub_interp = NovaInterpreter(parent=self)
        for p, val in zip(params, args):
            sub_interp.variables[p] = val

        return sub_interp.run("\n".join(func_info['body']))

    def evaluate(self, expr):
        expr = expr.strip()

        # Случайные числа random(min, max)
        rand_match = re.match(r'^random\s*\((.+)\s*,\s*(.+)\)$', expr)
        if rand_match:
            min_val = int(self.evaluate(rand_match.group(1)))
            max_val = int(self.evaluate(rand_match.group(2)))
            return random.randint(min_val, max_val)

        # Вызов пользовательской функции
        func_call_match = re.match(r'^([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*)\)$', expr)
        if func_call_match:
            func_name = func_call_match.group(1)
            if func_name in self.functions:
                args_str = func_call_match.group(2).strip()
                return self.call_function(func_name, args_str)

        # Ввод от пользователя input(...)
        input_match = re.match(r'^input\s*\((.+)\)$', expr)
        if input_match:
            prompt_text = self.evaluate(input_match.group(1))
            user_val = input(prompt_text)
            if user_val.isdigit():
                return int(user_val)
            if user_val.lower() == "true":
                return True
            if user_val.lower() == "false":
                return False
            return user_val

        # Строки в кавычках
        if (expr.startswith('"') and expr.endswith('"')) or (expr.startswith("'") and expr.endswith("'")):
            return expr[1:-1]

        # Подстановка переменных и списков
        eval_expr = expr
        for var_name, var_val in self.variables.items():
            if isinstance(var_val, str):
                replacement = f"'{var_val}'"
            else:
                replacement = str(var_val)
            eval_expr = re.sub(r'\b' + var_name + r'\b', replacement, eval_expr)

        eval_expr = re.sub(r'\btrue\b', 'True', eval_expr, flags=tr.IGNORECASE if 'tr' in globals() else re.IGNORECASE)
        eval_expr = re.sub(r'\bfalse\b', 'False', eval_expr, flags=re.IGNORECASE)

        try:
            return eval(eval_expr)
        except Exception:
            raise ValueError(f"Ошибка вычисления выражения: {expr}")
        import re

def execute_for_loop(line, block_lines, variables):
    # Пример строки: for i = 1 to 5
    match = re.match(r"for\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+)\s+to\s+(.+)", line)
    if match:
        var_name = match.group(1)
        start_val = int(eval(match.group(2), {}, variables))
        end_val = int(eval(match.group(3), {}, variables))
        
        # Цикл от start до end включительно
        for val in range(start_val, end_val + 1):
            variables[var_name] = val
            # Выполняем строки внутри блока { ... }
            for inner_line in block_lines:
                execute_line(inner_line, variables) # Здесь вызывается ваша функция выполнения строки
                import re

# Пример структуры для хранения переменных в interpreter.py
variables = {}

def evaluate_expression(expr, vars_dict):
    # Безопасная подстановка переменных и поддержка словарей
    e = expr.strip()
    for k, v in vars_dict.items():
        # Заменяем имя переменной на её значение
        reg = r'\b' + k + r'\b'
        e = re.sub(reg, repr(v), e)
    try:
        return eval(e)
    except Exception:
        return expr

def execute_line(line, vars_dict):
    line = line.strip()
    if not line or line.startswith("//"):
        return

    # 1. Создание словаря: let user = {"name": "Nova", "hp": 100}
    let_dict_match = re.match(r"^let\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(\{.*\})$", line)
    if let_dict_match:
        var_name = let_dict_match.group(1)
        dict_content = let_dict_match.group(2)
        # Превращаем строку словаря Python/JSON в реальный словарь
        vars_dict[var_name] = eval(dict_content)
        return

    # 2. Обычный let (переменные, числа, строки)
    let_match = re.match(r"^let\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+)$", line)
    if let_match:
        var_name = let_match.group(1)
        expr = let_match.group(2)
        vars_dict[var_name] = evaluate_expression(expr, vars_dict)
        return

    # 3. Команда print
    print_match = re.match(r"^print\s*\((.+)\)$", line)
    if print_match:
        res = evaluate_expression(print_match.group(1), vars_dict)
        print(res)
        return