use crate::core::value::{NativeFn, Value};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashSet;
use std::process::Command;
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum DrawCommand {
    #[serde(rename = "clear")]
    Clear { color: String },
    #[serde(rename = "rect")]
    Rect { x: f64, y: f64, w: f64, h: f64, color: String },
    #[serde(rename = "circle")]
    Circle { x: f64, y: f64, r: f64, color: String },
    #[serde(rename = "text")]
    Text { text: String, x: f64, y: f64, size: f64, color: String },
    #[serde(rename = "sprite")]
    Sprite { src: String, x: f64, y: f64, w: f64, h: f64 },
}

pub struct SharedGameState {
    pub width: u32,
    pub height: u32,
    pub title: String,
    pub keys_down: HashSet<String>,
    pub mouse_x: f64,
    pub mouse_y: f64,
    pub draw_commands: Vec<DrawCommand>,
    pub server_started: bool,
    pub port: u16,
}

impl SharedGameState {
    pub fn new() -> Self {
        Self {
            width: 800,
            height: 600,
            title: "N&S 2D Canvas Engine".to_string(),
            keys_down: HashSet::new(),
            mouse_x: 0.0,
            mouse_y: 0.0,
            draw_commands: Vec::new(),
            server_started: false,
            port: 8090,
        }
    }
}

pub fn open_browser(url: &str) {
    #[cfg(windows)]
    {
        let _ = Command::new("cmd").args(["/C", "start", url]).spawn();
    }
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(url).spawn();
    }
    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(url).spawn();
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        let _ = webbrowser::open(url);
    }
}

