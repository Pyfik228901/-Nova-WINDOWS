mod cli;
mod core;
mod ffi;
mod stdlib;
mod tooling;

use clap::{Parser as ClapParser, Subcommand};
use core::interpreter::Interpreter;
use core::lexer::Lexer;
use core::parser::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;

#[derive(ClapParser)]
#[command(
    name = "ns",
    author = "m1oky",
    version = "1.0.0",
    about = "N&S (Nova Script) — высокопроизводительный, модульный язык программирования",
    long_about = "N&S (Nova Script) — интерпретируемый язык нового поколения, созданный на Rust.\nПоддерживает модули web, game2d, cli, math, io и прямую интеграцию с языком GrassGreen (extern \"grass\")."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Выполнить код из строки (inline evaluation)
    #[arg(short = 'e', long = "eval")]
    eval: Option<String>,

    /// Прямой запуск файла скрипта (ns script.ns) или аргументы
    #[arg(trailing_var_arg = true)]
    direct_args: Vec<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Запуск интерактивной консоли REPL
    Repl,
    /// Запуск программы на языке N&S (.ns или .nova)
    Run {
        /// Путь к файлу программы
        file: String,
        /// Аргументы для скрипта
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },
    /// Создание шаблона нового проекта
    Init {
        /// Имя создаваемого проекта
        name: String,
    },
    /// Установка подсветки синтаксиса и сниппетов для VS Code
    #[command(name = "setup-vscode")]
    SetupVscode,
    /// Регистрация ассоциаций файлов .ns и .nova и установка иконки в Windows Explorer
    #[command(name = "register")]
    Register,
    /// Запуск Language Server Protocol через stdin/stdout
    Lsp,
}

fn main() {
    let cli = Cli::parse();

    if let Some(code) = cli.eval {
        run_code(&code, cli.direct_args);
        return;
    }

    match cli.command {
        Some(Commands::Repl) => {
            cli::repl::start_repl();
        }
        Some(Commands::Run { file, args }) => {
            run_file(&file, args);
        }
        Some(Commands::Init { name }) => {
            init_project(&name);
        }
        Some(Commands::SetupVscode) => {
            if let Err(e) = tooling::vscode::install_vscode_extension() {
                eprintln!("\x1b[31mОшибка установки расширения VS Code:\x1b[0m {}", e);
                exit(1);
            }
            let _ = tooling::assoc::register_associations();
        }
        Some(Commands::Register) => {
            if let Err(e) = tooling::assoc::register_associations() {
                eprintln!("\x1b[31m[Error]\x1b[0m {}", e);
                exit(1);
            }
        }
        Some(Commands::Lsp) => {
            tooling::lsp::run_lsp_server();
        }
        None => {
            if cli.direct_args.is_empty() {
                // ns без аргументов запускает REPL
                cli::repl::start_repl();
            } else {
                let first = &cli.direct_args[0];
                match first.as_str() {
                    "repl" => cli::repl::start_repl(),
                    "setup-vscode" => {
                        if let Err(e) = tooling::vscode::install_vscode_extension() {
                            eprintln!("\x1b[31mОшибка:\x1b[0m {}", e);
                            exit(1);
                        }
                        let _ = tooling::assoc::register_associations();
                    }
                    "register" | "associate" => {
                        if let Err(e) = tooling::assoc::register_associations() {
                            eprintln!("\x1b[31m[Error]\x1b[0m {}", e);
                            exit(1);
                        }
                    }
                    "lsp" => tooling::lsp::run_lsp_server(),
                    "init" => {
                        let name = cli.direct_args.get(1).cloned().unwrap_or_else(|| "my_ns_project".to_string());
                        init_project(&name);
                    }
                    file => {
                        let args = cli.direct_args[1..].to_vec();
                        run_file(file, args);
                    }
                }
            }
        }
    }
}

fn run_code(code: &str, args: Vec<String>) {
    let mut lexer = Lexer::new(code);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("\x1b[31m[Lexer Error]\x1b[0m {}", e);
            exit(1);
        }
    };

    let mut parser = Parser::new(code, tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("\x1b[31m[Syntax Error]\x1b[0m {}", e);
            exit(1);
        }
    };

    let mut interpreter = Interpreter::new(args);
    if let Err(e) = interpreter.interpret(&program) {
        eprintln!("\x1b[31m[Runtime Error]\x1b[0m {}", e);
        exit(1);
    }
}

fn run_file(file_path: &str, args: Vec<String>) {
    let path = PathBuf::from(file_path);
    if !path.exists() {
        eprintln!("\x1b[31m[Error]\x1b[0m Файл '{}' не найден!", file_path);
        exit(1);
    }

    let code = match fs::read_to_string(&path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("\x1b[31m[Error]\x1b[0m Ошибка чтения файла '{}': {}", file_path, e);
            exit(1);
        }
    };

    let mut lexer = Lexer::new(&code);
    let tokens = match lexer.tokenize() {
        Ok(t) => t,
        Err(e) => {
            eprintln!("\x1b[31m[Lexer Error]\x1b[0m {}", e);
            exit(1);
        }
    };

    let mut parser = Parser::new(&code, tokens);
    let program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("\x1b[31m[Syntax Error]\x1b[0m {}", e);
            exit(1);
        }
    };

    let mut interpreter = Interpreter::new(args);
    interpreter.current_file = Some(path.clone());

    if let Err(e) = interpreter.interpret(&program) {
        eprintln!("\x1b[31m[Runtime Error]\x1b[0m {}", e);
        exit(1);
    }
}

fn init_project(name: &str) {
    let project_dir = Path::new(name);
    let src_dir = project_dir.join("src");

    if let Err(e) = fs::create_dir_all(&src_dir) {
        eprintln!("\x1b[31m[Error] Ошибка создания директории '{}':\x1b[0m {}", name, e);
        return;
    }

    let main_ns_content = format!(
        "// Точка входа в проект N&S\nimport \"cli\"\n\nprint(cli.bold(cli.green(\"[OK] Проект '{}' успешно инициализирован!\")))\nprint(\"Для запуска используйте: ns src/main.ns\")\n",
        name
    );

    let gitignore_content = "# Временные файлы N&S\ntarget/\n*.tmp\n*.log\n";

    let readme_content = format!(
        "# {}\n\nПроект на языке программирования **N&S (Nova Script)**.\n\n## Запуск\n```bash\nns src/main.ns\n```\n",
        name
    );

    let _ = fs::write(src_dir.join("main.ns"), main_ns_content);
    let _ = fs::write(project_dir.join(".gitignore"), gitignore_content);
    let _ = fs::write(project_dir.join("README.md"), readme_content);

    println!();
    println!("\x1b[32m[OK] Новый проект N&S успешно создан в папке '{}'.\x1b[0m", name);
    println!("  - {}/src/main.ns", name);
    println!("  - {}/.gitignore", name);
    println!("  - {}/README.md", name);
    println!();
    println!("Для запуска выполните:");
    println!("  cd {}", name);
    println!("  ns src/main.ns");
    println!();
}
