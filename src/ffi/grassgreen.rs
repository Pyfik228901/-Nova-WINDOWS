use crate::core::value::{NativeFn, Value};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::rc::Rc;

pub struct GrassGreenBridge {
    executable_path: Option<PathBuf>,
}

impl GrassGreenBridge {
    pub fn new() -> Self {
        let executable_path = Self::find_grassgreen();
        Self { executable_path }
    }

    /// Finds the GrassGreen executable (grg / grassc / gg.cmd) in the system
    pub fn find_grassgreen() -> Option<PathBuf> {
        // 1. Check GRASSGREEN_HOME environment variable
        if let Ok(home) = env::var("GRASSGREEN_HOME") {
            let candidates = [
                Path::new(&home).join("bin").join("grg.exe"),
                Path::new(&home).join("bin").join("grassc.exe"),
                Path::new(&home).join("bin").join("grg"),
                Path::new(&home).join("bin").join("grassc"),
                Path::new(&home).join("grg.exe"),
            ];
            for cand in &candidates {
                if cand.exists() {
                    return Some(cand.clone());
                }
            }
        }

        // 2. Check standard Windows path C:\GrassGreen\bin
        #[cfg(windows)]
        {
            let win_paths = [
                r"C:\GrassGreen\bin\grg.exe",
                r"C:\GrassGreen\bin\grassc.exe",
                r"C:\GrassGreen\grg.exe",
            ];
            for p in &win_paths {
                let path = PathBuf::from(p);
                if path.exists() {
                    return Some(path);
                }
            }
        }

        // 3. Check system PATH
        if let Ok(path_var) = env::var("PATH") {
            let sep = if cfg!(windows) { ';' } else { ':' };
            for dir in path_var.split(sep) {
                let dir_path = Path::new(dir);
                let candidates = if cfg!(windows) {
                    vec!["grg.exe", "grassc.exe"]
                } else {
                    vec!["grg", "grassc"]
                };
                for name in candidates {
                    let cand = dir_path.join(name);
                    if cand.is_file() {
                        return Some(cand);
                    }
                }
            }
        }

        None
    }

    pub fn print_not_found_message() {
        eprintln!();
        eprintln!("\x1b[33m[GrassGreen FFI] [Warning]\x1b[0m Компилятор GrassGreen (grg / grassc) не найден в системе.");
        eprintln!("Для выполнения блоков extern \"grass\" и файлов .grass установите GrassGreen:");
        eprintln!("  Документация и загрузка: \x1b[36mhttps://grassgreen.info\x1b[0m");
        eprintln!("  Исходный код: \x1b[36mhttps://github.com/m1oky/GrassGreen\x1b[0m");
        eprintln!();
    }

