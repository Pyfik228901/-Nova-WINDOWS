const vscode = require('vscode');

const KEYWORDS = [
    { name: 'let', detail: 'N&S: Объявление переменной', doc: 'Объявление локальной или глобальной переменной.\nПример: let x = 42;', snippet: 'let ${1:name} = ${2:value};' },
    { name: 'func', detail: 'N&S: Определение функции', doc: 'Объявление пользовательской функции.\nПример: func add(a, b) { return a + b; }', snippet: 'func ${1:name}(${2:params}) {\n\t${3:// body}\n}' },
    { name: 'return', detail: 'N&S: Возврат из функции', doc: 'Возврат значения из тела функции.', snippet: 'return ${1:value};' },
    { name: 'if', detail: 'N&S: Условный оператор', doc: 'Ветвление логики по условию.', snippet: 'if (${1:condition}) {\n\t${2:// body}\n}' },
    { name: 'else', detail: 'N&S: Ветка else', doc: 'Альтернативная ветка условия if.', snippet: 'else {\n\t${1:// body}\n}' },
    { name: 'while', detail: 'N&S: Цикл while', doc: 'Выполнение цикла пока условие истинно.', snippet: 'while (${1:condition}) {\n\t${2:// body}\n}' },
    { name: 'for', detail: 'N&S: Цикл for..in', doc: 'Итерация по элементам массива.\nПример: for item in list { print(item); }', snippet: 'for ${1:item} in ${2:array} {\n\t${3:// body}\n}' },
    { name: 'import', detail: 'N&S: Импорт модуля', doc: 'Импорт стандартной библиотеки или внешнего файла.\nПример: import "web" as web', snippet: 'import "${1:module}" as ${2:alias};' },
    { name: 'as', detail: 'N&S: Псевдоним импорта', doc: 'Назначение алиаса для импортируемого модуля.', snippet: 'as ${1:alias}' },
    { name: 'extern', detail: 'N&S: Внешний блок GrassGreen', doc: 'Встраивание нативного кода GrassGreen (extern "grass").', snippet: 'extern "grass" {\n\t${1:gout << "Hello from GrassGreen" << endl;}\n}' },
    { name: 'true', detail: 'Булево значение true', doc: 'Логическая истина.', snippet: 'true' },
    { name: 'false', detail: 'Булево значение false', doc: 'Логическая ложь.', snippet: 'false' },
    { name: 'null', detail: 'Значение null', doc: 'Отсутствие значения.', snippet: 'null' }
];

const BUILTINS = [
    { name: 'print', signature: 'print(...)', doc: 'Вывод одного или нескольких значений в стандартный вывод с переносом строки.', snippet: 'print(${1:value});' },
    { name: 'input', signature: 'input(prompt = "")', doc: 'Ввод строки из терминала с автоматическим приведением типов.', snippet: 'input("${1:prompt}: ")' },
    { name: 'random', signature: 'random(min, max)', doc: 'Генерация псевдослучайного числа в заданном диапазоне.', snippet: 'random(${1:1}, ${2:100})' },
    { name: 'browse', signature: 'browse(url)', doc: 'Открытие URL-адреса или файла в веб-браузере по умолчанию.', snippet: 'browse("${1:http://localhost:8080}")' },
    { name: 'len', signature: 'len(collection)', doc: 'Возвращает количество элементов в массиве, карте (Map) или длину строки.', snippet: 'len(${1:target})' },
    { name: 'exit', signature: 'exit(code = 0)', doc: 'Немедленное завершение процесса с указанным кодом возврата.', snippet: 'exit(${1:0});' }
];