pub fn create_game2d_module(
    state_ref: Arc<Mutex<SharedGameState>>,
    callbacks_ref: Rc<RefCell<(Option<Value>, Option<Value>)>>,
) -> Value {
    let mut map = IndexMap::new();

    // game2d.window(w, h, title)
    let state_c = state_ref.clone();
    let window_fn: NativeFn = Rc::new(move |args: &[Value]| {
        let mut st = state_c.lock().unwrap();
        if let Some(w) = args.first() {
            st.width = match w {
                Value::Int(i) => *i as u32,
                Value::Float(f) => *f as u32,
                _ => 800,
            };
        }
        if let Some(h) = args.get(1) {
            st.height = match h {
                Value::Int(i) => *i as u32,
                Value::Float(f) => *f as u32,
                _ => 600,
            };
        }
        if let Some(t) = args.get(2) {
            st.title = t.to_string();
        }

        if !st.server_started {
            st.server_started = true;
            let port = st.port;
            let state_for_server = state_c.clone();

            thread::spawn(move || {
                run_http_server(port, state_for_server);
            });

            let url = format!("http://127.0.0.1:{}", port);
            println!("[Info] 2D Game Server running on {}", url);
            open_browser(&url);
        }

        Ok(Value::Null)
    });
    map.insert("window".to_string(), Value::NativeFunction("window".to_string(), window_fn));

    // game2d.on_update(func)
    let cb_c = callbacks_ref.clone();
    let update_fn: NativeFn = Rc::new(move |args: &[Value]| {
        if let Some(f) = args.first() {
            cb_c.borrow_mut().0 = Some(f.clone());
        }
        Ok(Value::Null)
    });
    map.insert("on_update".to_string(), Value::NativeFunction("on_update".to_string(), update_fn));

    // game2d.on_draw(func)
    let cb_c = callbacks_ref.clone();
    let draw_fn: NativeFn = Rc::new(move |args: &[Value]| {
        if let Some(f) = args.first() {
            cb_c.borrow_mut().1 = Some(f.clone());
        }
        Ok(Value::Null)
    });
    map.insert("on_draw".to_string(), Value::NativeFunction("on_draw".to_string(), draw_fn));

    // game2d.clear(color)
    let state_c = state_ref.clone();
    let clear_fn: NativeFn = Rc::new(move |args: &[Value]| {
        let color = args.first().map(|v| v.to_string()).unwrap_or_else(|| "#000000".to_string());
        state_c.lock().unwrap().draw_commands.push(DrawCommand::Clear { color });
        Ok(Value::Null)
    });
    map.insert("clear".to_string(), Value::NativeFunction("clear".to_string(), clear_fn));

    // game2d.rect(x, y, w, h, color)
    let state_c = state_ref.clone();
    let rect_fn: NativeFn = Rc::new(move |args: &[Value]| {
        let to_f64 = |idx: usize, def: f64| -> f64 {
            match args.get(idx) {
                Some(Value::Int(i)) => *i as f64,
                Some(Value::Float(f)) => *f,
                _ => def,
            }
        };
        let x = to_f64(0, 0.0);
        let y = to_f64(1, 0.0);
        let w = to_f64(2, 50.0);
        let h = to_f64(3, 50.0);
        let color = args.get(4).map(|v| v.to_string()).unwrap_or_else(|| "#ffffff".to_string());
        state_c.lock().unwrap().draw_commands.push(DrawCommand::Rect { x, y, w, h, color });
        Ok(Value::Null)
    });
    map.insert("rect".to_string(), Value::NativeFunction("rect".to_string(), rect_fn));

    // game2d.circle(x, y, r, color)
    let state_c = state_ref.clone();
    let circle_fn: NativeFn = Rc::new(move |args: &[Value]| {
        let to_f64 = |idx: usize, def: f64| -> f64 {
            match args.get(idx) {
                Some(Value::Int(i)) => *i as f64,
                Some(Value::Float(f)) => *f,
                _ => def,
            }
        };
        let x = to_f64(0, 0.0);
        let y = to_f64(1, 0.0);
        let r = to_f64(2, 25.0);
        let color = args.get(3).map(|v| v.to_string()).unwrap_or_else(|| "#ffffff".to_string());
        state_c.lock().unwrap().draw_commands.push(DrawCommand::Circle { x, y, r, color });
        Ok(Value::Null)
    });
    map.insert("circle".to_string(), Value::NativeFunction("circle".to_string(), circle_fn));

    // game2d.text(text, x, y, size, color)
    let state_c = state_ref.clone();
    let text_fn: NativeFn = Rc::new(move |args: &[Value]| {
        let text = args.first().map(|v| v.to_string()).unwrap_or_default();
        let to_f64 = |idx: usize, def: f64| -> f64 {
            match args.get(idx) {
                Some(Value::Int(i)) => *i as f64,
                Some(Value::Float(f)) => *f,
                _ => def,
            }
        };
        let x = to_f64(1, 0.0);
        let y = to_f64(2, 0.0);
        let size = to_f64(3, 16.0);
        let color = args.get(4).map(|v| v.to_string()).unwrap_or_else(|| "#ffffff".to_string());
        state_c.lock().unwrap().draw_commands.push(DrawCommand::Text { text, x, y, size, color });
        Ok(Value::Null)
    });
    map.insert("text".to_string(), Value::NativeFunction("text".to_string(), text_fn));

    // game2d.sprite(src, x, y, w, h)
    let state_c = state_ref.clone();
    let sprite_fn: NativeFn = Rc::new(move |args: &[Value]| {
        let src = args.first().map(|v| v.to_string()).unwrap_or_default();
        let to_f64 = |idx: usize, def: f64| -> f64 {
            match args.get(idx) {
                Some(Value::Int(i)) => *i as f64,
                Some(Value::Float(f)) => *f,
                _ => def,
            }
        };
        let x = to_f64(1, 0.0);
        let y = to_f64(2, 0.0);
        let w = to_f64(3, 50.0);
        let h = to_f64(4, 50.0);
        state_c.lock().unwrap().draw_commands.push(DrawCommand::Sprite { src, x, y, w, h });
        Ok(Value::Null)
    });
    map.insert("sprite".to_string(), Value::NativeFunction("sprite".to_string(), sprite_fn));

    // game2d.key_down(key)
    let state_c = state_ref.clone();
    let key_down_fn: NativeFn = Rc::new(move |args: &[Value]| {
        let key = args.first().map(|v| v.to_string()).unwrap_or_default();
        let is_down = state_c.lock().unwrap().keys_down.contains(&key);
        Ok(Value::Bool(is_down))
    });
    map.insert("key_down".to_string(), Value::NativeFunction("key_down".to_string(), key_down_fn));

    // game2d.mouse_x()
    let state_c = state_ref.clone();
    let mouse_x_fn: NativeFn = Rc::new(move |_args: &[Value]| {
        let mx = state_c.lock().unwrap().mouse_x;
        Ok(Value::Float(mx))
    });
    map.insert("mouse_x".to_string(), Value::NativeFunction("mouse_x".to_string(), mouse_x_fn));

    // game2d.mouse_y()
    let state_c = state_ref.clone();
    let mouse_y_fn: NativeFn = Rc::new(move |_args: &[Value]| {
        let my = state_c.lock().unwrap().mouse_y;
        Ok(Value::Float(my))
    });
    map.insert("mouse_y".to_string(), Value::NativeFunction("mouse_y".to_string(), mouse_y_fn));

    Value::Map(Rc::new(RefCell::new(map)))
}

