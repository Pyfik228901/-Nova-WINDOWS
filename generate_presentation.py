import os

def create_default_nova_site():
    nova_file = "site.nova"
    if not os.path.exists(nova_file):
        with open(nova_file, "w", encoding="utf-8") as f:
            f.write('''title("NovaScript — Интерактивный портал")
header("🎮 Экосистема NovaScript и 2D-игры")
paragraph("Добро пожаловать в презентацию языка. Управляйте движком и изучайте создание 2D-миров.")

joystick_card("🕹️", "Интерактивный 3D-джойстик управления движком")

presentation_start("Интерактивная презентация возможностей")
slide("01", "Чистый синтаксис", "Создавайте скрипты и игры без раздутых веб-технологий.")
slide("02", "Мгновенная компиляция", "Превращайте файлы .nova в готовые консольные или веб-проекты.")
slide("03", "Разработка 2D-игр", "Используйте встроенные шаблоны для создания аркад и игровых полей.")
presentation_end()

button("Скачать шаблон 2D-игры", "downloadTemplate()")
''')

def generate_nova_site():
    create_default_nova_site()
    nova_file = "site.nova"
    
    with open(nova_file, "r", encoding="utf-8") as f:
        lines = f.readlines()

    html_content = """<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <title>NovaScript — Презентация и 2D-игры</title>
    <link rel="icon" href="icon.svg" type="image/svg">
    <style>
        :root {
            --bg: #1e1e2e;
            --card-bg: #181825;
            --text: #cdd6f4;
            --subtext: #a6adc8;
            --accent: #89b4fa;
            --green: #a6e3a1;
            --pink: #f38ba8;
            --purple: #cba6f7;
            --border: #313244;
            --glow: rgba(137, 180, 250, 0.4);
        }
        body {
            background-color: var(--bg);
            color: var(--text);
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            display: flex;
            flex-direction: column;
            align-items: center;
            padding: 50px 20px;
            margin: 0;
            overflow-x: hidden;
            position: relative;
            min-height: 100vh;
            box-sizing: border-box;
        }
        .container {
            max-width: 850px;
            width: 100%;
            background-color: var(--card-bg);
            border: 1px solid var(--border);
            border-radius: 16px;
            padding: 40px;
            text-align: center;
            box-shadow: 0 8px 30px rgba(0,0,0,0.6);
            position: relative;
            z-index: 2;
        }
        h1 { color: var(--accent); font-size: 32px; margin-bottom: 10px; text-shadow: 0 0 10px rgba(137, 180, 250, 0.3); }
        p.desc { color: var(--subtext); font-size: 16px; margin-bottom: 30px; line-height: 1.5; }

        /* 3D-джойстик */
        .joystick-box {
            display: flex;
            justify-content: center;
            margin: 30px 0;
            perspective: 1000px;
        }
        .joystick {
            background: #11111b;
            border: 2px solid var(--border);
            border-radius: 16px;
            width: 150px;
            height: 150px;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            font-size: 56px;
            cursor: pointer;
            transition: transform 0.1s ease-out, box-shadow 0.3s ease, border-color 0.3s ease;
            box-shadow: 0 10px 30px rgba(0,0,0,0.5);
            user-select: none;
        }
        .joystick:hover {
            border-color: var(--accent);
            box-shadow: 0 0 25px var(--glow), 0 15px 35px rgba(0,0,0,0.7);
        }
        .joystick-label {
            font-size: 12px;
            color: var(--subtext);
            margin-top: 8px;
            font-weight: 600;
        }

        /* Слайды */
        .presentation {
            text-align: left;
            margin: 40px 0;
        }
        .pres-title {
            color: var(--green);
            font-size: 20px;
            margin-bottom: 20px;
            text-align: center;
            font-weight: bold;
            text-shadow: 0 0 10px rgba(166, 227, 161, 0.3);
        }
        .slide-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(220px, 1fr));
            gap: 20px;
        }
        .slide-card {
            background: #11111b;
            border: 1px solid var(--border);
            border-radius: 12px;
            padding: 20px;
            position: relative;
            overflow: hidden;
            transition: transform 0.3s, box-shadow 0.3s, border-color 0.3s;
        }
        .slide-card:hover {
            transform: translateY(-5px);
            border-color: var(--accent);
            box-shadow: 0 8px 25px rgba(137, 180, 250, 0.2);
        }
        .slide-number {
            font-size: 12px;
            color: var(--accent);
            font-weight: bold;
            text-transform: uppercase;
            letter-spacing: 1px;
            margin-bottom: 8px;
        }
        .slide-heading {
            font-size: 16px;
            font-weight: bold;
            color: var(--text);
            margin-bottom: 8px;
        }
        .slide-text {
            font-size: 13px;
            color: var(--subtext);
            line-height: 1.4;
        }

        /* Блок 2D Игр и Вселенной */
        .game-section {
            display: flex;
            align-items: center;
            justify-content: space-between;
            background: #11111b;
            border: 1px solid var(--border);
            border-radius: 14px;
            padding: 25px;
            margin-top: 35px;
            gap: 20px;
            text-align: left;
        }
        .game-info h3 {
            color: var(--purple);
            margin-top: 0;
            margin-bottom: 10px;
            font-size: 20px;
        }
        .game-info p {
            color: var(--subtext);
            font-size: 14px;
            margin-bottom: 15px;
            line-height: 1.4;
        }
        .game-buttons {
            display: flex;
            gap: 10px;
            flex-wrap: wrap;
        }

        /* Анимированная крутящаяся вселенная */
        .universe-box {
            width: 120px;
            height: 120px;
            position: relative;
            display: flex;
            align-items: center;
            justify-content: center;
            perspective: 600px;
            flex-shrink: 0;
        }
        .orbit {
            position: absolute;
            width: 100px;
            height: 100px;
            border: 2px dashed rgba(203, 166, 247, 0.3);
            border-radius: 50%;
            animation: rotateUniverse 10s linear infinite;
        }
        .orbit:nth-child(2) {
            width: 70px;
            height: 70px;
            border: 2px dotted rgba(137, 180, 250, 0.4);
            animation: rotateUniverseReverse 7s linear infinite;
        }
        .core-star {
            font-size: 36px;
            animation: pulseStar 3s ease-in-out infinite;
            z-index: 2;
        }
        .planet {
            position: absolute;
            width: 12px;
            height: 12px;
            background-color: var(--pink);
            border-radius: 50%;
            box-shadow: 0 0 10px var(--pink);
            top: 5px;
            left: 50px;
        }
        @keyframes rotateUniverse {
            0% { transform: rotateX(60deg) rotateZ(0deg); }
            100% { transform: rotateX(60deg) rotateZ(360deg); }
        }
        @keyframes rotateUniverseReverse {
            0% { transform: rotateX(40deg) rotateZ(360deg); }
            100% { transform: rotateX(40deg) rotateZ(0deg); }
        }
        @keyframes pulseStar {
            0%, 100% { transform: scale(1); filter: drop-shadow(0 0 5px var(--purple)); }
            50% { transform: scale(1.15); filter: drop-shadow(0 0 15px var(--purple)); }
        }

        /* Кнопки */
        .btn {
            background-color: #252538;
            color: var(--accent);
            border: 1px solid var(--border);
            padding: 10px 18px;
            border-radius: 10px;
            font-weight: bold;
            cursor: pointer;
            font-size: 14px;
            transition: all 0.3s ease;
            text-decoration: none;
            display: inline-block;
        }
        .btn:hover {
            border-color: var(--accent);
            background-color: #2a2a40;
            box-shadow: 0 0 15px rgba(137, 180, 250, 0.3);
            transform: translateY(-2px);
        }
        .btn-primary {
            background-color: var(--accent);
            color: var(--bg);
        }
        .btn-primary:hover {
            background-color: #b4befe;
            box-shadow: 0 0 20px var(--accent);
        }

        .btn-container {
            margin-top: 30px;
        }

        /* Модальное окно для кода */
        .modal {
            display: none;
            position: fixed;
            top: 0; left: 0; width: 100%; height: 100%;
            background: rgba(0,0,0,0.7);
            z-index: 1000;
            justify-content: center;
            align-items: center;
        }
        .modal-content {
            background: #11111b;
            border: 1px solid var(--accent);
            border-radius: 14px;
            width: 90%;
            max-width: 600px;
            padding: 25px;
            text-align: left;
            box-shadow: 0 10px 40px rgba(0,0,0,0.8);
        }
        .modal-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            margin-bottom: 15px;
        }
        .modal-header h3 { color: var(--green); margin: 0; }
        .close-btn { background: none; border: none; color: var(--subtext); font-size: 20px; cursor: pointer; }
        pre {
            background: #181825;
            padding: 15px;
            border-radius: 8px;
            color: var(--text);
            overflow-x: auto;
            font-family: 'Courier New', Courier, monospace;
            font-size: 13px;
            border: 1px solid var(--border);
        }

        /* Пиксельная змейка */
        .snake-segment {
            position: absolute;
            width: 10px;
            height: 10px;
            background-color: var(--green);
            box-shadow: 0 0 8px var(--green);
            z-index: 99;
            pointer-events: none;
            border-radius: 2px;
        }
        .snake-head {
            background-color: var(--pink);
            box-shadow: 0 0 12px var(--pink);
        }
    </style>
</head>
<body>
    <div class="container">
"""

    in_presentation = False

    for line in lines:
        line = line.strip()
        if not line or line.startswith("//"):
            continue

        if line.startswith("title("):
            pass
        elif line.startswith("header("):
            val = line[7:-1].strip('"\'')
            html_content += f"        <h1>{val}</h1>\n"
        elif line.startswith("paragraph("):
            val = line[10:-1].strip('"\'')
            html_content += f'        <p class="desc">{val}</p>\n'
        elif line.startswith("joystick_card("):
            parts = line[14:-1].split(",", 1)
            icon = parts[0].strip().strip('"\'')
            text = parts[1].strip().strip('"\'') if len(parts) > 1 else ""
            html_content += f"""        <div class="joystick-box">
            <div class="joystick" id="joystick">
                <span>{icon}</span>
                <div class="joystick-label">{text}</div>
            </div>
        </div>\n"""
        elif line.startswith("presentation_start("):
            title = line[19:-1].strip('"\'')
            html_content += f'        <div class="presentation">\n            <div class="pres-title">{title}</div>\n            <div class="slide-grid">\n'
            in_presentation = True
        elif line.startswith("slide("):
            parts = line[6:-1].split(",")
            num = parts[0].strip().strip('"\'')
            heading = parts[1].strip().strip('"\'')
            text = parts[2].strip().strip('"\'') if len(parts) > 2 else ""
            if in_presentation:
                html_content += f"""                <div class="slide-card">
                    <div class="slide-number">Слайд {num}</div>
                    <div class="slide-heading">{heading}</div>
                    <div class="slide-text">{text}</div>
                </div>\n"""
        elif line == "presentation_end()":
            html_content += "            </div>\n        </div>\n"
            in_presentation = False
        elif line.startswith("button("):
            parts = line[7:-1].split(",", 1)
            text = parts[0].strip().strip('"\'')
            action = parts[1].strip().strip('"\'') if len(parts) > 1 else ""
            html_content += f'        <div class="btn-container"><button class="btn btn-primary" onclick="{action}" style="font-size: 16px; padding: 15px 30px;">{text}</button></div>\n'

    # Добавляем блок 2D-игр, анимированную вселенную и модальное окно кода
    html_content += """
        <!-- Секция создания 2D-игр и анимированная вселенная -->
        <div class="game-section">
            <div class="game-info">
                <h3>🚀 Гайд по созданию 2D-игр на NovaScript</h3>
                <p>Изучите полное руководство по построению игровых миров, управлению массивами и обработке консольной графики.</p>
                <div class="game-buttons">
                    <a href="#" class="btn btn-primary" onclick="alert('Открывается полная инструкция по 2D-играм на NovaScript!')">📖 Полная инструкция</a>
                    <button class="btn" onclick="showGameCode()">💻 Посмотреть код</button>
                </div>
            </div>
            <div class="universe-box">
                <div class="orbit"></div>
                <div class="orbit">
                    <div class="planet"></div>
                </div>
                <div class="core-star">🌌</div>
            </div>
        </div>

    </div>

    <!-- Модальное окно для исходного кода игры -->
    <div class="modal" id="codeModal">
        <div class="modal-content">
            <div class="modal-header">
                <h3>📜 Исходный код примера игры (.nova)</h3>
                <button class="close-btn" onclick="closeGameCode()">&times;</button>
            </div>
            <pre><code>// 🎮 Пример консольной 2D-игры на NovaScript
print("Запуск игрового движка...")
let map = [0, 0, 1, 0, 2]
print("Карта уровня:")
print(map)

let step = input("Сделайте шаг (1-5): ")
if step == 5 {
    print("🎉 Найдено сокровище!")
} else {
    print("❌ Пустое поле, попробуйте еще раз.")
}</code></pre>
        </div>
    </div>

    <script>
        // 3D-наклон джойстика
        const joystick = document.getElementById('joystick');
        if (joystick) {
            joystick.addEventListener('mousemove', (e) => {
                const rect = joystick.getBoundingClientRect();
                const x = e.clientX - rect.left - rect.width / 2;
                const y = e.clientY - rect.top - rect.height / 2;
                joystick.style.transform = `rotateX(${-y / 5}deg) rotateY(${x / 5}deg) scale(1.05)`;
            });
            joystick.addEventListener('mouseleave', () => {
                joystick.style.transform = 'rotateX(0deg) rotateY(0deg) scale(1)';
            });
        }

        // Скачивание шаблона
        function downloadTemplate() {
            const gameCode = `// 🎮 Шаблон 2D-игры на NovaScript
print("Инициализация игрового мира...")
let grid = [0, 1, 0, 0]
print(grid)
print("Готово к запуску!")`;
            const blob = new Blob([gameCode], { type: 'text/plain;charset=utf-8' });
            const link = document.createElement('a');
            link.href = URL.createObjectURL(blob);
            link.download = 'game_template.nova';
            document.body.appendChild(link);
            link.click();
            document.body.removeChild(link);
        }

        // Модальное окно кода
        function showGameCode() {
            document.getElementById('codeModal').style.display = 'flex';
        }
        function closeGameCode() {
            document.getElementById('codeModal').style.display = 'none';
        }

        // Пиксельная змейка по периметру
        document.addEventListener("DOMContentLoaded", () => {
            const length = 16;
            const segments = [];
            const step = 12;

            for (let i = 0; i < length; i++) {
                const seg = document.createElement("div");
                seg.className = "snake-segment" + (i === 0 ? " snake-head" : "");
                document.body.appendChild(seg);
                segments.push({ el: seg, x: -50, y: -50 });
            }

            let path = [];
            function updatePath() {
                path = [];
                const container = document.querySelector(".container");
                if (!container) return;
                const rect = container.getBoundingClientRect();
                const padding = 12;
                let left = rect.left + window.scrollX - padding;
                let top = rect.top + window.scrollY - padding;
                let right = rect.right + window.scrollX + padding;
                let bottom = rect.bottom + window.scrollY + padding;

                for (let x = left; x <= right; x += step) path.push({x, y: top});
                for (let y = top; y <= bottom; y += step) path.push({x: right, y});
                for (let x = right; x >= left; x -= step) path.push({x, y: bottom});
                for (let y = bottom; y >= top; y -= step) path.push({x: left, y});
            }

            window.addEventListener("resize", updatePath);
            updatePath();

            let currentIndex = 0;
            function animateSnake() {
                if (path.length === 0) return;
                for (let i = segments.length - 1; i > 0; i--) {
                    segments[i].x = segments[i - 1].x;
                    segments[i].y = segments[i - 1].y;
                }
                currentIndex = (currentIndex + 1) % path.length;
                segments[0].x = path[currentIndex].x;
                segments[0].y = path[currentIndex].y;

                for (let i = 0; i < segments.length; i++) {
                    segments[i].el.style.left = segments[i].x + "px";
                    segments[i].el.style.top = segments[i].y + "px";
                }
            }
            setInterval(animateSnake, 45);
        });
    </script>
</body>
</html>"""

    with open("index.html", "w", encoding="utf-8") as f:
        f.write(html_content)
    
    print("✨ Презентация с инструкцией, вращающейся вселенной и просмотром кода успешно обновлена!")

if __name__ == "__main__":
    generate_nova_site()