const MODULE_MEMBERS = {
    web: [
        { name: 'get', signature: 'web.get(path, handler)', doc: 'Регистрация обработчика GET-запроса.\nПример: web.get("/", func(req) { return "Hello"; });', snippet: 'get("${1:/}", func(req) {\n\treturn ${2:web.html("Title", "Body")};\n});' },
        { name: 'post', signature: 'web.post(path, handler)', doc: 'Регистрация обработчика POST-запроса.', snippet: 'post("${1:/api}", func(req) {\n\treturn {"status": "ok"};\n});' },
        { name: 'server', signature: 'web.server(port)', doc: 'Запуск встроенного HTTP-сервера на указанном порту.', snippet: 'server(${1:8080});' },
        { name: 'listen', signature: 'web.listen(port)', doc: 'Алиас метода web.server.', snippet: 'listen(${1:8080});' },
        { name: 'json_stringify', signature: 'web.json_stringify(value)', doc: 'Сериализация структуры данных N&S в форматированную строку JSON.', snippet: 'json_stringify(${1:data})' },
        { name: 'json_parse', signature: 'web.json_parse(json_str)', doc: 'Десериализация строки JSON в структуры данных N&S (Map/Array).', snippet: 'json_parse(${1:json_str})' },
        { name: 'html', signature: 'web.html(title, body)', doc: 'Генерация адаптивной HTML-страницы с современной темной темой.', snippet: 'html("${1:Title}", ${2:content})' },
        { name: 'tag', signature: 'web.tag(name, attrs, content)', doc: 'Генерация HTML-тега с атрибутами и содержимым.', snippet: 'tag("${1:div}", { "${2:class}": "${3:box}" }, ${4:text})' }
    ],
    game2d: [
        { name: 'window', signature: 'game2d.window(width, height, title)', doc: 'Инициализация окна HTML5 Canvas в браузере через локальный веб-сервер.', snippet: 'window(${1:800}, ${2:600}, "${3:N&S Game}");' },
        { name: 'clear', signature: 'game2d.clear(color)', doc: 'Очистка холста указанным цветом (HEX или CSS-имя).', snippet: 'clear("${1:#11111b}");' },
        { name: 'rect', signature: 'game2d.rect(x, y, w, h, color)', doc: 'Отрисовка закрашенного прямоугольника.', snippet: 'rect(${1:x}, ${2:y}, ${3:w}, ${4:h}, "${5:#89b4fa}");' },
        { name: 'circle', signature: 'game2d.circle(x, y, radius, color)', doc: 'Отрисовка закрашенного круга.', snippet: 'circle(${1:x}, ${2:y}, ${3:radius}, "${4:#a6e3a1}");' },
        { name: 'text', signature: 'game2d.text(string, x, y, size, color)', doc: 'Отрисовка текста на холсте.', snippet: 'text("${1:Hello}", ${2:x}, ${3:y}, ${4:20}, "${5:#cdd6f4}");' },
        { name: 'sprite', signature: 'game2d.sprite(src, x, y, w, h)', doc: 'Отрисовка растрового спрайта/изображения по URL или пути.', snippet: 'sprite("${1:player.png}", ${2:x}, ${3:y}, ${4:w}, ${5:h});' },
        { name: 'update', signature: 'game2d.update(callback)', doc: 'Установка функции обновления игровой логики (вызывается каждый тик).', snippet: 'update(func(dt) {\n\t${1:// update logic}\n});' },
        { name: 'draw', signature: 'game2d.draw(callback)', doc: 'Установка функции отрисовки кадра.', snippet: 'draw(func() {\n\t${1:// draw frame}\n});' },
        { name: 'run', signature: 'game2d.run()', doc: 'Запуск игрового цикла 60 FPS.', snippet: 'run();' },
        { name: 'key_down', signature: 'game2d.key_down(keyCode)', doc: 'Проверка удержания клавиши (например "KeyW", "KeyA", "Space", "ArrowUp").', snippet: 'key_down("${1:KeyW}")' },
        { name: 'mouse_x', signature: 'game2d.mouse_x()', doc: 'Текущая X-координата курсора мыши на холсте.', snippet: 'mouse_x()' },
        { name: 'mouse_y', signature: 'game2d.mouse_y()', doc: 'Текущая Y-координата курсора мыши на холсте.', snippet: 'mouse_y()' }
    ],
    math: [
        { name: 'sqrt', signature: 'math.sqrt(x)', doc: 'Вычисление квадратного корня числа.', snippet: 'sqrt(${1:x})' },
        { name: 'pow', signature: 'math.pow(base, exp)', doc: 'Возведение числа в степень.', snippet: 'pow(${1:base}, ${2:exp})' },
        { name: 'abs', signature: 'math.abs(x)', doc: 'Абсолютная величина числа.', snippet: 'abs(${1:x})' },
        { name: 'floor', signature: 'math.floor(x)', doc: 'Округление вниз до ближайшего целого.', snippet: 'floor(${1:x})' },
        { name: 'ceil', signature: 'math.ceil(x)', doc: 'Округление вверх до ближайшего целого.', snippet: 'ceil(${1:x})' },
        { name: 'round', signature: 'math.round(x)', doc: 'Округление до ближайшего целого.', snippet: 'round(${1:x})' },
        { name: 'sin', signature: 'math.sin(rad)', doc: 'Тригонометрический синус угла в радианах.', snippet: 'sin(${1:rad})' },
        { name: 'cos', signature: 'math.cos(rad)', doc: 'Тригонометрический косинус угла в радианах.', snippet: 'cos(${1:rad})' },
        { name: 'tan', signature: 'math.tan(rad)', doc: 'Тригонометрический тангенс угла в радианах.', snippet: 'tan(${1:rad})' },
        { name: 'random', signature: 'math.random()', doc: 'Случайное число от 0.0 до 1.0.', snippet: 'random()' },
        { name: 'pi', signature: 'math.pi', doc: 'Число Пи (3.141592653589793).', snippet: 'pi' },
        { name: 'e', signature: 'math.e', doc: 'Число Эйлера e (2.718281828459045).', snippet: 'e' }
    ],
    io: [
        { name: 'read_file', signature: 'io.read_file(path)', doc: 'Чтение всего содержимого файла в виде строки.', snippet: 'read_file("${1:file.txt}")' },
        { name: 'write_file', signature: 'io.write_file(path, content)', doc: 'Запись строки в файл.', snippet: 'write_file("${1:file.txt}", ${2:content});' },
        { name: 'delete_file', signature: 'io.delete_file(path)', doc: 'Удаление файла с диска.', snippet: 'delete_file("${1:file.txt}");' },
        { name: 'remove_file', signature: 'io.remove_file(path)', doc: 'Алиас метода io.delete_file.', snippet: 'remove_file("${1:file.txt}");' },
        { name: 'exists', signature: 'io.exists(path)', doc: 'Проверка существования файла или директории.', snippet: 'exists("${1:file.txt}")' },
        { name: 'browse', signature: 'io.browse(url)', doc: 'Открытие адреса или локального файла в браузере.', snippet: 'browse("${1:url}")' },
        { name: 'print', signature: 'io.print(...)', doc: 'Вывод данных в консоль.', snippet: 'print(${1:value});' },
        { name: 'input', signature: 'io.input(prompt)', doc: 'Ввод строки из консоли.', snippet: 'input("${1:prompt}: ")' }
    ],
    cli: [
        { name: 'args', signature: 'cli.args()', doc: 'Возвращает массив аргументов командной строки, переданных скрипту.', snippet: 'args()' },
        { name: 'green', signature: 'cli.green(text)', doc: 'Окрашивает текст в зеленый цвет ANSI.', snippet: 'green(${1:text})' },
        { name: 'red', signature: 'cli.red(text)', doc: 'Окрашивает текст в красный цвет ANSI.', snippet: 'red(${1:text})' },
        { name: 'blue', signature: 'cli.blue(text)', doc: 'Окрашивает текст в синий цвет ANSI.', snippet: 'blue(${1:text})' },
        { name: 'yellow', signature: 'cli.yellow(text)', doc: 'Окрашивает текст в желтый цвет ANSI.', snippet: 'yellow(${1:text})' },
        { name: 'cyan', signature: 'cli.cyan(text)', doc: 'Окрашивает текст в голубой (cyan) цвет ANSI.', snippet: 'cyan(${1:text})' },
        { name: 'magenta', signature: 'cli.magenta(text)', doc: 'Окрашивает текст в пурпурный цвет ANSI.', snippet: 'magenta(${1:text})' },
        { name: 'bold', signature: 'cli.bold(text)', doc: 'Делает текст жирным в терминале.', snippet: 'bold(${1:text})' },
        { name: 'clear', signature: 'cli.clear()', doc: 'Очистка экрана терминала.', snippet: 'clear();' }
    ]
};

