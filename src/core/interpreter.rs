use super::ast::*;
use super::env::Environment;
use super::lexer::Lexer;
use super::parser::Parser;
use super::value::{CustomFunction, NativeFn, Value};
use crate::ffi::grassgreen::GrassGreenBridge;
use crate::stdlib::cli_lib::create_cli_module;
use crate::stdlib::game2d::{create_game2d_module, SharedGameState};
use crate::stdlib::io::create_io_module;
use crate::stdlib::math::create_math_module;
use crate::stdlib::web::{create_web_module, value_to_json, WebRouter};
use indexmap::IndexMap;
use std::cell::RefCell;
use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

#[derive(Debug)]
pub enum ControlSignal {
    Return(Value),
    Error(String),
}

impl fmt::Display for ControlSignal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ControlSignal::Return(v) => write!(f, "Return({})", v),
            ControlSignal::Error(e) => write!(f, "{}", e),
        }
    }
}

pub struct Interpreter {
    pub global_env: Rc<RefCell<Environment>>,
    pub current_env: Rc<RefCell<Environment>>,
    pub web_router: Rc<RefCell<WebRouter>>,
    pub game2d_state: Arc<Mutex<SharedGameState>>,
    pub game2d_callbacks: Rc<RefCell<(Option<Value>, Option<Value>)>>, // (on_update, on_draw)
    pub gg_bridge: GrassGreenBridge,
    pub script_args: Vec<String>,
    pub current_file: Option<PathBuf>,
}

impl Interpreter {
    pub fn new(script_args: Vec<String>) -> Self {
        let global_env = Rc::new(RefCell::new(Environment::new()));
        let web_router = Rc::new(RefCell::new(WebRouter::new()));
        let game2d_state = Arc::new(Mutex::new(SharedGameState::new()));
        let game2d_callbacks = Rc::new(RefCell::new((None, None)));
        let gg_bridge = GrassGreenBridge::new();

        let mut interp = Self {
            global_env: global_env.clone(),
            current_env: global_env,
            web_router,
            game2d_state,
            game2d_callbacks,
            gg_bridge,
            script_args,
            current_file: None,
        };

        interp.register_builtins();
        interp
    }