fn run_http_server(port: u16, state: Arc<Mutex<SharedGameState>>) {
    let addr = format!("127.0.0.1:{}", port);
    let server = match tiny_http::Server::http(&addr) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("[Error] Failed to bind 2D Game server to {}: {}", addr, e);
            return;
        }
    };

    for mut request in server.incoming_requests() {
        let url = request.url().to_string();
        let path = url.split('?').next().unwrap_or("/");

        match path {
            "/" => {
                let (w, h, title) = {
                    let st = state.lock().unwrap();
                    (st.width, st.height, st.title.clone())
                };

                let html_page = format!(r#"<!DOCTYPE html>
<html lang="ru">
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <style>
        body {{
            background: #11111b;
            color: #cdd6f4;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            height: 100vh;
            margin: 0;
            overflow: hidden;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            user-select: none;
        }}
        #game-container {{
            border: 2px solid #89b4fa;
            border-radius: 8px;
            box-shadow: 0 12px 36px rgba(0,0,0,0.7);
            background: #181825;
            overflow: hidden;
        }}
        canvas {{ display: block; }}
        .status {{
            margin-top: 12px;
            font-size: 13px;
            color: #a6adc8;
        }}
    </style>
</head>
<body>
    <div id="game-container">
        <canvas id="canvas" width="{}" height="{}"></canvas>
    </div>
    <div class="status">[N&S HTML5 Canvas Engine • 60 FPS]</div>
    <script>
        const canvas = document.getElementById('canvas');
        const ctx = canvas.getContext('2d');
        const keys = new Set();
        let mouseX = 0, mouseY = 0;

        window.addEventListener('keydown', e => keys.add(e.code));
        window.addEventListener('keyup', e => keys.delete(e.code));
        canvas.addEventListener('mousemove', e => {{
            const rect = canvas.getBoundingClientRect();
            mouseX = e.clientX - rect.left;
            mouseY = e.clientY - rect.top;
        }});

        const images = {{}};
        function getImage(src) {{
            if (!images[src]) {{
                const img = new Image();
                img.src = src;
                images[src] = img;
            }}
            return images[src];
        }}

        async function sendInput() {{
            try {{
                await fetch('/input', {{
                    method: 'POST',
                    headers: {{ 'Content-Type': 'application/json' }},
                    body: JSON.stringify({{
                        keys: Array.from(keys),
                        mouseX: mouseX,
                        mouseY: mouseY
                    }})
                }});
            }} catch (err) {{}}
        }}

        async function renderFrame() {{
            try {{
                const res = await fetch('/frame');
                const data = await res.json();
                if (data.commands) {{
                    for (const cmd of data.commands) {{
                        if (cmd.type === 'clear') {{
                            ctx.fillStyle = cmd.color;
                            ctx.fillRect(0, 0, canvas.width, canvas.height);
                        }} else if (cmd.type === 'rect') {{
                            ctx.fillStyle = cmd.color;
                            ctx.fillRect(cmd.x, cmd.y, cmd.w, cmd.h);
                        }} else if (cmd.type === 'circle') {{
                            ctx.fillStyle = cmd.color;
                            ctx.beginPath();
                            ctx.arc(cmd.x, cmd.y, cmd.r, 0, Math.PI * 2);
                            ctx.fill();
                        }} else if (cmd.type === 'text') {{
                            ctx.fillStyle = cmd.color;
                            ctx.font = `${{cmd.size}}px sans-serif`;
                            ctx.fillText(cmd.text, cmd.x, cmd.y);
                        }} else if (cmd.type === 'sprite') {{
                            const img = getImage(cmd.src);
                            if (img.complete) {{
                                ctx.drawImage(img, cmd.x, cmd.y, cmd.w, cmd.h);
                            }}
                        }}
                    }}
                }}
            }} catch (err) {{}}
        }}

        async function loop() {{
            await sendInput();
            await renderFrame();
            requestAnimationFrame(loop);
        }}
        requestAnimationFrame(loop);
    </script>
</body>
</html>"#, title, w, h);

                let header = tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"text/html; charset=utf-8"[..],
                ).unwrap();
                let response = tiny_http::Response::from_string(html_page).with_header(header);
                let _ = request.respond(response);
            }
            "/frame" => {
                let commands = {
                    let st = state.lock().unwrap();
                    st.draw_commands.clone()
                };

                let resp_body = serde_json::json!({ "commands": commands }).to_string();
                let header = tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"application/json; charset=utf-8"[..],
                ).unwrap();
                let response = tiny_http::Response::from_string(resp_body).with_header(header);
                let _ = request.respond(response);
            }
            "/input" => {
                let mut body_str = String::new();
                let _ = request.as_reader().read_to_string(&mut body_str);

                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body_str) {
                    let mut st = state.lock().unwrap();
                    st.keys_down.clear();
                    if let Some(keys_arr) = json["keys"].as_array() {
                        for k in keys_arr {
                            if let Some(ks) = k.as_str() {
                                st.keys_down.insert(ks.to_string());
                            }
                        }
                    }
                    st.mouse_x = json["mouseX"].as_f64().unwrap_or(0.0);
                    st.mouse_y = json["mouseY"].as_f64().unwrap_or(0.0);
                }

                let header = tiny_http::Header::from_bytes(
                    &b"Content-Type"[..],
                    &b"application/json"[..],
                ).unwrap();
                let response = tiny_http::Response::from_string("{\"status\":\"ok\"}").with_header(header);
                let _ = request.respond(response);
            }
            _ => {
                let response = tiny_http::Response::from_string("404 Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    }
}