function activate(context) {
    console.log('N&S (Nova Script) Language Support успешно активирован.');

    // 1. Completion Provider
    const completionProvider = vscode.languages.registerCompletionItemProvider('ns', {
        provideCompletionItems(document, position) {
            const linePrefix = document.lineAt(position).text.substring(0, position.character);
            const items = [];

            // Проверка ввода через точку (member access): web., game2d., math., io., cli.
            const memberMatch = linePrefix.match(/([a-zA-Z0-9_]+)\.$/);
            if (memberMatch) {
                const modName = memberMatch[1];
                if (MODULE_MEMBERS[modName]) {
                    for (const m of MODULE_MEMBERS[modName]) {
                        const item = new vscode.CompletionItem(m.name, vscode.CompletionItemKind.Method);
                        item.detail = m.signature;
                        item.documentation = new vscode.MarkdownString(m.doc);
                        item.insertText = new vscode.SnippetString(m.snippet);
                        items.push(item);
                    }
                    return items;
                }
            }

            // Ключевые слова
            for (const kw of KEYWORDS) {
                const item = new vscode.CompletionItem(kw.name, vscode.CompletionItemKind.Keyword);
                item.detail = kw.detail;
                item.documentation = new vscode.MarkdownString(kw.doc);
                item.insertText = new vscode.SnippetString(kw.snippet);
                items.push(item);
            }

            // Встроенные функции
            for (const fn of BUILTINS) {
                const item = new vscode.CompletionItem(fn.name, vscode.CompletionItemKind.Function);
                item.detail = fn.signature;
                item.documentation = new vscode.MarkdownString(fn.doc);
                item.insertText = new vscode.SnippetString(fn.snippet);
                items.push(item);
            }

            // Модули стандартной библиотеки
            for (const modName of Object.keys(MODULE_MEMBERS)) {
                const item = new vscode.CompletionItem(modName, vscode.CompletionItemKind.Module);
                item.detail = `N&S Модуль ${modName}`;
                item.documentation = new vscode.MarkdownString(`Модуль стандартной библиотеки \`${modName}\`.\nИспользуйте \`import "${modName}" as ${modName}\`.`);
                item.insertText = modName;
                items.push(item);
            }

            return items;
        }
    }, '.', ' ', '(');

    // 2. Hover Provider
    const hoverProvider = vscode.languages.registerHoverProvider('ns', {
        provideHover(document, position) {
            const range = document.getWordRangeAtPosition(position, /[a-zA-Z0-9_]+/);
            if (!range) return null;

            const word = document.getText(range);

            const fn = BUILTINS.find(f => f.name === word);
            if (fn) {
                const md = new vscode.MarkdownString();
                md.appendCodeblock(fn.signature, 'ns');
                md.appendMarkdown(`\n${fn.doc}`);
                return new vscode.Hover(md);
            }

            if (MODULE_MEMBERS[word]) {
                const md = new vscode.MarkdownString();
                md.appendCodeblock(`import "${word}" as ${word};`, 'ns');
                md.appendMarkdown(`\n**Модуль стандартной библиотеки N&S \`${word}\`**`);
                return new vscode.Hover(md);
            }

            for (const modName in MODULE_MEMBERS) {
                const member = MODULE_MEMBERS[modName].find(m => m.name === word);
                if (member) {
                    const md = new vscode.MarkdownString();
                    md.appendCodeblock(member.signature, 'ns');
                    md.appendMarkdown(`\n${member.doc}`);
                    return new vscode.Hover(md);
                }
            }

            const kw = KEYWORDS.find(k => k.name === word);
            if (kw) {
                const md = new vscode.MarkdownString();
                md.appendCodeblock(kw.name, 'ns');
                md.appendMarkdown(`\n${kw.doc}`);
                return new vscode.Hover(md);
            }

            return null;
        }
    });

    // 3. Run Command (F5)
    const runCommand = vscode.commands.registerCommand('ns.runFile', async () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor || (editor.document.languageId !== 'ns')) {
            vscode.window.showErrorMessage('Откройте файл N&S (.ns / .nova) для запуска.');
            return;
        }
        await editor.document.save();
        const filePath = editor.document.fileName;
        let terminal = vscode.window.terminals.find(t => t.name === 'N&S Terminal');
        if (!terminal) {
            terminal = vscode.window.createTerminal('N&S Terminal');
        }
        terminal.show();
        terminal.sendText(`ns "${filePath}"`);
        vscode.window.setStatusBarMessage('$(play) N&S: Запуск...', 3000);
    });

    // 4. Status Bar Button
    const runStatusBarItem = vscode.window.createStatusBarItem(vscode.StatusBarAlignment.Left, 100);
    runStatusBarItem.command = 'ns.runFile';
    runStatusBarItem.text = '$(play) N&S: Запустить (F5)';
    runStatusBarItem.tooltip = 'Запустить текущий скрипт в терминале N&S';

    function updateStatusBar(editor) {
        if (editor && editor.document.languageId === 'ns') {
            runStatusBarItem.show();
        } else {
            runStatusBarItem.hide();
        }
    }

    vscode.window.onDidChangeActiveTextEditor(updateStatusBar, null, context.subscriptions);
    updateStatusBar(vscode.window.activeTextEditor);

    context.subscriptions.push(completionProvider, hoverProvider, runCommand, runStatusBarItem);
}

function deactivate() {}

module.exports = {
    activate,
    deactivate
};
