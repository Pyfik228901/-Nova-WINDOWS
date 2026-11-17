use crate::core::interpreter::Interpreter;
use crate::core::lexer::Lexer;
use crate::core::parser::Parser;
use crate::core::value::Value;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::env;

pub fn start_repl() {
    println!();
    println!("\x1b[1;36m[N&S REPL v1.0.0]\x1b[0m");
    println!("Интерактивная консоль языка N&S. Введите выражение или команду.");
    println!("Поддерживается многострочный ввод блоков {{ }}.");
    println!("Для выхода введите \x1b[33mexit\x1b[0m или нажмите \x1b[33mCtrl+C / Ctrl+D\x1b[0m.");
    println!();

    let mut rl = match DefaultEditor::new() {
        Ok(editor) => editor,
        Err(e) => {
            eprintln!("Ошибка инициализации терминала REPL: {}", e);
            return;
        }
    };

    let history_file = env::temp_dir().join(".ns_history");
    let _ = rl.load_history(&history_file);

    let mut interpreter = Interpreter::new(vec![]);
    let mut multiline_buffer = String::new();

    loop {
        let prompt = if multiline_buffer.is_empty() {
            "\x1b[1;32mns>\x1b[0m "
        } else {
            "\x1b[1;33m... \x1b[0m "
        };

        match rl.readline(prompt) {
            Ok(line) => {
                let trimmed = line.trim();
                if multiline_buffer.is_empty() && (trimmed == "exit" || trimmed == "quit") {
                    println!("До свидания!");
                    break;
                }

                if multiline_buffer.is_empty() {
                    let _ = rl.add_history_entry(line.as_str());
                }

                multiline_buffer.push_str(&line);
                multiline_buffer.push('\n');

                // Check brace and paren balance for multiline input
                if is_balanced(&multiline_buffer) {
                    let code = multiline_buffer.trim().to_string();
                    multiline_buffer.clear();

                    if code.is_empty() {
                        continue;
                    }

                    let mut lexer = Lexer::new(&code);
                    let tokens = match lexer.tokenize() {
                        Ok(toks) => toks,
                        Err(e) => {
                            eprintln!("\x1b[31m[Lexer Error]\x1b[0m {}", e);
                            continue;
                        }
                    };

                    let mut parser = Parser::new(&code, tokens);
                    let program = match parser.parse() {
                        Ok(prog) => prog,
                        Err(e) => {
                            eprintln!("\x1b[31m[Syntax Error]\x1b[0m {}", e);
                            continue;
                        }
                    };

                    match interpreter.interpret(&program) {
                        Ok(val) => {
                            if val != Value::Null {
                                println!("\x1b[36m=>\x1b[0m {}", val);
                            }
                        }
                        Err(e) => {
                            eprintln!("\x1b[31m[Runtime Error]\x1b[0m {}", e);
                        }
                    }
                }
            }
            Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
                println!("\nВыход из REPL.");
                break;
            }
            Err(err) => {
                eprintln!("Ошибка чтения: {:?}", err);
                break;
            }
        }
    }

    let _ = rl.save_history(&history_file);
}

fn is_balanced(code: &str) -> bool {
    let mut braces = 0;
    let mut parens = 0;
    let mut in_string = false;
    let mut quote_char = ' ';
    let mut prev_char = ' ';

    for ch in code.chars() {
        if in_string {
            if ch == quote_char && prev_char != '\\' {
                in_string = false;
            }
        } else {
            match ch {
                '"' | '\'' => {
                    in_string = true;
                    quote_char = ch;
                }
                '{' => braces += 1,
                '}' => braces -= 1,
                '(' => parens += 1,
                ')' => parens -= 1,
                _ => {}
            }
        }
        prev_char = ch;
    }

    braces <= 0 && parens <= 0 && !in_string
}