    fn register_builtins(&mut self) {
        let mut env = self.global_env.borrow_mut();

        // 1. print(...)
        let print_fn: NativeFn = Rc::new(|args: &[Value]| {
            let s = args
                .iter()
                .map(|a| a.to_string())
                .collect::<Vec<String>>()
                .join(" ");
            println!("{}", s);
            Ok(Value::Null)
        });
        env.define("print", Value::NativeFunction("print".to_string(), print_fn));

        // 2. input(prompt)
        let input_fn: NativeFn = Rc::new(|args: &[Value]| {
            if let Some(prompt) = args.first() {
                print!("{}", prompt);
                let _ = std::io::stdout().flush();
            }
            let mut line = String::new();
            std::io::stdin()
                .read_line(&mut line)
                .map_err(|e| e.to_string())?;
            let trimmed = line.trim();

            if let Ok(i) = trimmed.parse::<i64>() {
                return Ok(Value::Int(i));
            }
            if let Ok(f) = trimmed.parse::<f64>() {
                return Ok(Value::Float(f));
            }
            if trimmed.eq_ignore_ascii_case("true") {
                return Ok(Value::Bool(true));
            }
            if trimmed.eq_ignore_ascii_case("false") {
                return Ok(Value::Bool(false));
            }
            Ok(Value::String(trimmed.to_string()))
        });
        env.define("input", Value::NativeFunction("input".to_string(), input_fn));

        // 3. random(min, max)
        let random_fn: NativeFn = Rc::new(|args: &[Value]| {
            let mut rng = rand::thread_rng();
            use rand::Rng;
            if args.len() >= 2 {
                let min_val = match &args[0] {
                    Value::Int(i) => *i,
                    Value::Float(f) => *f as i64,
                    _ => 0,
                };
                let max_val = match &args[1] {
                    Value::Int(i) => *i,
                    Value::Float(f) => *f as i64,
                    _ => 100,
                };
                if min_val > max_val {
                    Ok(Value::Int(rng.gen_range(max_val..=min_val)))
                } else {
                    Ok(Value::Int(rng.gen_range(min_val..=max_val)))
                }
            } else if args.len() == 1 {
                let max_val = match &args[0] {
                    Value::Int(i) => *i,
                    Value::Float(f) => *f as i64,
                    _ => 100,
                };
                Ok(Value::Int(rng.gen_range(0..=max_val)))
            } else {
                Ok(Value::Float(rng.gen::<f64>()))
            }
        });
        env.define("random", Value::NativeFunction("random".to_string(), random_fn));

        // 4. browse(url)
        let browse_fn: NativeFn = Rc::new(|args: &[Value]| {
            let target = args.first().ok_or("Expected URL or file path")?.to_string();
            println!("[Info] Открытие в браузере: {}", target);
            crate::stdlib::game2d::open_browser(&target);
            Ok(Value::Null)
        });
        env.define("browse", Value::NativeFunction("browse".to_string(), browse_fn));

        // 5. len(val)
        let len_fn: NativeFn = Rc::new(|args: &[Value]| {
            let val = args.first().ok_or("Expected argument for len()")?;
            match val {
                Value::String(s) => Ok(Value::Int(s.chars().count() as i64)),
                Value::Array(a) => Ok(Value::Int(a.borrow().len() as i64)),
                Value::Map(m) => Ok(Value::Int(m.borrow().len() as i64)),
                _ => Err(format!("Type '{}' has no length", val.type_name())),
            }
        });
        env.define("len", Value::NativeFunction("len".to_string(), len_fn));

        // 6. Presentation builtins
        let dummy_fn: NativeFn = Rc::new(|args: &[Value]| {
            let joined = args.iter().map(|a| a.to_string()).collect::<Vec<_>>().join(" - ");
            println!("[Presentation] {}", joined);
            Ok(Value::Null)
        });
        for name in &[
            "title",
            "header",
            "paragraph",
            "joystick_card",
            "presentation_start",
            "slide",
            "presentation_end",
            "button",
            "downloadTemplate",
        ] {
            env.define(name, Value::NativeFunction(name.to_string(), dummy_fn.clone()));
        }

        // exit(code)
        let exit_fn: NativeFn = Rc::new(|args: &[Value]| {
            let code = match args.first() {
                Some(Value::Int(c)) => *c as i32,
                _ => 0,
            };
            std::process::exit(code);
        });
        env.define("exit", Value::NativeFunction("exit".to_string(), exit_fn));
    }

    pub fn interpret(&mut self, program: &Program) -> Result<Value, String> {
        let mut last_val = Value::Null;
        for stmt in &program.statements {
            match self.execute_stmt(stmt) {
                Ok(val) => last_val = val,
                Err(ControlSignal::Return(v)) => return Ok(v),
                Err(ControlSignal::Error(e)) => return Err(e),
            }
        }
        Ok(last_val)
    }

