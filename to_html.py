import os
import webbrowser

def generate_nova_site():
    # Общие стили со змейкой и неоновым свечением
    shared_style = """
        :root { 
            --bg: #1e1e2e; 
            --card-bg: #181825; 
            --text: #cdd6f4; 
            --subtext: #a6adc8; 
            --accent: #89b4fa; 
            --green: #a6e3a1; 
            --pink: #f38ba8; 
            --border: #313244; 
        }
        body { 
            background-color: var(--bg); 
            color: var(--text); 
            font-family: 'Segoe UI', sans-serif; 
            margin: 0; 
            padding: 40px; 
            display: flex; 
            flex-direction: column; 
            align-items: center; 
            min-height: 100vh; 
            justify-content: center;
            overflow-x: hidden;
            position: relative;
        }
        .container { 
            max-width: 600px; 
            width: 100%; 
            background-color: var(--card-bg); 
            border: 1px solid var(--border); 
            border-radius: 16px; 
            padding: 40px; 
            text-align: center; 
            box-shadow: 0 8px 30px rgba(0,0,0,0.4); 
            position: relative;
            z-index: 2;
        }
        h1 { color: var(--accent); margin-bottom: 10px; }
        p { color: var(--subtext); margin-bottom: 30px; }
        
        .menu-btn, .btn {
            background-color: #252538;
            border: 1px solid var(--border);
            color: var(--text);
            padding: 15px 20px;
            border-radius: 10px;
            font-size: 16px;
            font-weight: bold;
            text-decoration: none;
            cursor: pointer;
            transition: all 0.3s ease;
            box-shadow: 0 0 10px rgba(137, 180, 250, 0.1);
            display: block;
            margin-bottom: 15px;
            width: 100%;
            box-sizing: border-box;
        }
        .menu-btn:hover, .btn:hover {
            border-color: var(--accent);
            background-color: #2a2a40;
            color: var(--accent);
            box-shadow: 0 0 20px var(--accent), 0 0 40px rgba(137, 180, 250, 0.4);
            transform: translateY(-2px);
        }

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
    """

    snake_script = """
    <script>
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
    """

    # 1. Главное меню (Hub) с иконкой
    hub_content = f"""<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <title>NovaScript Hub</title>
    <link rel="icon" href="icon.svg" type="image/svg">
    <style>{shared_style}</style>
</head>
<body>
    <div class="container">
        <h1>NovaScript (.nova) Hub</h1>
        <p>Выберите проект для запуска:</p>
        <a href="presentation.html" class="menu-btn">📖 Презентация языка</a>
        <a href="game.html" class="menu-btn">🤖 Крестики-нолики с ИИ</a>
    </div>
    {snake_script}
</body>
</html>"""

    # 2. Страница презентации с иконкой
    presentation_content = f"""<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <title>NovaScript — Презентация</title>
    <link rel="icon" href="icon.svg" type="image/svg">
    <style>{shared_style}</style>
</head>
<body>
    <div class="container" style="text-align: left;">
        <h1 style="text-align: center;">📖 Синтаксис NovaScript</h1>
        <p style="text-align: center;">Базовые возможности языка</p>
        <p><b>1. Переменные:</b> <code>let name = "Nova"</code></p>
        <p><b>2. Условия:</b> <code>if score > 50 {{ print("Победа") }}</code></p>
        <p><b>3. Вывод:</b> <code>print("Привет")</code></p>
        <br>
        <div style="text-align: center;">
            <a href="index.html" class="btn" style="display: inline-block;">&larr; В главное меню</a>
        </div>
    </div>
    {snake_script}
</body>
</html>"""

    # 3. Страница игры с ИИ и иконкой
    game_content = """<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <title>NovaScript — Крестики-нолики с ИИ</title>
    <link rel="icon" href="icon.svg" type="image/svg">
    <style>
        :root { 
            --bg: #1e1e2e; --card-bg: #181825; --text: #cdd6f4; 
            --subtext: #a6adc8; --accent: #89b4fa; --green: #a6e3a1; 
            --pink: #f38ba8; --border: #313244; 
        }
        body { 
            background-color: var(--bg); color: var(--text); 
            font-family: 'Segoe UI', sans-serif; margin: 0; padding: 40px; 
            display: flex; flex-direction: column; align-items: center; 
            min-height: 100vh; justify-content: center; overflow-x: hidden; position: relative;
        }
        .container { 
            max-width: 600px; width: 100%; background-color: var(--card-bg); 
            border: 1px solid var(--border); border-radius: 16px; padding: 40px; 
            text-align: center; box-shadow: 0 8px 30px rgba(0,0,0,0.4); position: relative; z-index: 2;
        }
        h1 { color: var(--accent); margin-bottom: 10px; }
        .btn {
            background-color: #252538; border: 1px solid var(--border); color: var(--text);
            padding: 15px 20px; border-radius: 10px; font-size: 16px; font-weight: bold;
            text-decoration: none; cursor: pointer; transition: all 0.3s ease; display: block; width: 100%; box-sizing: border-box;
        }
        .btn:hover { border-color: var(--accent); background-color: #2a2a40; color: var(--accent); }
        .board { display: grid; grid-template-columns: repeat(3, 90px); grid-template-rows: repeat(3, 90px); gap: 8px; justify-content: center; margin: 20px 0; }
        .cell { background-color: #11111b; border: 2px solid var(--border); border-radius: 10px; font-size: 36px; font-weight: bold; display: flex; align-items: center; justify-content: center; cursor: pointer; transition: 0.2s; }
        .cell:hover { border-color: var(--accent); background-color: #252538; box-shadow: 0 0 10px var(--accent); }
        .cell.x { color: var(--pink); }
        .cell.o { color: var(--green); }
        .snake-segment { position: absolute; width: 10px; height: 10px; background-color: var(--green); box-shadow: 0 0 8px var(--green); z-index: 99; pointer-events: none; border-radius: 2px; }
        .snake-head { background-color: var(--pink); box-shadow: 0 0 12px var(--pink); }
    </style>
</head>
<body>
    <div class="container">
        <h1>🤖 Крестики-нолики с ИИ</h1>
        <div id="status" style="font-weight: bold; color: var(--subtext); margin: 15px 0;">Ваш ход (X)</div>

        <div class="board">
            <div class="cell" onclick="playerMove(0)"></div>
            <div class="cell" onclick="playerMove(1)"></div>
            <div class="cell" onclick="playerMove(2)"></div>
            <div class="cell" onclick="playerMove(3)"></div>
            <div class="cell" onclick="playerMove(4)"></div>
            <div class="cell" onclick="playerMove(5)"></div>
            <div class="cell" onclick="playerMove(6)"></div>
            <div class="cell" onclick="playerMove(7)"></div>
            <div class="cell" onclick="playerMove(8)"></div>
        </div>

        <button class="btn" onclick="resetGame()">Заново</button>
        <br>
        <a href="index.html" style="color: var(--subtext); text-decoration: none; font-size: 14px; display: inline-block; margin-top: 10px;">&larr; На главную</a>
    </div>

    <script>
        let board = ["", "", "", "", "", "", "", "", ""];
        let isGameActive = true;
        const wins = [[0,1,2],[3,4,5],[6,7,8],[0,3,6],[1,4,7],[2,5,8],[0,4,8],[2,4,6]];

        function playerMove(i) {
            if (board[i] !== "" || !isGameActive) return;
            board[i] = "X";
            update();
            
            if (checkWin("X")) { 
                document.getElementById("status").textContent = "🎉 Вы победили!"; 
                isGameActive = false; 
                return; 
            }
            if (!board.includes("")) { 
                document.getElementById("status").textContent = "🤝 Ничья!"; 
                isGameActive = false; 
                return; 
            }
            
            document.getElementById("status").textContent = "🤖 ИИ думает...";
            setTimeout(aiMove, 350);
        }

        function aiMove() {
            if (!isGameActive) return;
            
            let move = findBestMove("O");
            if (move === -1) move = findBestMove("X");
            if (move === -1 && board[4] === "") move = 4;
            if (move === -1) {
                let empty = [];
                board.forEach((v, idx) => { if (v === "") empty.push(idx); });
                if (empty.length > 0) move = empty[Math.floor(Math.random() * empty.length)];
            }

            if (move !== -1) {
                board[move] = "O";
                update();
                if (checkWin("O")) { 
                    document.getElementById("status").textContent = "🤖 ИИ победил!"; 
                    isGameActive = false; 
                    return; 
                }
                if (!board.includes("")) { 
                    document.getElementById("status").textContent = "🤝 Ничья!"; 
                    isGameActive = false; 
                    return; 
                }
                document.getElementById("status").textContent = "Ваш ход (X)";
            }
        }

        function findBestMove(player) {
            for (let w of wins) {
                let [a, b, c] = w;
                let line = [board[a], board[b], board[c]];
                if (line.filter(v => v === player).length === 2 && line.includes("")) {
                    if (board[a] === "") return a;
                    if (board[b] === "") return b;
                    if (board[c] === "") return c;
                }
            }
            return -1;
        }

        function checkWin(p) { 
            return wins.some(w => board[w[0]] === p && board[w[1]] === p && board[w[2]] === p); 
        }

        function update() {
            document.querySelectorAll(".cell").forEach((c, i) => {
                c.textContent = board[i];
                c.className = "cell " + (board[i] === "X" ? "x" : board[i] === "O" ? "o" : "");
            });
        }

    function resetGame() { 
            board = ["","","","","","","","",""]; 
            isGameActive = true; 
            document.getElementById("status").textContent = "Ваш ход (X)"; 
            update(); 
        }
    </script>
</body>
</html>"""

    # Сохраняем все файлы
    with open("index.html", "w", encoding="utf-8") as f:
        f.write(hub_content)
    with open("presentation.html", "w", encoding="utf-8") as f:
        f.write(presentation_content)
    with open("game.html", "w", encoding="utf-8") as f:
        f.write(game_content)

    print("🚀 Сайт успешно сгенерирован с новой неоновой иконкой!")
    
    # Автоматически открываем index.html в браузере
    file_path = os.path.abspath("index.html")
    webbrowser.open(f"file://{file_path}")

if __name__ == "__main__":
    generate_nova_site()