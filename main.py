import sys
import os
import random

def run_nova_script(filename):
    if not os.path.exists(filename):
        print(f"❌ Ошибка: Файл '{filename}' не найден!")
        return

    print(f"🚀 Запуск NovaScript: {filename}")
    print("=" * 45)

    with open(filename, "r", encoding="utf-8") as f:
        lines = f.readlines()

    variables = {}  # Хранилище переменных
    i = 0
    
    while i < len(lines):
        line = lines[i].strip()
        i += 1

        # Пропускаем пустые строки и комментарии
        if not line or line.startswith("//"):
            continue

        # 1. Переменные: let name = value
        if line.startswith("let "):
            parts = line[4:].split("=", 1)
            var_name = parts[0].strip()
            var_val = parts[1].strip()

            if var_val.startswith("input(") and var_val.endswith(")"):
                prompt_text = var_val[6:-1].strip('"').strip("'")
                variables[var_name] = input(prompt_text + " ")
            elif var_val.startswith("random(") and var_val.endswith(")"):
                args = var_val[7:-1].split(",")
                variables[var_name] = random.randint(int(args[0].strip()), int(args[1].strip()))
            elif var_val.startswith('"') and var_val.endswith('"'):
                variables[var_name] = var_val[1:-1]
            elif var_val.isdigit():
                variables[var_name] = int(var_val)
            else:
                variables[var_name] = var_val

        # 2. Вывод: print(...)
        elif line.startswith("print(") and line.endswith(")"):
            content = line[6:-1].strip()
            if content in variables:
                print(variables[content])
            elif (content.startswith('"') and content.endswith('"')) or (content.startswith("'") and content.endswith("'")):
                print(content[1:-1])
            else:
                print(content)

        # 3. Условия: if ... {
        elif line.startswith("if "):
            # Извлекаем условие между 'if' и '{'
            if "{" in line:
                condition_part = line[3:line.index("{")].strip()
            else:
                condition_part = line[3:].strip()

            is_true = False

            # Проверяем знак равенства (==)
            if "==" in condition_part:
                left, right = condition_part.split("==")
                left_val = str(variables.get(left.strip(), left.strip())).strip('"').strip("'")
                right_val = right.strip().strip('"').strip("'")
                if left_val == right_val:
                    is_true = True

            # Проверяем знак больше (>)
            elif ">" in condition_part:
                left, right = condition_part.split(">")
                left_val = float(variables.get(left.strip(), left.strip()))
                right_val = float(right.strip())
                if left_val > right_val:
                    is_true = True

            # Проверяем знак меньше (<)
            elif "<" in condition_part:
                left, right = condition_part.split("<")
                left_val = float(variables.get(left.strip(), left.strip()))
                right_val = float(right.strip())
                if left_val < right_val:
                    is_true = True

            # Если условие ложно, пропускаем блок кода до закрывающей скобки '}'
            if not is_true:
                brace_count = 1
                if "{" in line:
                    brace_count = line.count("{") - line.count("}")
                
                while i < len(lines) and brace_count > 0:
                    next_line = lines[i].strip()
                    brace_count += next_line.count("{")
                    brace_count -= next_line.count("}")
                    i += 1

    print("=" * 45)
    print("✨ Выполнение скрипта завершено.")

if __name__ == "__main__":
    if len(sys.argv) > 1:
        run_nova_script(sys.argv[1])
    else:
        print("💡 Подсказка: python main.py имя_файла.nova")