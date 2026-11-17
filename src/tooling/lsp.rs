use crate::core::lexer::Lexer;
use crate::core::parser::Parser;
use serde_json::{json, Value as JsonValue};
use std::io::{self, BufRead, Read, Write};

pub fn run_lsp_server() {
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let stdout = io::stdout();
    let mut writer = stdout.lock();

    loop {
        let mut line = String::new();
        let bytes_read = match reader.read_line(&mut line) {
            Ok(b) => b,
            Err(_) => break,
        };

        if bytes_read == 0 {
            break;
        }

        let line_trimmed = line.trim();
        if line_trimmed.starts_with("Content-Length:") {
            let length_str = line_trimmed.trim_start_matches("Content-Length:").trim();
            let content_length: usize = match length_str.parse() {
                Ok(l) => l,
                Err(_) => continue,
            };

            // Read until empty line separator \r\n\r\n
            let mut empty_line = String::new();
            while let Ok(b) = reader.read_line(&mut empty_line) {
                if b == 0 || empty_line.trim().is_empty() {
                    break;
                }
                empty_line.clear();
            }

            // Read exact payload
            let mut body_buf = vec![0u8; content_length];
            if reader.read_exact(&mut body_buf).is_err() {
                break;
            }

            if let Ok(msg) = serde_json::from_slice::<JsonValue>(&body_buf) {
                handle_lsp_message(&msg, &mut writer);
            }
        }
    }
}

fn handle_lsp_message(msg: &JsonValue, writer: &mut impl Write) {
    let method = msg["method"].as_str().unwrap_or_default();
    let id = &msg["id"];

    match method {
        "initialize" => {
            let result = json!({
                "capabilities": {
                    "textDocumentSync": 1, // Full sync
                    "completionProvider": {
                        "resolveProvider": false,
                        "triggerCharacters": [".", " ", "(", "\"", "'"]
                    }
                },
                "serverInfo": {
                    "name": "ns-lsp",
                    "version": "1.0.0"
                }
            });
            send_response(writer, id, result);
        }
        "initialized" => {
            // Notification: no response needed
        }
        "textDocument/didOpen" => {
            if let Some(text) = msg["params"]["textDocument"]["text"].as_str() {
                let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or_default();
                publish_diagnostics(writer, uri, text);
            }
        }
        "textDocument/didChange" => {
            if let Some(content_changes) = msg["params"]["contentChanges"].as_array() {
                if let Some(last_change) = content_changes.last() {
                    if let Some(text) = last_change["text"].as_str() {
                        let uri = msg["params"]["textDocument"]["uri"].as_str().unwrap_or_default();
                        publish_diagnostics(writer, uri, text);
                    }
                }
            }
        }
        "textDocument/completion" => {
            let completions = get_completions();
            send_response(writer, id, json!(completions));
        }
        "shutdown" => {
            send_response(writer, id, json!(null));
        }
        "exit" => {
            std::process::exit(0);
        }
        _ => {
            // For unhandled requests with an ID, return null result
            if !id.is_null() {
                send_response(writer, id, json!(null));
            }
        }
    }
}

fn publish_diagnostics(writer: &mut impl Write, uri: &str, text: &str) {
    let mut diagnostics = Vec::new();

    let mut lexer = Lexer::new(text);
    match lexer.tokenize() {
        Ok(tokens) => {
            let mut parser = Parser::new(text, tokens);
            if let Err(e) = parser.parse() {
                let line = e.line.saturating_sub(1);
                let col = e.column.saturating_sub(1);
                diagnostics.push(json!({
                    "range": {
                        "start": { "line": line, "character": col },
                        "end": { "line": line, "character": col + 5 }
                    },
                    "severity": 1, // Error
                    "source": "ns",
                    "message": e.message
                }));
            }
        }
        Err(e) => {
            let line = e.line.saturating_sub(1);
            let col = e.column.saturating_sub(1);
            diagnostics.push(json!({
                "range": {
                    "start": { "line": line, "character": col },
                    "end": { "line": line, "character": col + 1 }
                },
                "severity": 1, // Error
                "source": "ns",
                "message": e.message
            }));
        }
    }

    let notification = json!({
        "jsonrpc": "2.0",
        "method": "textDocument/publishDiagnostics",
        "params": {
            "uri": uri,
            "diagnostics": diagnostics
        }
    });

    send_notification(writer, notification);
}

fn get_completions() -> Vec<JsonValue> {
    let mut items = Vec::new();

    let keywords = [
        ("let", "Объявление переменной", 14),
        ("func", "Объявление функции", 14),
        ("if", "Условный оператор", 14),
        ("else", "Ветка иначе", 14),
        ("while", "Цикл while", 14),
        ("for", "Цикл for-in", 14),
        ("in", "Оператор in", 14),
        ("return", "Возврат значения", 14),
        ("import", "Импорт модуля", 14),
        ("as", "Псевдоним модуля", 14),
        ("extern", "Встраивание чужого языка (extern \"grass\")", 14),
        ("true", "Логическая истина", 21),
        ("false", "Логическая ложь", 21),
        ("null", "Нулевое значение", 21),
    ];

    for (kw, doc, kind) in keywords {
        items.push(json!({
            "label": kw,
            "kind": kind,
            "detail": doc
        }));
    }

    let builtins = [
        ("print(expr)", "Печать в консоль", 3),
        ("input(prompt)", "Чтение строки из терминала", 3),
        ("random(min, max)", "Случайное число", 3),
        ("browse(url)", "Открыть в веб-браузере", 3),
        ("len(collection)", "Длина коллекции или строки", 3),
    ];

    for (b, doc, kind) in builtins {
        items.push(json!({
            "label": b.split('(').next().unwrap_or(b),
            "kind": kind,
            "detail": doc,
            "insertText": b
        }));
    }

    let modules = [
        ("web", "HTTP сервер, JSON, шаблоны", 9),
        ("game2d", "2D игровой движок HTML5 Canvas", 9),
        ("cli", "Цвета ANSI, аргументы командной строки", 9),
        ("math", "Математические функции и константы", 9),
        ("io", "Файловый ввод-вывод", 9),
    ];

    for (m, doc, kind) in modules {
        items.push(json!({
            "label": m,
            "kind": kind,
            "detail": doc
        }));
    }

    items
}

fn send_response(writer: &mut impl Write, id: &JsonValue, result: JsonValue) {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    });
    send_payload(writer, resp);
}

fn send_notification(writer: &mut impl Write, notification: JsonValue) {
    send_payload(writer, notification);
}

fn send_payload(writer: &mut impl Write, payload: JsonValue) {
    if let Ok(body) = serde_json::to_string(&payload) {
        let msg = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        let _ = writer.write_all(msg.as_bytes());
        let _ = writer.flush();
    }
}
