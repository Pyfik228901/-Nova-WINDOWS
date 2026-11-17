# Стандартная библиотека: 2D-движок (`game2d`)

Модуль `game2d` предоставляет простой, производительный и кроссплатформенный 2D-движок для создания графических приложений и игр на языке **N&S** с выводом изображения в браузере через технологию **HTML5 Canvas**.

Импортируется инструкцией:
```javascript
import "game2d" as game2d
```

---

## 1. Архитектура работы движка

Традиционные нативные графические библиотеки (SDL2, OpenGL, GLFW) требуют сложной настройки окружения, компиляторов C++ и графических драйверов. Движок `game2d` в N&S решает эту проблему архитектурно:

1. **Фоновый HTTP-сервер:** При вызове `game2d.window()` интерпретатор N&S запускает в отдельном потоке легковесный сервер `tiny_http` на локальном адресе `http://127.0.0.1:8090` и автоматически открывает эту страницу в браузере операционной системы по умолчанию.
2. **HTML5 Canvas:** Браузер загружает страницу с элементом `<canvas>` заданного размера, тёмной темой и скриптом рендеринга.
3. **Обмен кадрами (`GET /frame`):** На стороне скрипта N&S команды отрисовки (`clear`, `rect`, `circle`, `text`, `sprite`) формируют список JSON-команд текущего кадра. Браузер запрашивает `/frame` через `requestAnimationFrame` и отрисовывает кадр на аппаратном Canvas 2D Context.
4. **Считывание ввода (`POST /input`):** Обработчики событий браузера (`keydown`, `keyup`, `mousemove`) отправляют актуальный список зажатых клавиш и координаты курсора мыши на маршрут `POST /input`.
5. **Потокобезопасность:** Доступ к общему состоянию (`SharedGameState`) защищён структурой `Arc<Mutex<...>>`, что исключает гонки данных и взаимные блокировки (Deadlock-free).

---

## 2. Инициализация окна и игровой цикл

### 2.1. `game2d.window`
- **Сигнатура:** `game2d.window(width?: Int, height?: Int, title?: String) -> Null`
- **Описание:** Инициализирует окно холста, задаёт его размеры и заголовок страницы (по умолчанию `800x600`, заголовок `"N&S 2D Canvas Engine"`). Запускает локальный веб-сервер и открывает вкладку в браузере.
- **Пример:**
  ```javascript
  game2d.window(800, 600, "Космический шутер");
  ```

---

### 2.2. `game2d.update`
- **Сигнатура:** `game2d.update(callback: Function) -> Null`
- **Описание:** Регистрирует функцию обновления игровой логики. Функция-колбэк принимает один аргумент `dt` (`Float`) — время, прошедшее с предыдущего кадра в секундах (дельта-время). Вызывается перед каждым шагом отрисовки.
- **Пример:**
  ```javascript
  let x = 100;

  game2d.update(func(dt) {
      if (game2d.key_down("ArrowRight")) {
          x += 200.0 * dt;
      }
  });
  ```

---

### 2.3. `game2d.draw`
- **Сигнатура:** `game2d.draw(callback: Function) -> Null`
- **Описание:** Регистрирует функцию отрисовки кадра. Функция не принимает аргументов. Внутри неё вызываются методы `clear`, `rect`, `circle`, `text` и `sprite`.
- **Пример:**
  ```javascript
  game2d.draw(func() {
      game2d.clear("#11111b");
      game2d.circle(x, 300, 25, "#89b4fa");
  });
  ```

---

### 2.4. `game2d.run`
- **Сигнатура:** `game2d.run() -> Null`
- **Описание:** Запускает главный игровой цикл с частотой 60 кадров в секунду (время кадра ~16.6 мс). Цикл последовательно вызывает функцию `update(dt)`, очищает предыдущий буфер команд, вызывает `draw()` и ожидает следующего такта таймера.
- **Пример:**
  ```javascript
  game2d.run();
  ```

---

## 3. Функции отрисовки (Canvas 2D)

### 3.1. `game2d.clear`
- **Сигнатура:** `game2d.clear(color: String) -> Null`
- **Описание:** Очищает весь холст цветом `color`. Цвет задаётся строкой в формате HEX (например, `"#11111b"`, `"#ffffff"`) или стандартным CSS-именем (`"black"`, `"red"`).
- **Пример:**
  ```javascript
  game2d.clear("#181825");
  ```

---

### 3.2. `game2d.rect`
- **Сигнатура:** `game2d.rect(x: Number, y: Number, w: Number, h: Number, color: String) -> Null`
- **Описание:** Добавляет команду отрисовки закрашенного прямоугольника.
  * `x`, `y` — координаты левого верхнего угла.
  * `w`, `h` — ширина и высота прямоугольника.
  * `color` — цвет заливки.
- **Пример:**
  ```javascript
  game2d.rect(50, 50, 120, 80, "#89b4fa");
  ```

---

