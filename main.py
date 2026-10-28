import sys
import os
from nova.lexer import Lexer
from nova.parser import Parser
from nova.interpreter import Interpreter

def main():
    if len(sys.argv) < 2:
        print("💡 Подсказка: python main.py имя_файла.nova")
        return

    filename = sys.argv[1]
    if not os.path.exists(filename):
        print(f"❌ Ошибка: Файл '{filename}' не найден!")
        return

    print(f"🚀 Запуск NovaScript: {filename}")
    print("=" * 45)

    with open(filename, "r", encoding="utf-8") as f:
        code = f.read()

    try:
        lexer = Lexer(code)
        tokens = lexer.tokenize()
        parser = Parser(tokens)
        tree = parser.parse()
        interpreter = Interpreter()
        interpreter.interpret(tree)
    except Exception as e:
        print(f"❌ Ошибка выполнения: {e}")
        sys.exit(1)

    print("=" * 45)
    print("✨ Выполнение скрипта завершено.")

if __name__ == "__main__":
    main()