    pub fn execute_stmt(&mut self, stmt: &Stmt) -> Result<Value, ControlSignal> {
        match stmt {
            Stmt::VarDecl { name, init } => {
                let val = self.eval_expr(init)?;
                self.current_env.borrow_mut().define(name, val.clone());
                Ok(val)
            }
            Stmt::Assignment { target, op, value } => {
                let rval = self.eval_expr(value)?;
                match target {
                    AssignTarget::Variable(name) => {
                        let final_val = if let Some(binop) = op {
                            let curr = self.current_env.borrow().get(name).unwrap_or(Value::Null);
                            self.apply_binary_op(&curr, binop, &rval)
                                .map_err(ControlSignal::Error)?
                        } else {
                            rval
                        };
                        self.current_env.borrow_mut().assign(name, final_val.clone());
                        Ok(final_val)
                    }
                    AssignTarget::Member(object_expr, member) => {
                        let obj = self.eval_expr(object_expr)?;
                        match obj {
                            Value::Map(map) => {
                                let final_val = if let Some(binop) = op {
                                    let curr = map.borrow().get(member).cloned().unwrap_or(Value::Null);
                                    self.apply_binary_op(&curr, binop, &rval)
                                        .map_err(ControlSignal::Error)?
                                } else {
                                    rval
                                };
                                map.borrow_mut().insert(member.clone(), final_val.clone());
                                Ok(final_val)
                            }
                            _ => Err(ControlSignal::Error(format!(
                                "Cannot assign property '{}' to non-map type '{}'",
                                member,
                                obj.type_name()
                            ))),
                        }
                    }
                    AssignTarget::Index(object_expr, index_expr) => {
                        let obj = self.eval_expr(object_expr)?;
                        let idx = self.eval_expr(index_expr)?;
                        match (obj, idx) {
                            (Value::Array(arr), Value::Int(i)) => {
                                let mut elements = arr.borrow_mut();
                                let len = elements.len() as i64;
                                let actual_idx = if i < 0 { len + i } else { i };
                                if actual_idx < 0 || actual_idx >= len {
                                    return Err(ControlSignal::Error(format!(
                                        "Array index {} out of bounds (len {})",
                                        i, len
                                    )));
                                }
                                let final_val = if let Some(binop) = op {
                                    let curr = &elements[actual_idx as usize];
                                    self.apply_binary_op(curr, binop, &rval)
                                        .map_err(ControlSignal::Error)?
                                } else {
                                    rval
                                };
                                elements[actual_idx as usize] = final_val.clone();
                                Ok(final_val)
                            }
                            (Value::Map(map), Value::String(key)) => {
                                let final_val = if let Some(binop) = op {
                                    let curr = map.borrow().get(&key).cloned().unwrap_or(Value::Null);
                                    self.apply_binary_op(&curr, binop, &rval)
                                        .map_err(ControlSignal::Error)?
                                } else {
                                    rval
                                };
                                map.borrow_mut().insert(key, final_val.clone());
                                Ok(final_val)
                            }
                            (other_obj, other_idx) => Err(ControlSignal::Error(format!(
                                "Cannot index '{}' with '{}'",
                                other_obj.type_name(),
                                other_idx.type_name()
                            ))),
                        }
                    }
                }
            }
            Stmt::Print(expr) => {
                let val = self.eval_expr(expr)?;
                println!("{}", val);
                Ok(val)
            }
            Stmt::Expr(expr) => self.eval_expr(expr),
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let cond_val = self.eval_expr(condition)?;
                if cond_val.is_truthy() {
                    self.execute_block(then_branch)
                } else if let Some(else_stmts) = else_branch {
                    self.execute_block(else_stmts)
                } else {
                    Ok(Value::Null)
                }
            }
            Stmt::While { condition, body } => {
                let mut last = Value::Null;
                while self.eval_expr(condition)?.is_truthy() {
                    last = self.execute_block(body)?;
                }
                Ok(last)
            }
            Stmt::For {
                item,
                iterable,
                body,
            } => {
                let iter_val = self.eval_expr(iterable)?;
                let mut last = Value::Null;

                match iter_val {
                    Value::Array(arr) => {
                        let elements = arr.borrow().clone();
                        for el in elements {
                            let child_env = Rc::new(RefCell::new(Environment::new_child(
                                self.current_env.clone(),
                            )));
                            child_env.borrow_mut().define(item, el);
                            let prev_env = self.current_env.clone();
                            self.current_env = child_env;
                            let res = self.execute_block_inner(body);
                            self.current_env = prev_env;
                            last = res?;
                        }
                    }
                    Value::Map(m) => {
                        let keys: Vec<String> = m.borrow().keys().cloned().collect();
                        for k in keys {
                            let child_env = Rc::new(RefCell::new(Environment::new_child(
                                self.current_env.clone(),
                            )));
                            child_env.borrow_mut().define(item, Value::String(k));
                            let prev_env = self.current_env.clone();
                            self.current_env = child_env;
                            let res = self.execute_block_inner(body);
                            self.current_env = prev_env;
                            last = res?;
                        }
                    }
                    Value::String(s) => {
                        for ch in s.chars() {
                            let child_env = Rc::new(RefCell::new(Environment::new_child(
                                self.current_env.clone(),
                            )));
                            child_env.borrow_mut().define(item, Value::String(ch.to_string()));
                            let prev_env = self.current_env.clone();
                            self.current_env = child_env;
                            let res = self.execute_block_inner(body);
                            self.current_env = prev_env;
                            last = res?;
                        }
                    }
                    _ => {
                        return Err(ControlSignal::Error(format!(
                            "Type '{}' is not iterable in for loop",
                            iter_val.type_name()
                        )));
                    }
                }
                Ok(last)
            }
            Stmt::FuncDef { name, params, body } => {
                let func = CustomFunction {
                    name: Some(name.clone()),
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.current_env.clone(),
                };
                let val = Value::Function(Rc::new(func));
                self.current_env.borrow_mut().define(name, val.clone());
                Ok(val)
            }
            Stmt::Return(expr_opt) => {
                let val = if let Some(e) = expr_opt {
                    self.eval_expr(e)?
                } else {
                    Value::Null
                };
                Err(ControlSignal::Return(val))
            }
            Stmt::Import {
                module_path,
                alias,
                is_grass,
            } => {
                if *is_grass || module_path.ends_with(".grass") {
                    let resolved_path = self.resolve_file_path(module_path);
                    let path_str = resolved_path.to_str().unwrap_or(module_path);
                    let mod_val = self.gg_bridge.import_grass_file(path_str);
                    let name = alias.clone().unwrap_or_else(|| {
                        Path::new(module_path)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("grass_mod")
                            .to_string()
                    });
                    self.current_env.borrow_mut().define(&name, mod_val);
                    return Ok(Value::Null);
                }

                // Check standard modules
                let mod_val = match module_path.as_str() {
                    "io" => create_io_module(),
                    "math" => create_math_module(),
                    "cli" => create_cli_module(&self.script_args),
                    "web" => {
                        let mut web_map = match create_web_module(self.web_router.clone()) {
                            Value::Map(m) => m.borrow().clone(),
                            _ => IndexMap::new(),
                        };

                        let router_c = self.web_router.clone();
                        let global_env_c = self.global_env.clone();
                        let args_c = self.script_args.clone();
                        let server_fn: NativeFn = Rc::new(move |args: &[Value]| {
                            let port = match args.first() {
                                Some(Value::Int(p)) => *p as u16,
                                Some(Value::Float(f)) => *f as u16,
                                _ => 8080,
                            };
                            Self::run_web_server(port, router_c.clone(), global_env_c.clone(), args_c.clone())
                        });
                        web_map.insert("server".to_string(), Value::NativeFunction("server".to_string(), server_fn.clone()));
                        web_map.insert("listen".to_string(), Value::NativeFunction("listen".to_string(), server_fn));

                        Value::Map(Rc::new(RefCell::new(web_map)))
                    }
                    "game2d" => {
                        let mut game_map = match create_game2d_module(self.game2d_state.clone(), self.game2d_callbacks.clone()) {
                            Value::Map(m) => m.borrow().clone(),
                            _ => IndexMap::new(),
                        };

                        let state_c = self.game2d_state.clone();
                        let cb_c = self.game2d_callbacks.clone();
                        let global_env_c = self.global_env.clone();
                        let args_c = self.script_args.clone();

                        let start_fn: NativeFn = Rc::new(move |_args: &[Value]| {
                            println!("[Info] Starting 2D Game loop (press Ctrl+C to stop)...");
                            let mut last_time = Instant::now();
                            let frame_duration = Duration::from_millis(16); // ~60 FPS

                            let (update_fn, draw_fn) = {
                                let cb = cb_c.borrow();
                                (cb.0.clone(), cb.1.clone())
                            };

                            let mut interp = Interpreter::new(args_c.clone());
                            interp.global_env = global_env_c.clone();
                            interp.current_env = global_env_c.clone();

                            loop {
                                let now = Instant::now();
                                let dt = now.duration_since(last_time).as_secs_f64();
                                last_time = now;

                                if let Some(ref u_fn) = update_fn {
                                    let _ = interp.call_function(u_fn, &[Value::Float(dt)]);
                                }

                                state_c.lock().unwrap().draw_commands.clear();

                                if let Some(ref d_fn) = draw_fn {
                                    let _ = interp.call_function(d_fn, &[]);
                                }

                                let elapsed = now.elapsed();
                                if elapsed < frame_duration {
                                    thread::sleep(frame_duration - elapsed);
                                }
                            }
                        });
                        game_map.insert("start".to_string(), Value::NativeFunction("start".to_string(), start_fn));

                        Value::Map(Rc::new(RefCell::new(game_map)))
                    }
                    _ => {
                        let file_path = self.resolve_file_path(module_path);
                        if !file_path.exists() {
                            return Err(ControlSignal::Error(format!(
                                "Import error: Module or file '{}' not found",
                                module_path
                            )));
                        }

                        let code = fs::read_to_string(&file_path).map_err(|e| {
                            ControlSignal::Error(format!(
                                "Failed to read module file '{}': {}",
                                file_path.display(),
                                e
                            ))
                        })?;

                        let mut lexer = Lexer::new(&code);
                        let tokens = lexer.tokenize().map_err(|e| ControlSignal::Error(e.to_string()))?;
                        let mut parser = Parser::new(&code, tokens);
                        let program = parser.parse().map_err(|e| ControlSignal::Error(e.to_string()))?;

                        let module_env = Rc::new(RefCell::new(Environment::new_child(self.global_env.clone())));
                        let mut sub_interp = Interpreter::new(self.script_args.clone());
                        sub_interp.current_file = Some(file_path.clone());
                        sub_interp.current_env = module_env.clone();

                        sub_interp.interpret(&program).map_err(ControlSignal::Error)?;

                        let mut exports = IndexMap::new();
                        for key in module_env.borrow().keys() {
                            if let Some(val) = module_env.borrow().get(&key) {
                                exports.insert(key, val);
                            }
                        }
                        Value::Map(Rc::new(RefCell::new(exports)))
                    }
                };

                let name = alias.clone().unwrap_or_else(|| {
                    Path::new(module_path)
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or(module_path)
                        .to_string()
                });
                self.current_env.borrow_mut().define(&name, mod_val);
                Ok(Value::Null)
            }
            Stmt::ExternBlock { language, code } => {
                if language.eq_ignore_ascii_case("grass") || language.eq_ignore_ascii_case("gg") {
                    let res = self.gg_bridge.execute_raw_code(code);
                    Ok(res)
                } else {
                    eprintln!(
                        "\x1b[33m[Warning]\x1b[0m Unsupported extern language '{}'",
                        language
                    );
                    Ok(Value::Null)
                }
            }
            Stmt::Block(stmts) => self.execute_block(stmts),
        }
    }

    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<Value, ControlSignal> {
        let child_env = Rc::new(RefCell::new(Environment::new_child(self.current_env.clone())));
        let prev_env = self.current_env.clone();
        self.current_env = child_env;
        let res = self.execute_block_inner(stmts);
        self.current_env = prev_env;
        res
    }

    fn execute_block_inner(&mut self, stmts: &[Stmt]) -> Result<Value, ControlSignal> {
        let mut last_val = Value::Null;
        for s in stmts {
            last_val = self.execute_stmt(s)?;
        }
        Ok(last_val)
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, ControlSignal> {
        match expr {
            Expr::Literal(lit) => match lit {
                LiteralValue::Int(i) => Ok(Value::Int(*i)),
                LiteralValue::Float(f) => Ok(Value::Float(*f)),
                LiteralValue::String(s) => Ok(Value::String(s.clone())),
                LiteralValue::Bool(b) => Ok(Value::Bool(*b)),
                LiteralValue::Null => Ok(Value::Null),
            },
            Expr::Variable(name) => {
                if let Some(val) = self.current_env.borrow().get(name) {
                    Ok(val)
                } else {
                    Err(ControlSignal::Error(format!("Undefined variable '{}'", name)))
                }
            }
            Expr::Binary { left, op, right } => {
                let lval = self.eval_expr(left)?;

                if *op == BinaryOp::And {
                    if !lval.is_truthy() {
                        return Ok(Value::Bool(false));
                    }
                    let rval = self.eval_expr(right)?;
                    return Ok(Value::Bool(rval.is_truthy()));
                }
                if *op == BinaryOp::Or {
                    if lval.is_truthy() {
                        return Ok(Value::Bool(true));
                    }
                    let rval = self.eval_expr(right)?;
                    return Ok(Value::Bool(rval.is_truthy()));
                }

                let rval = self.eval_expr(right)?;
                self.apply_binary_op(&lval, op, &rval).map_err(ControlSignal::Error)
            }
            Expr::Unary { op, expr } => {
                let val = self.eval_expr(expr)?;
                match op {
                    UnaryOp::Neg => val.neg().map_err(ControlSignal::Error),
                    UnaryOp::Not => val.not().map_err(ControlSignal::Error),
                }
            }
            Expr::Call { callee, args } => {
                let func_val = self.eval_expr(callee)?;
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg)?);
                }
                self.call_function(&func_val, &evaluated_args)
            }
            Expr::MemberAccess { object, member } => {
                let obj = self.eval_expr(object)?;
                match obj {
                    Value::Map(map) => {
                        if let Some(val) = map.borrow().get(member) {
                            Ok(val.clone())
                        } else if map.borrow().get("available").is_some() && map.borrow().contains_key("run") {
                            // GrassGreen module dynamic function call
                            if let Some(Value::NativeFunction(_, run_fn)) = map.borrow().get("run").cloned() {
                                let member_name = member.clone();
                                let dyn_fn: NativeFn = Rc::new(move |args: &[Value]| {
                                    let mut full_args = vec![Value::String(member_name.clone())];
                                    full_args.extend_from_slice(args);
                                    run_fn(&full_args)
                                });
                                Ok(Value::NativeFunction(member.clone(), dyn_fn))
                            } else {
                                Ok(Value::Null)
                            }
                        } else {
                            Ok(Value::Null)
                        }
                    }
                    Value::String(s) => match member.as_str() {
                        "length" => Ok(Value::Int(s.chars().count() as i64)),
                        "upper" => {
                            let s_clone = s.clone();
                            let fn_val: NativeFn = Rc::new(move |_| Ok(Value::String(s_clone.to_uppercase())));
                            Ok(Value::NativeFunction("upper".to_string(), fn_val))
                        }
                        "lower" => {
                            let s_clone = s.clone();
                            let fn_val: NativeFn = Rc::new(move |_| Ok(Value::String(s_clone.to_lowercase())));
                            Ok(Value::NativeFunction("lower".to_string(), fn_val))
                        }
                        "trim" => {
                            let s_clone = s.clone();
                            let fn_val: NativeFn = Rc::new(move |_| Ok(Value::String(s_clone.trim().to_string())));
                            Ok(Value::NativeFunction("trim".to_string(), fn_val))
                        }
                        _ => Ok(Value::Null),
                    },
                    Value::Array(arr) => match member.as_str() {
                        "length" => Ok(Value::Int(arr.borrow().len() as i64)),
                        "push" => {
                            let arr_c = arr.clone();
                            let fn_val: NativeFn = Rc::new(move |args: &[Value]| {
                                if let Some(item) = args.first() {
                                    arr_c.borrow_mut().push(item.clone());
                                }
                                Ok(Value::Null)
                            });
                            Ok(Value::NativeFunction("push".to_string(), fn_val))
                        }
                        "pop" => {
                            let arr_c = arr.clone();
                            let fn_val: NativeFn = Rc::new(move |_| {
                                Ok(arr_c.borrow_mut().pop().unwrap_or(Value::Null))
                            });
                            Ok(Value::NativeFunction("pop".to_string(), fn_val))
                        }
                        _ => Ok(Value::Null),
                    },
                    _ => Ok(Value::Null),
                }
            }
            Expr::IndexAccess { object, index } => {
                let obj = self.eval_expr(object)?;
                let idx = self.eval_expr(index)?;
                match (obj, idx) {
                    (Value::Array(arr), Value::Int(i)) => {
                        let elements = arr.borrow();
                        let len = elements.len() as i64;
                        let actual_idx = if i < 0 { len + i } else { i };
                        if actual_idx >= 0 && (actual_idx as usize) < elements.len() {
                            Ok(elements[actual_idx as usize].clone())
                        } else {
                            Ok(Value::Null)
                        }
                    }
                    (Value::Map(map), Value::String(key)) => {
                        Ok(map.borrow().get(&key).cloned().unwrap_or(Value::Null))
                    }
                    (Value::String(s), Value::Int(i)) => {
                        let chars: Vec<char> = s.chars().collect();
                        let len = chars.len() as i64;
                        let actual_idx = if i < 0 { len + i } else { i };
                        if actual_idx >= 0 && (actual_idx as usize) < chars.len() {
                            Ok(Value::String(chars[actual_idx as usize].to_string()))
                        } else {
                            Ok(Value::Null)
                        }
                    }
                    (other_obj, other_idx) => Err(ControlSignal::Error(format!(
                        "Cannot index '{}' with '{}'",
                        other_obj.type_name(),
                        other_idx.type_name()
                    ))),
                }
            }
            Expr::ArrayLiteral(elements) => {
                let mut vals = Vec::new();
                for el in elements {
                    vals.push(self.eval_expr(el)?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(vals))))
            }
            Expr::MapLiteral(entries) => {
                let mut map = IndexMap::new();
                for (k, v) in entries {
                    map.insert(k.clone(), self.eval_expr(v)?);
                }
                Ok(Value::Map(Rc::new(RefCell::new(map))))
            }
            Expr::FunctionExpr { name, params, body } => {
                let func = CustomFunction {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.current_env.clone(),
                };
                let val = Value::Function(Rc::new(func));
                if let Some(n) = name {
                    self.current_env.borrow_mut().define(n, val.clone());
                }
                Ok(val)
            }
        }
    }

    pub fn call_function(&mut self, callee: &Value, args: &[Value]) -> Result<Value, ControlSignal> {
        match callee {
            Value::NativeFunction(_, native_fn) => {
                native_fn(args).map_err(ControlSignal::Error)
            }
            Value::Function(custom_fn) => {
                if args.len() != custom_fn.params.len() {
                    let fn_name = custom_fn.name.as_deref().unwrap_or("anonymous");
                    return Err(ControlSignal::Error(format!(
                        "Function '{}' expects {} arguments, but {} were provided",
                        fn_name,
                        custom_fn.params.len(),
                        args.len()
                    )));
                }

                let child_env = Rc::new(RefCell::new(Environment::new_child(custom_fn.closure.clone())));

                for (i, param) in custom_fn.params.iter().enumerate() {
                    let arg_val = args[i].clone();
                    child_env.borrow_mut().define(param, arg_val);
                }

                let prev_env = self.current_env.clone();
                self.current_env = child_env;

                let mut result = Value::Null;
                for stmt in &custom_fn.body {
                    match self.execute_stmt(stmt) {
                        Ok(v) => result = v,
                        Err(ControlSignal::Return(ret_val)) => {
                            self.current_env = prev_env;
                            return Ok(ret_val);
                        }
                        Err(ControlSignal::Error(e)) => {
                            self.current_env = prev_env;
                            return Err(ControlSignal::Error(e));
                        }
                    }
                }

                self.current_env = prev_env;
                Ok(result)
            }
            _ => Err(ControlSignal::Error(format!(
                "Cannot call non-function of type '{}'",
                callee.type_name()
            ))),
        }
    }

    fn apply_binary_op(&self, left: &Value, op: &BinaryOp, right: &Value) -> Result<Value, String> {
        match op {
            BinaryOp::Add => left.add(right),
            BinaryOp::Sub => left.sub(right),
            BinaryOp::Mul => left.mul(right),
            BinaryOp::Div => left.div(right),
            BinaryOp::Mod => left.rem(right),
            BinaryOp::Eq => Ok(Value::Bool(left == right)),
            BinaryOp::NotEq => Ok(Value::Bool(left != right)),
            BinaryOp::Lt => {
                let ord = left.compare(right)?;
                Ok(Value::Bool(ord == std::cmp::Ordering::Less))
            }
            BinaryOp::Gt => {
                let ord = left.compare(right)?;
                Ok(Value::Bool(ord == std::cmp::Ordering::Greater))
            }
            BinaryOp::LtEq => {
                let ord = left.compare(right)?;
                Ok(Value::Bool(ord != std::cmp::Ordering::Greater))
            }
            BinaryOp::GtEq => {
                let ord = left.compare(right)?;
                Ok(Value::Bool(ord != std::cmp::Ordering::Less))
            }
            BinaryOp::And | BinaryOp::Or => unreachable!(),
        }
    }

    fn resolve_file_path(&self, path_str: &str) -> PathBuf {
        let p = Path::new(path_str);
        if p.is_absolute() {
            return p.to_path_buf();
        }

        if let Some(curr) = &self.current_file {
            if let Some(parent) = curr.parent() {
                let resolved = parent.join(p);
                if resolved.exists() {
                    return resolved;
                }
            }
        }

        PathBuf::from(path_str)
    }

    fn run_web_server(
        port: u16,
        router: Rc<RefCell<WebRouter>>,
        global_env: Rc<RefCell<Environment>>,
        args: Vec<String>,
    ) -> Result<Value, String> {
        let addr = format!("0.0.0.0:{}", port);
        let server = tiny_http::Server::http(&addr)
            .map_err(|e| format!("Failed to start HTTP server on {}: {}", addr, e))?;

        println!();
        println!("\x1b[32m[OK] N&S Web Server running on http://localhost:{}\x1b[0m", port);
        println!("[Info] Press Ctrl+C to stop server.");
        println!();

        for mut request in server.incoming_requests() {
            let method = request.method().as_str().to_uppercase();
            let url = request.url().to_string();
            let path = url.split('?').next().unwrap_or("/").to_string();

            let mut body_str = String::new();
            let _ = request.as_reader().read_to_string(&mut body_str);

            let mut should_stop = path == "/shutdown";
            let handler_opt = router.borrow().routes.get(&(method.clone(), path.clone())).cloned();

            if let Some(handler) = handler_opt {
                let mut req_map = IndexMap::new();
                req_map.insert("method".to_string(), Value::String(method));
                req_map.insert("path".to_string(), Value::String(path));
                req_map.insert("body".to_string(), Value::String(body_str));
                let req_val = Value::Map(Rc::new(RefCell::new(req_map)));

                let mut interp = Interpreter::new(args.clone());
                interp.global_env = global_env.clone();
                interp.current_env = global_env.clone();

                match interp.call_function(&handler, &[req_val]) {
                    Ok(resp_val) => {
                        if let Value::Map(m) = &resp_val {
                            if let Some(Value::Bool(true)) = m.borrow().get("shutdown") {
                                should_stop = true;
                            }
                        }

                        let (content_type, body_data) = match resp_val {
                            Value::String(s) => {
                                if s.trim().starts_with('<') {
                                    ("text/html; charset=utf-8", s)
                                } else if s.trim().starts_with('{') || s.trim().starts_with('[') {
                                    ("application/json; charset=utf-8", s)
                                } else {
                                    ("text/plain; charset=utf-8", s)
                                }
                            }
                            Value::Map(_) | Value::Array(_) => {
                                let json = value_to_json(&resp_val);
                                ("application/json; charset=utf-8", json.to_string())
                            }
                            other => ("text/plain; charset=utf-8", other.to_string()),
                        };

                        let header = tiny_http::Header::from_bytes(
                            &b"Content-Type"[..],
                            content_type.as_bytes(),
                        )
                        .unwrap();
                        let response = tiny_http::Response::from_string(body_data).with_header(header);
                        let _ = request.respond(response);
                    }
                    Err(e) => {
                        let err_msg = format!("Internal Server Error: {}", e);
                        let response = tiny_http::Response::from_string(err_msg).with_status_code(500);
                        let _ = request.respond(response);
                    }
                }
            } else if path == "/shutdown" {
                let response = tiny_http::Response::from_string(r#"{"status":"shutdown"}"#);
                let _ = request.respond(response);
            } else {
                let response = tiny_http::Response::from_string("404 Not Found").with_status_code(404);
                let _ = request.respond(response);
            }

            if should_stop {
                println!("[Info] Web server received shutdown signal. Stopping web server.");
                break;
            }
        }

        Ok(Value::Null)
    }
}