### 3.3. `game2d.circle`
- **Сигнатура:** `game2d.circle(x: Number, y: Number, r: Number, color: String) -> Null`
- **Описание:** Добавляет команду отрисовки закрашенного круга.
  * `x`, `y` — координаты центра круга.
  * `r` — радиус круга.
  * `color` — цвет заливки.
- **Пример:**
  ```javascript
  game2d.circle(400, 300, 30, "#a6e3a1");
  ```

---

### 3.4. `game2d.text`
- **Сигнатура:** `game2d.text(text: String, x: Number, y: Number, size: Number, color: String) -> Null`
- **Описание:** Отрисовывает текст на холсте.
  * `text` — выводимая строка.
  * `x`, `y` — базовые координаты начала текста.
  * `size` — размер шрифта в пикселях.
  * `color` — цвет текста.
- **Пример:**
  ```javascript
  game2d.text("Счёт: 100", 20, 40, 24, "#cdd6f4");
  ```

---

### 3.5. `game2d.sprite`
- **Сигнатура:** `game2d.sprite(src: String, x: Number, y: Number, w: Number, h: Number) -> Null`
- **Описание:** Отрисовывает растровое изображение (спрайт). В браузере работает автоматическое кэширование загруженных изображений по их URL или локальному пути `src`.
  * `src` — ссылка на изображение (URL или data-URI).
  * `x`, `y` — координаты левого верхнего угла спрайта.
  * `w`, `h` — ширина и высота отрисовки спрайта.
- **Пример:**
  ```javascript
  game2d.sprite("https://example.com/player.png", 200, 200, 64, 64);
  ```

---

## 4. Обработка ввода (Клавиатура и мышь)

### 4.1. `game2d.key_down`
- **Сигнатура:** `game2d.key_down(keyCode: String) -> Bool`
- **Описание:** Проверяет, зажата ли в текущий момент клавиша с кодом `keyCode` (используются стандартные коды DOM KeyboardEvent `e.code`). Возвращает `true`, если клавиша зажата, и `false` в противном случае.
- **Популярные коды клавиш:**
  * Стрелки: `"ArrowUp"`, `"ArrowDown"`, `"ArrowLeft"`, `"ArrowRight"`.
  * Буквы: `"KeyW"`, `"KeyA"`, `"KeyS"`, `"KeyD"`, `"KeyR"`, `"KeyF"`.
  * Служебные: `"Space"`, `"Enter"`, `"Escape"`, `"ShiftLeft"`.
- **Пример:**
  ```javascript
  if (game2d.key_down("Space")) {
      print("Прыжок!");
  }
  ```

---

### 4.2. `game2d.mouse_x` и `game2d.mouse_y`
- **Сигнатура:** `game2d.mouse_x() -> Float`, `game2d.mouse_y() -> Float`
- **Описание:** Возвращают актуальные координаты курсора мыши относительно левого верхнего угла холста `<canvas>`.
- **Пример:**
  ```javascript
  let mx = game2d.mouse_x();
  let my = game2d.mouse_y();
  // Отрисовка прицела на позиции курсора
  game2d.circle(mx, my, 5, "#f38ba8");
  ```

---

## 5. Полный пример: Интерактивная 2D-игра

```javascript
// game.ns
import "game2d" as game2d

let WIDTH = 800;
let HEIGHT = 600;

game2d.window(WIDTH, HEIGHT, "N&S Canvas Game");

// Состояние игрока
let player_x = 400.0;
let player_y = 300.0;
let speed = 250.0;
let score = 0;

// Цель (монетка)
let coin_x = 200.0;
let coin_y = 200.0;

game2d.update(func(dt) {
    // Управление стрелками или WASD
    if (game2d.key_down("ArrowLeft") || game2d.key_down("KeyA")) {
        player_x -= speed * dt;
    }
    if (game2d.key_down("ArrowRight") || game2d.key_down("KeyD")) {
        player_x += speed * dt;
    }
    if (game2d.key_down("ArrowUp") || game2d.key_down("KeyW")) {
        player_y -= speed * dt;
    }
    if (game2d.key_down("ArrowDown") || game2d.key_down("KeyS")) {
        player_y += speed * dt;
    }

    // Проверка сбора монетки
    let dx = player_x - coin_x;
    let dy = player_y - coin_y;
    if (dx * dx + dy * dy < 25.0 * 25.0) {
        score += 1;
        coin_x = (coin_x + 170.0) % 700.0 + 50.0;
        coin_y = (coin_y + 130.0) % 500.0 + 50.0;
    }
});

game2d.draw(func() {
    // 1. Очистка фона
    game2d.clear("#11111b");

    // 2. Отрисовка монетки
    game2d.circle(coin_x, coin_y, 12, "#f9e2af");

    // 3. Отрисовка игрока
    game2d.circle(player_x, player_y, 20, "#89b4fa");

    // 4. Отрисовка интерфейса (UI)
    game2d.text("Счёт: " + score, 20, 40, 20, "#a6e3a1");
    game2d.text("Управление: Стрелки / WASD", 20, 570, 14, "#6c7086");
});

print("[Info] Запуск 2D-движка в браузере...");
game2d.run();
```