    /// Executes raw GrassGreen code from an `extern "grass"` block
    pub fn execute_raw_code(&self, code: &str) -> Value {
        let exe = match &self.executable_path {
            Some(path) => path,
            None => {
                Self::print_not_found_message();
                return Value::Null;
            }
        };

        // Prepare GrassGreen code (wrap in main if not present)
        let prepared_code = if !code.contains("!GrassGreen!") {
            if !code.contains("main(") {
                format!("!GrassGreen!\n\nint main() {{\n{}\n    return 0;\n}}\n", code)
            } else {
                format!("!GrassGreen!\n\n{}\n", code)
            }
        } else {
            code.to_string()
        };

        // Create a temporary .grass file
        let temp_dir = env::temp_dir();
        let file_name = format!("ns_extern_{}.grass", std::process::id());
        let temp_file = temp_dir.join(file_name);

        if let Err(e) = fs::write(&temp_file, prepared_code) {
            eprintln!("\x1b[31m[GrassGreen FFI] [Error]\x1b[0m Не удалось создать временный файл: {}", e);
            return Value::Null;
        }

        let output_res = Command::new(exe)
            .arg("-r")
            .arg(&temp_file)
            .output();

        let _ = fs::remove_file(&temp_file);

        match output_res {
            Ok(output) => {
                let raw_stdout = String::from_utf8_lossy(&output.stdout).to_string();
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                if !stderr.is_empty() {
                    eprintln!("\x1b[33m[GrassGreen Output]\x1b[0m {}", stderr.trim());
                }

                // Filter out runtime debug logs from output
                let filtered_lines: Vec<&str> = raw_stdout
                    .lines()
                    .filter(|line| !line.trim().starts_with("[GrassVM]:") && !line.trim().starts_with("[GrassGreen]:"))
                    .collect();
                let clean_output = filtered_lines.join("\n").trim().to_string();

                if clean_output.is_empty() {
                    return Value::Null;
                }

                println!("{}", clean_output);

                // If output looks like JSON, attempt to parse it
                if (clean_output.starts_with('{') && clean_output.ends_with('}'))
                    || (clean_output.starts_with('[') && clean_output.ends_with(']'))
                {
                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&clean_output) {
                        return Self::json_to_value(&json_val);
                    }
                }

                // If output is a number
                if let Ok(i) = clean_output.parse::<i64>() {
                    return Value::Int(i);
                }
                if let Ok(f) = clean_output.parse::<f64>() {
                    return Value::Float(f);
                }

                Value::String(clean_output)
            }
            Err(e) => {
                eprintln!("\x1b[31m[GrassGreen FFI] [Error]\x1b[0m Ошибка запуска: {}", e);
                Value::Null
            }
        }
    }

    /// Loads a .grass file as a module object in N&S
    pub fn import_grass_file(&self, file_path: &str) -> Value {
        let exe = match &self.executable_path {
            Some(path) => path.clone(),
            None => {
                Self::print_not_found_message();
                let mut map = IndexMap::new();
                map.insert("available".to_string(), Value::Bool(false));
                return Value::Map(Rc::new(RefCell::new(map)));
            }
        };

        let path = PathBuf::from(file_path);
        let mut map = IndexMap::new();
        map.insert("path".to_string(), Value::String(file_path.to_string()));
        map.insert("available".to_string(), Value::Bool(path.exists()));

        // Function: mod.run(args...)
        let exe_clone = exe.clone();
        let target_path = path.clone();
        let run_fn: NativeFn = Rc::new(move |args: &[Value]| {
            if !target_path.exists() {
                return Err(format!("GrassGreen file not found: {}", target_path.display()));
            }

            // Ensure running .grass file
            let actual_file_to_run = if target_path.extension().and_then(|s| s.to_str()) != Some("grass") {
                let grass_file = target_path.with_extension("grass");
                let _ = fs::copy(&target_path, &grass_file);
                grass_file
            } else {
                target_path.clone()
            };

            // If a function name is called via gg.func_name(args...)
            let temp_wrapper_opt = if !args.is_empty() {
                let first_is_ident = if let Value::String(s) = &args[0] {
                    !s.is_empty() && s.chars().all(|c| c.is_alphanumeric() || c == '_') && !s.chars().next().unwrap().is_ascii_digit()
                } else {
                    false
                };

                if first_is_ident {
                    let func_name = args[0].to_string();
                    let fn_args = &args[1..];
                    let formatted_args: Vec<String> = fn_args.iter().map(|a| a.to_string()).collect();
                    let canonical_path = fs::canonicalize(&actual_file_to_run)
                        .unwrap_or_else(|_| actual_file_to_run.clone())
                        .to_string_lossy()
                        .replace('\\', "/");

                    let wrapper_code = format!(
                        "!GrassGreen!\n\n#include \"{}\"\n\nint main() {{\n    gout << {}({}) << endl;\n    return 0;\n}}\n",
                        canonical_path,
                        func_name,
                        formatted_args.join(", ")
                    );

                    let temp_dir = env::temp_dir();
                    let temp_file = temp_dir.join(format!("ns_gg_call_{}.grass", std::process::id()));
                    if fs::write(&temp_file, wrapper_code).is_ok() {
                        Some(temp_file)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            let file_to_execute = temp_wrapper_opt.as_ref().unwrap_or(&actual_file_to_run);

            let mut cmd = Command::new(&exe_clone);
            cmd.arg("-r").arg(file_to_execute);

            let output_res = cmd.output();

            if let Some(ref temp_file) = temp_wrapper_opt {
                let _ = fs::remove_file(temp_file);
            }

            match output_res {
                Ok(output) => {
                    let raw_stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    if !stderr.is_empty() {
                        eprintln!("\x1b[33m[GrassGreen Stderr]\x1b[0m {}", stderr);
                    }

                    let clean_output: Vec<&str> = raw_stdout
                        .lines()
                        .filter(|l| !l.trim().starts_with("[GrassVM]:") && !l.trim().starts_with("[GrassGreen]:"))
                        .collect();
                    let result_str = clean_output.join("\n").trim().to_string();

                    println!("{}", result_str);

                    if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&result_str) {
                        Ok(Self::json_to_value(&json_val))
                    } else {
                        Ok(Value::String(result_str))
                    }
                }
                Err(e) => Err(format!("GrassGreen execution error: {}", e)),
            }
        });
        map.insert("run".to_string(), Value::NativeFunction("run".to_string(), run_fn));

        Value::Map(Rc::new(RefCell::new(map)))
    }

    pub fn json_to_value(json: &serde_json::Value) -> Value {
        match json {
            serde_json::Value::Null => Value::Null,
            serde_json::Value::Bool(b) => Value::Bool(*b),
            serde_json::Value::Number(num) => {
                if let Some(i) = num.as_i64() {
                    Value::Int(i)
                } else if let Some(f) = num.as_f64() {
                    Value::Float(f)
                } else {
                    Value::Null
                }
            }
            serde_json::Value::String(s) => Value::String(s.clone()),
            serde_json::Value::Array(arr) => {
                let elements: Vec<Value> = arr.iter().map(Self::json_to_value).collect();
                Value::Array(Rc::new(RefCell::new(elements)))
            }
            serde_json::Value::Object(obj) => {
                let mut map = IndexMap::new();
                for (k, v) in obj {
                    map.insert(k.clone(), Self::json_to_value(v));
                }
                Value::Map(Rc::new(RefCell::new(map)))
            }
        }
    }
}
