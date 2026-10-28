Markdown<div align="center">

# 🌟 NovaScript

<img src="https://readme-typing-svg.herokuapp.com?font=Fira+Code&weight=600&size=24&duration=3000&pause=1000&color=7C3AED&center=true&vCenter=true&width=600&height=60&lines=Легковесный+интерпретатор+на+Python;Создавай+консольные+утилиты+и+игры;Генерация+презентаций+без+JS/HTML;Простота.+Модульность.+Скорость." alt="Typing Animation" />

<p align="center">
  <b>Кастомный интерпретируемый язык программирования общего назначения на чистом Python.</b>
</p>

<!-- Бейджи -->
<p align="center">
  <img src="https://img.shields.io/badge/python-3.8%2B-blue?style=for-the-badge&logo=python&logoColor=white" alt="Python Version">
  <img src="https://img.shields.io/badge/html5-29%25-orange?style=for-the-badge&logo=html5&logoColor=white" alt="HTML5">
  <img src="https://img.shields.io/badge/license-MIT-green?style=for-the-badge" alt="License">
  <img src="https://img.shields.io/badge/status-active%20development-brightgreen?style=for-the-badge" alt="Status">
  <img src="https://img.shields.io/badge/dependencies-zero-success?style=for-the-badge" alt="Dependencies">
</p>

</div>

---

## 📌 О проекте

**NovaScript** — это модульный интерпретатор программирования, спроектированный «с нуля» без применения внешних библиотек (без ANTLR, PLY или Yacc). Язык нацелен на быстрое создание консольных скриптов, 2D текстовых игр и автоматическую генерацию презентаций/веб-документов без усложнения стека веб-разработки.

### Key Highlights
* 🚀 **Zero Dependencies**: Работает «из коробки» на стандартном Python 3.8+.
* 🧠 **Понятный AST**: Идеальная архитектура для изучения разработки языков программирования.
* 🌐 **Встроенный Web Interop**: Поддержка открытия локальных презентаций и браузерных страниц напрямую через системный `browse()`.

---

## 📊 Состав кодовой базы

<div align="center">

| Язык | Процент | Назначение в проекте |
| :--- | :---: | :--- |
| 🔵 **Python** | **71%** | Ядро интерпретатора (`Lexer`, `Parser`, `AST`, `Interpreter`, `Environment`) |
| 🟠 **HTML** | **29%** | Шаблоны генерации веб-документов, интерактивная документация и UI |

</div>

<br>

> **Распределение кода в репозитории:**
> `█████████████████████████████░░░░░░░░░░` **71% Python • 29% HTML**

---

## 🏗 Архитектура и Структура Проекта

Интерпретатор построен по классической многоуровневой схеме вычислений:

```mermaid
graph TD
    Source[Код NovaScript] -->|Текст программы| Lexer[Lexer nova/lexer.py]
    Lexer -->|Поток токенов| Parser[Parser nova/parser.py]
    Parser -->|Абстрактное синтаксическое дерево| AST[AST Tree]
    AST -->|Обход узлов| Interpreter[Interpreter nova/interpreter.py]
    Interpreter <--> Environment[Environment Таблица символов]
    Interpreter --> Output[Выполнение / Вывод в консоль / Web Browser]
📁 Дерево файловPlaintextnovascript/
├── nova/
│   ├── __init__.py      # Инициализация модуля
│   ├── lexer.py         # Лексический анализатор (токенизатор)
│   ├── parser.py        # Рекурсивный нисходящий парсер (AST)
│   └── interpreter.py   # Выполнение AST, логика Environment и built-in функций
├── templates/           # HTML-шаблоны для генератора документов
│   └── presentation.html
├── main.py              # Точка входа для запуска скриптов
├── README.md            # Документация проекта
└── examples/            # Примеры программ на NovaScript
    └── factorial.nova
💻 Справочник по синтаксису (Syntax Guide)1. Переменные и типы данныхОбъявление происходит с помощью ключевого слова let. Поддерживаются динамические типы:Фрагмент кодаlet name = "NovaScript"  // Строка (String)
let version = 1.0        // Число с плавающей точкой (Float)
let count = 42           // Целое число (Integer)
let items = [1, 2, 3]    // Список (List)
2. Управляющие конструкцииФрагмент кода// Условный оператор if/else
let score = 85
if score > 50 {
    print("Экзамен сдан!")
} else {
    print("Пересдача.")
}

// Цикл while
let step = 0
while step < 3 {
    print(step)
    step = step + 1
}
3. Пользовательские функцииФрагмент кодаfunc greet(user) {
    print("Привет, " + user + "!")
}

func calculate_sum(a, b) {
    return a + b
}

greet("Разработчик")
let total = calculate_sum(10, 20)
print(total)
🛠 Встроенная библиотека (Standard Library)ФункцияОписаниеПримерprint(expr)Печать значения выражения в консольprint("Hello World")input(prompt)Интерактивный запрос ввода от пользователяlet val = input("Введите число: ")random(min, max)Возвращает случайное целое число в диапазонеlet dice = random(1, 6)browse(url)Открывает утилиту, URL или HTML-файл в браузереbrowse("https://python.org")🚀 Быстрый старт и запускТребованияPython 3.8 или выше.Клонирование и запускКлонируйте репозиторий:Bashgit clone [https://github.com/your-username/novascript.git](https://github.com/your-username/novascript.git)
cd novascript
Создайте файл программы example.nova:Фрагмент кодаlet name = input("Как тебя зовут? ")
print("Добро пожаловать в NovaScript, " + name)
Запустите интерпретатор:Bashpython main.py example.nova
