use ns::core::interpreter::Interpreter;
use ns::core::lexer::Lexer;
use ns::core::parser::Parser;
use ns::core::value::Value;
use ns::stdlib::game2d::DrawCommand;

fn eval_code(code: &str) -> Result<Value, String> {
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().map_err(|e| e.to_string())?;
    let mut parser = Parser::new(code, tokens);
    let program = parser.parse().map_err(|e| e.to_string())?;
    let mut interpreter = Interpreter::new(vec![]);
    interpreter.interpret(&program)
}

#[test]
fn test_basic_arithmetic_and_precedence() {
    let result = eval_code("let a = 2 + 3 * 4; a;").expect("Evaluation failed");
    assert_eq!(result, Value::Int(14));

    let result2 = eval_code("let b = (2 + 3) * 4; b;").expect("Evaluation failed");
    assert_eq!(result2, Value::Int(20));

    let result3 = eval_code("let c = 10 - 4 - 2; c;").expect("Evaluation failed");
    assert_eq!(result3, Value::Int(4));
}

#[test]
fn test_comparison_and_logic() {
    let res1 = eval_code("let x = 10 > 5 && 3 < 4; x;").expect("Evaluation failed");
    assert_eq!(res1, Value::Bool(true));

    let res2 = eval_code("let y = 10 == 20 || 5 != 5; y;").expect("Evaluation failed");
    assert_eq!(res2, Value::Bool(false));

    let res3 = eval_code("let z = !false; z;").expect("Evaluation failed");
    assert_eq!(res3, Value::Bool(true));
}

#[test]
fn test_functions_and_recursion() {
    let code = r#"
        func factorial(n) {
            if (n <= 1) {
                return 1;
            }
            return n * factorial(n - 1);
        }
        factorial(5);
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result, Value::Int(120));
}

#[test]
fn test_scope_isolation() {
    let code = r#"
        let x = 100;
        func modify() {
            let x = 42;
            return x;
        }
        let inner = modify();
        [inner, x];
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    if let Value::Array(arr) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed[0], Value::Int(42));
        assert_eq!(borrowed[1], Value::Int(100));
    } else {
        panic!("Expected array return");
    }
}

#[test]
fn test_arrays_and_maps() {
    let code = r#"
        let list = [1, 2, 3];
        list[1] = 99;
        let dict = { "key": 42, "items": list };
        dict.items[1];
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result, Value::Int(99));
}

#[test]
fn test_loops_while_and_for() {
    let code = r#"
        let sum = 0;
        let i = 1;
        while (i <= 5) {
            sum = sum + i;
            i = i + 1;
        }

        let arr = [10, 20, 30];
        let arr_sum = 0;
        for val in arr {
            arr_sum = arr_sum + val;
        }

        [sum, arr_sum];
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    if let Value::Array(arr) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed[0], Value::Int(15));
        assert_eq!(borrowed[1], Value::Int(60));
    } else {
        panic!("Expected array return");
    }
}

#[test]
fn test_negative_division_by_zero() {
    let result = eval_code("let a = 10 / 0;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Division by zero"));
}

#[test]
fn test_negative_undefined_variable() {
    let result = eval_code("let a = undeclared_var + 1;");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Undefined variable 'undeclared_var'"));
}

#[test]
fn test_negative_function_arity() {
    let code = r#"
        func add(a, b) { return a + b; }
        add(1);
    "#;
    let result = eval_code(code);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("expects 2 arguments, but 1 were provided"));
}

#[test]
fn test_negative_syntax_error_unclosed_brace() {
    let src = "{ let x = 1;";
    let mut lexer = Lexer::new(src);
    let tokens = lexer.tokenize().expect("Lexing should succeed");
    let mut parser = Parser::new(src, tokens);
    let result = parser.parse();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Expected '}' after block"));
}

#[test]
fn test_negative_lexer_error_unclosed_quote() {
    let mut lexer = Lexer::new("\"unclosed string");
    let result = lexer.tokenize();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unterminated string literal"));
}

#[test]
fn test_stdlib_math() {
    let code = r#"
        import "math" as math
        let s = math.sqrt(16);
        let p = math.pow(2, 4);
        let a = math.abs(-42);
        [s, p, a];
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    if let Value::Array(arr) = result {
        let borrowed = arr.borrow();
        assert_eq!(borrowed[0], Value::Float(4.0));
        assert_eq!(borrowed[1], Value::Float(16.0));
        assert_eq!(borrowed[2], Value::Int(42));
    } else {
        panic!("Expected array return");
    }
}

#[test]
fn test_stdlib_web_json() {
    let code = r#"
        import "web" as web
        let data = { "name": "Nova", "code": 101, "active": true };
        let str = web.json_stringify(data);
        let parsed = web.json_parse(str);
        parsed.name == "Nova" && parsed.code == 101 && parsed.active == true;
    "#;
    let result = eval_code(code).expect("Evaluation failed");
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_game2d_frame_buffer() {
    let interp = Interpreter::new(vec![]);
    let state = interp.game2d_state.clone();

    // Call game2d drawing commands via code
    let code = r##"
        import "game2d" as game2d
        game2d.clear("#11111b");
        game2d.rect(10, 20, 100, 50, "#89b4fa");
        game2d.circle(200, 150, 30, "#a6e3a1");
        game2d.text("Hello Canvas", 50, 80, 16, "#cdd6f4");
    "##;
    let mut lexer = Lexer::new(code);
    let tokens = lexer.tokenize().expect("Lexing failed");
    let mut parser = Parser::new(code, tokens);
    let program = parser.parse().expect("Parsing failed");
    let mut local_interp = Interpreter::new(vec![]);
    local_interp.game2d_state = state.clone();
    local_interp.interpret(&program).expect("Interpret failed");

    // Check captured frame buffer
    let st = state.lock().unwrap();
    assert_eq!(st.draw_commands.len(), 4);

    match &st.draw_commands[0] {
        DrawCommand::Clear { color } => assert_eq!(color, "#11111b"),
        _ => panic!("Expected Clear command"),
    }
    match &st.draw_commands[1] {
        DrawCommand::Rect { x, y, w, h, color } => {
            assert_eq!(*x, 10.0);
            assert_eq!(*y, 20.0);
            assert_eq!(*w, 100.0);
            assert_eq!(*h, 50.0);
            assert_eq!(color, "#89b4fa");
        }
        _ => panic!("Expected Rect command"),
    }
    match &st.draw_commands[2] {
        DrawCommand::Circle { x, y, r, color } => {
            assert_eq!(*x, 200.0);
            assert_eq!(*y, 150.0);
            assert_eq!(*r, 30.0);
            assert_eq!(color, "#a6e3a1");
        }
        _ => panic!("Expected Circle command"),
    }
    match &st.draw_commands[3] {
        DrawCommand::Text { text, x, y, size, color } => {
            assert_eq!(text, "Hello Canvas");
            assert_eq!(*x, 50.0);
            assert_eq!(*y, 80.0);
            assert_eq!(*size, 16.0);
            assert_eq!(color, "#cdd6f4");
        }
        _ => panic!("Expected Text command"),
    }
}
