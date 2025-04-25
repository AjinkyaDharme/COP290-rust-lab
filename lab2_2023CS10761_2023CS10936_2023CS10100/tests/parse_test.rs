use spreadsheet::command::{BinaryOp, Command, Expr, Function};
use spreadsheet::parser;

#[test]
fn test_parse_command_quit() {
    let result = parser::parse_command("q");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::Quit => {}
        _ => panic!("Expected Quit command"),
    }
}

#[test]
fn test_parse_scroll_commands() {
    // Test scroll directions
    for cmd in &["w", "a", "s", "d"] {
        let result = parser::parse_command(cmd);
        assert!(result.is_ok());

        let (_, command) = result.unwrap();
        match command {
            Command::ScrollUp
            | Command::ScrollDown
            | Command::ScrollLeft
            | Command::ScrollRight => {}
            _ => panic!("Expected scroll command"),
        }
    }

    // Test scroll_to
    let result = parser::parse_command("scroll_to A1");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::ScrollTo(_) => {}
        _ => panic!("Expected ScrollTo command"),
    }
}

#[test]
fn test_parse_simple_formula() {
    let result = parser::parse_command("A1=42");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::SetCell { cell, expr } => {
            assert_eq!(cell.row, 1);
            assert_eq!(cell.col, 1);

            match expr {
                Expr::Constant(val) => assert_eq!(val, 42),
                _ => panic!("Expected constant expression"),
            }
        }
        _ => panic!("Expected SetCell command"),
    }
}

#[test]
fn test_parse_complex_formula() {
    let result = parser::parse_command("B2=A1+5*C3");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::SetCell { cell, expr } => {
            assert_eq!(cell.row, 2);
            assert_eq!(cell.col, 2);

            // Check that expr is a binary operation (addition)
            match expr {
                Expr::BinaryOp(_, op, _) => {
                    assert!(matches!(op, BinaryOp::Add));
                    // Further checks for operands could be added
                }
                _ => panic!("Expected binary operation"),
            }
        }
        _ => panic!("Expected SetCell command"),
    }
}

#[test]
fn test_parse_function_call() {
    let result = parser::parse_command("C3=SUM(A1:B2)");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::SetCell { cell, expr } => {
            assert_eq!(cell.row, 3);
            assert_eq!(cell.col, 3);

            match expr {
                Expr::FunctionCall(func, _) => {
                    assert!(matches!(func, Function::Sum));
                }
                _ => panic!("Expected function call"),
            }
        }
        _ => panic!("Expected SetCell command"),
    }
}

#[test]
fn test_disable_output() {
    let result = parser::parse_command("disable_output");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::DisableOutput => {}
        _ => panic!("Expected DisableOutput command"),
    }
}

#[test]
fn test_enable_output() {
    let result = parser::parse_command("enable_output");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::EnableOutput => {}
        _ => panic!("Expected EnableOutput command"),
    }
}

#[test]
fn test_negative_constant() {
    let result = parser::parse_command("A1=-42");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::SetCell { cell, expr } => {
            assert_eq!(cell.row, 1);
            assert_eq!(cell.col, 1);

            match expr {
                Expr::Constant(val) => assert_eq!(val, -42),
                _ => panic!("Expected negative constant expression"),
            }
        }
        _ => panic!("Expected SetCell command"),
    }
}

#[test]
fn test_parse_parenthesized_expr() {
    let result = parser::parse_command("A1=(2+3)*4");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::SetCell { cell: _, expr } => match expr {
            Expr::BinaryOp(_, BinaryOp::Multiply, _) => {}
            _ => panic!("Expected parenthesized expression"),
        },
        _ => panic!("Expected SetCell command"),
    }
}

#[test]
fn test_parse_cell_ref_expr() {
    let result = parser::parse_command("A1=B2");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::SetCell { cell, expr } => {
            assert_eq!(cell.row, 1);
            assert_eq!(cell.col, 1);

            match expr {
                Expr::CellRef(cell_ref) => {
                    assert_eq!(cell_ref.row, 2);
                    assert_eq!(cell_ref.col, 2);
                }
                _ => panic!("Expected cell reference expression"),
            }
        }
        _ => panic!("Expected SetCell command"),
    }
}

#[test]
fn test_parse_range() {
    let result = parser::parse_command("A1=MIN(B2:C3)");
    assert!(result.is_ok());

    let (_, command) = result.unwrap();
    match command {
        Command::SetCell { cell: _, expr } => match expr {
            Expr::FunctionCall(Function::Min, boxed_arg) => match *boxed_arg {
                Expr::Range(start, end) => {
                    assert_eq!(start.col, 2);
                    assert_eq!(start.row, 2);
                    assert_eq!(end.col, 3);
                    assert_eq!(end.row, 3);
                }
                _ => panic!("Expected range expression"),
            },
            _ => panic!("Expected function call with range"),
        },
        _ => panic!("Expected SetCell command"),
    }
}

#[test]
fn test_parse_various_functions() {
    // Test other function types
    let functions = [
        ("MAX(A1:B2)", Function::Max),
        ("AVG(A1:B2)", Function::Avg),
        ("SUM(A1:B2)", Function::Sum),
        ("STDEV(A1:B2)", Function::Stdev),
        ("SLEEP(A1:B2)", Function::Sleep),
    ];

    for (func_str, expected_func) in &functions {
        let cmd = format!("D4={}", func_str);
        let result = parser::parse_command(&cmd);
        assert!(result.is_ok());

        let (_, command) = result.unwrap();
        match command {
            Command::SetCell { cell: _, expr } => match expr {
                Expr::FunctionCall(func, _) => {
                    assert!(matches!(func, f if f == *expected_func));
                }
                _ => panic!("Expected function call"),
            },
            _ => panic!("Expected SetCell command"),
        }
    }
}
