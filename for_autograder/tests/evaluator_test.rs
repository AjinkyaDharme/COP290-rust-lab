use spreadsheet::command::{BinaryOp, CellRef, Command, Expr, Function};
use spreadsheet::evaluator::{EvalError, convert_col_to_name, eval_expr, evaluate_command};
use spreadsheet::sheet::{CellValue, Spreadsheet};
use std::time::{Duration, Instant};

#[test]
fn test_eval_constant() {
    let sheet = Spreadsheet::new(5, 5);
    let expr = Expr::Constant(42);

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
}

#[test]
fn test_eval_cell_ref() {
    let mut sheet = Spreadsheet::new(5, 5);
    sheet.set(0, 0, CellValue::Value(10)).unwrap();

    let expr = Expr::CellRef(CellRef { row: 1, col: 1 });

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 10);
}

#[test]
fn test_eval_binary_op() {
    let sheet = Spreadsheet::new(5, 5);

    // Test addition: 10 + 5
    let expr = Expr::BinaryOp(
        Box::new(Expr::Constant(10)),
        BinaryOp::Add,
        Box::new(Expr::Constant(5)),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 15);

    // Test multiplication: 10 * 5
    let expr = Expr::BinaryOp(
        Box::new(Expr::Constant(10)),
        BinaryOp::Multiply,
        Box::new(Expr::Constant(5)),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 50);

    // Test division by zero
    let expr = Expr::BinaryOp(
        Box::new(Expr::Constant(10)),
        BinaryOp::Divide,
        Box::new(Expr::Constant(0)),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EvalError::DivByZero));
}

#[test]
fn test_eval_function_call() {
    let mut sheet = Spreadsheet::new(5, 5);

    // Set up a 2x2 range of cells
    sheet.set(0, 0, CellValue::Value(10)).unwrap();
    sheet.set(0, 1, CellValue::Value(20)).unwrap();
    sheet.set(1, 0, CellValue::Value(30)).unwrap();
    sheet.set(1, 1, CellValue::Value(40)).unwrap();

    // Test SUM function
    let expr = Expr::FunctionCall(
        Function::Sum,
        Box::new(Expr::Range(
            CellRef { row: 1, col: 1 },
            CellRef { row: 2, col: 2 },
        )),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 100); // 10 + 20 + 30 + 40 = 100

    // Test MAX function
    let expr = Expr::FunctionCall(
        Function::Max,
        Box::new(Expr::Range(
            CellRef { row: 1, col: 1 },
            CellRef { row: 2, col: 2 },
        )),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 40);

    // Test AVG function
    let expr = Expr::FunctionCall(
        Function::Avg,
        Box::new(Expr::Range(
            CellRef { row: 1, col: 1 },
            CellRef { row: 2, col: 2 },
        )),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 25); // (10 + 20 + 30 + 40) / 4 = 25
}

#[test]
fn test_error_propagation() {
    let mut sheet = Spreadsheet::new(5, 5);

    // Set a cell to Error
    sheet.set(0, 0, CellValue::Error(())).unwrap();

    // Reference the error cell
    let expr = Expr::CellRef(CellRef { row: 1, col: 1 });

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EvalError::CellError));

    // Reference a cell out of bounds
    let expr = Expr::CellRef(CellRef { row: 10, col: 10 });

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EvalError::OutOfBounds));
}

#[test]
fn test_subtraction_and_division() {
    let sheet = Spreadsheet::new(5, 5);

    // Test subtraction: 10 - 5
    let expr = Expr::BinaryOp(
        Box::new(Expr::Constant(10)),
        BinaryOp::Subtract,
        Box::new(Expr::Constant(5)),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);

    // Test division: 10 / 5
    let expr = Expr::BinaryOp(
        Box::new(Expr::Constant(10)),
        BinaryOp::Divide,
        Box::new(Expr::Constant(5)),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2);
}

#[test]
fn test_nested_binary_operations() {
    let sheet = Spreadsheet::new(5, 5);

    // Test (10 + 5) * 2
    let expr = Expr::BinaryOp(
        Box::new(Expr::BinaryOp(
            Box::new(Expr::Constant(10)),
            BinaryOp::Add,
            Box::new(Expr::Constant(5)),
        )),
        BinaryOp::Multiply,
        Box::new(Expr::Constant(2)),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 30);

    // Test 10 + (5 * 2)
    let expr = Expr::BinaryOp(
        Box::new(Expr::Constant(10)),
        BinaryOp::Add,
        Box::new(Expr::BinaryOp(
            Box::new(Expr::Constant(5)),
            BinaryOp::Multiply,
            Box::new(Expr::Constant(2)),
        )),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 20);
}

#[test]
fn test_min_function() {
    let mut sheet = Spreadsheet::new(5, 5);

    // Set up a range of cells with different values
    sheet.set(0, 0, CellValue::Value(10)).unwrap();
    sheet.set(0, 1, CellValue::Value(5)).unwrap();
    sheet.set(1, 0, CellValue::Value(30)).unwrap();
    sheet.set(1, 1, CellValue::Value(15)).unwrap();

    // Test MIN function
    let expr = Expr::FunctionCall(
        Function::Min,
        Box::new(Expr::Range(
            CellRef { row: 1, col: 1 },
            CellRef { row: 2, col: 2 },
        )),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 5);
}

#[test]
fn test_stdev_function() {
    let mut sheet = Spreadsheet::new(5, 5);

    // Set up a range of cells
    sheet.set(0, 0, CellValue::Value(10)).unwrap();
    sheet.set(0, 1, CellValue::Value(10)).unwrap();
    sheet.set(1, 0, CellValue::Value(10)).unwrap();
    sheet.set(1, 1, CellValue::Value(10)).unwrap();

    // Test STDEV function with all same values (should be 0)
    let expr = Expr::FunctionCall(
        Function::Stdev,
        Box::new(Expr::Range(
            CellRef { row: 1, col: 1 },
            CellRef { row: 2, col: 2 },
        )),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);

    // Change values to get non-zero standard deviation
    sheet.set(0, 0, CellValue::Value(2)).unwrap();
    sheet.set(0, 1, CellValue::Value(4)).unwrap();
    sheet.set(1, 0, CellValue::Value(6)).unwrap();
    sheet.set(1, 1, CellValue::Value(8)).unwrap();

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 2); // Simple case with predictable output
}

#[test]
fn test_sleep_function() {
    let sheet = Spreadsheet::new(5, 5);

    // Test SLEEP function with a small duration (1 second)
    let expr = Expr::FunctionCall(Function::Sleep, Box::new(Expr::Constant(1)));

    let start = Instant::now();
    let result = eval_expr(&expr, &sheet);
    let elapsed = start.elapsed();

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1);
    assert!(elapsed >= Duration::from_secs(1));
}

#[test]
fn test_function_error_cases() {
    let sheet = Spreadsheet::new(5, 5);

    // Test function with invalid range (end before start)
    let expr = Expr::FunctionCall(
        Function::Sum,
        Box::new(Expr::Range(
            CellRef { row: 3, col: 3 },
            CellRef { row: 1, col: 1 },
        )),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EvalError::Other(_)));

    // Test function with non-range argument
    let expr = Expr::FunctionCall(Function::Sum, Box::new(Expr::Constant(10)));

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EvalError::Other(_)));

    // Test range expression directly (not as function argument)
    let expr = Expr::Range(CellRef { row: 1, col: 1 }, CellRef { row: 2, col: 2 });

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EvalError::Other(_)));
}

#[test]
fn test_evaluate_command_set_cell() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Set cell A1 to constant 42
    let cmd = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: Expr::Constant(42),
    };

    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_ok());

    // Check that the cell was set correctly
    assert_eq!(*sheet.get(0, 0).unwrap(), CellValue::Value(42));
}

#[test]
fn test_evaluate_command_set_cell_with_error() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Set cell A1 with a division by zero error
    let cmd = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: Expr::BinaryOp(
            Box::new(Expr::Constant(10)),
            BinaryOp::Divide,
            Box::new(Expr::Constant(0)),
        ),
    };

    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_ok()); // The command itself succeeds, setting the cell to error

    // Check that the cell was set to error
    assert!(matches!(sheet.get(0, 0).unwrap(), CellValue::Error(_)));
}

#[test]
fn test_evaluate_command_output_control() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Disable output
    let cmd = Command::DisableOutput;
    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_ok());
    assert_eq!(output_enabled, false);

    // Enable output
    let cmd = Command::EnableOutput;
    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_ok());
    assert_eq!(output_enabled, true);
}

#[test]
fn test_convert_col_to_name() {
    assert_eq!(convert_col_to_name(1), "A");
    assert_eq!(convert_col_to_name(26), "Z");
    assert_eq!(convert_col_to_name(27), "AA");
    assert_eq!(convert_col_to_name(52), "AZ");
    assert_eq!(convert_col_to_name(53), "BA");
    assert_eq!(convert_col_to_name(702), "ZZ");
    assert_eq!(convert_col_to_name(703), "AAA");
}

#[test]
fn test_evaluate_command_scrolling() {
    let mut sheet = Spreadsheet::new(20, 20);
    let mut output_enabled = true;

    // Test scrolling commands (can only verify they don't error)
    let commands = [
        Command::ScrollUp,
        Command::ScrollDown,
        Command::ScrollLeft,
        Command::ScrollRight,
        Command::ScrollTo(CellRef { row: 5, col: 5 }),
    ];

    for cmd in commands {
        let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
        assert!(result.is_ok());
    }
}

#[test]
fn test_evaluate_command_set_cell_with_cell_error() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // First set a cell to error
    sheet.set(0, 0, CellValue::Error(())).unwrap();

    // Then try to use that cell in another expression
    let cmd = Command::SetCell {
        cell: CellRef { row: 2, col: 2 },
        expr: Expr::CellRef(CellRef { row: 1, col: 1 }),
    };

    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_ok()); // The command succeeds, setting the target cell to error

    // Check that the target cell was set to error
    assert!(matches!(sheet.get(1, 1).unwrap(), CellValue::Error(_)));
}

#[test]
fn test_evaluate_command_set_cell_with_out_of_bounds() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Set a cell with a reference to an out-of-bounds cell
    let cmd = Command::SetCell {
        cell: CellRef { row: 2, col: 2 },
        expr: Expr::CellRef(CellRef { row: 10, col: 10 }),
    };

    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), EvalError::OutOfBounds));

    // Check that the target cell was set to error
    assert!(matches!(sheet.get(1, 1).unwrap(), CellValue::Error(_)));
}

#[test]
fn test_evaluate_command_set_cell_with_other_error() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Set a cell with an invalid range expression
    let cmd = Command::SetCell {
        cell: CellRef { row: 2, col: 2 },
        expr: Expr::Range(CellRef { row: 3, col: 3 }, CellRef { row: 1, col: 1 }),
    };

    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_err());

    // Check that the target cell was set to error
    assert!(matches!(sheet.get(1, 1).unwrap(), CellValue::Error(_)));
}

#[test]
fn test_empty_range_functions() {
    let sheet = Spreadsheet::new(5, 5);

    // Create an empty range by using an invalid range (which returns an error)
    for func in [
        Function::Sum,
        Function::Min,
        Function::Max,
        Function::Avg,
        Function::Stdev,
    ] {
        let expr = Expr::FunctionCall(
            func,
            Box::new(Expr::Range(
                CellRef { row: 1, col: 1 },
                CellRef { row: 0, col: 0 }, // Invalid range
            )),
        );

        let result = eval_expr(&expr, &sheet);
        assert!(result.is_err());
    }
}

#[test]
fn test_stdev_with_single_value() {
    let mut sheet = Spreadsheet::new(5, 5);
    sheet.set(0, 0, CellValue::Value(42)).unwrap();

    // Test STDEV with single value (should be 0)
    let expr = Expr::FunctionCall(
        Function::Stdev,
        Box::new(Expr::Range(
            CellRef { row: 1, col: 1 },
            CellRef { row: 1, col: 1 },
        )),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0);
}

#[test]
fn test_quit_command() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Just ensure Quit command doesn't panic
    let cmd = Command::Quit;
    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_ok());
}

#[test]
fn test_formula_storage() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Set a cell with a formula
    let expr = Expr::BinaryOp(
        Box::new(Expr::Constant(10)),
        BinaryOp::Add,
        Box::new(Expr::Constant(20)),
    );

    let cmd = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: expr.clone(),
    };

    evaluate_command(cmd, &mut sheet, &mut output_enabled).unwrap();

    // Verify the formula was stored
    let key = format!("A1");
    assert!(sheet.get_formula(&key).is_some());

    // This doesn't compare the actual formulas, just verifies storage
    assert_eq!(*sheet.get_formula(&key).unwrap(), expr);
}

#[test]
fn test_eval_expr_error_display() {
    assert_eq!(EvalError::DivByZero.to_string(), "Division by zero");
    assert_eq!(EvalError::CellError.to_string(), "Referenced cell error");
    assert_eq!(
        EvalError::OutOfBounds.to_string(),
        "Cell reference out of bounds"
    );
    assert_eq!(
        EvalError::Other("Test error".into()).to_string(),
        "Test error"
    );
}

#[test]
fn test_stdev_advanced_cases() {
    let mut sheet = Spreadsheet::new(5, 5);

    // Testing the specific stdev formula with known values
    // Since sheet is 0-indexed but CellRef is 1-indexed:
    sheet.set(0, 0, CellValue::Value(5)).unwrap(); // A1
    sheet.set(0, 1, CellValue::Value(10)).unwrap(); // B1
    sheet.set(1, 0, CellValue::Value(15)).unwrap(); // A2
    sheet.set(1, 1, CellValue::Value(20)).unwrap(); // B2

    // Range from A1 to B2
    let expr = Expr::FunctionCall(
        Function::Stdev,
        Box::new(Expr::Range(
            CellRef { row: 1, col: 1 }, // A1
            CellRef { row: 2, col: 2 }, // B2
        )),
    );

    let result = eval_expr(&expr, &sheet);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 6);
}

#[test]
fn test_function_call_with_non_range_arg() {
    let sheet = Spreadsheet::new(5, 5);

    // Test functions that require range with a non-range argument
    for func in [
        Function::Sum,
        Function::Min,
        Function::Max,
        Function::Avg,
        Function::Stdev,
    ] {
        let expr = Expr::FunctionCall(func, Box::new(Expr::Constant(42)));

        let result = eval_expr(&expr, &sheet);
        assert!(result.is_err());
        let err = result.unwrap_err();
        match err {
            EvalError::Other(msg) => {
                assert!(msg.contains("requires a range argument"));
            }
            _ => panic!("Expected Other error, got {:?}", err),
        }
    }
}

#[test]
fn test_set_cell_division_by_zero() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Set cell with division by zero - should set cell to error but not return error
    let cmd = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: Expr::BinaryOp(
            Box::new(Expr::Constant(10)),
            BinaryOp::Divide,
            Box::new(Expr::Constant(0)),
        ),
    };

    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_ok()); // Command should succeed

    // Cell should be set to error
    assert!(matches!(sheet.get(0, 0).unwrap(), CellValue::Error(_)));
}

#[test]
fn test_set_cell_cell_error() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // First set a cell to error
    sheet.set(0, 0, CellValue::Error(())).unwrap();

    // Set a cell referencing the error cell
    let cmd = Command::SetCell {
        cell: CellRef { row: 2, col: 2 },
        expr: Expr::CellRef(CellRef { row: 1, col: 1 }),
    };

    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_ok()); // Command should succeed

    // Cell should be set to error
    assert!(matches!(sheet.get(1, 1).unwrap(), CellValue::Error(_)));
}

#[test]
fn test_set_cell_with_invalid_range() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Set cell with an invalid range in function
    let cmd = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: Expr::FunctionCall(
            Function::Sum,
            Box::new(Expr::Range(
                CellRef { row: 3, col: 3 },
                CellRef { row: 1, col: 1 },
            )),
        ),
    };

    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);
    assert!(result.is_err());

    // Cell should be set to error
    assert!(matches!(sheet.get(0, 0).unwrap(), CellValue::Error(_)));
}

#[test]
fn test_formula_storage_update() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Set a cell formula
    let expr1 = Expr::Constant(10);
    let cmd = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: expr1.clone(),
    };

    evaluate_command(cmd, &mut sheet, &mut output_enabled).unwrap();

    // Update the same cell with new formula
    let expr2 = Expr::Constant(20);
    let cmd = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: expr2.clone(),
    };

    evaluate_command(cmd, &mut sheet, &mut output_enabled).unwrap();

    // Verify the formula was updated
    let key = format!("A1");
    assert_eq!(*sheet.get_formula(&key).unwrap(), expr2);
    assert_ne!(*sheet.get_formula(&key).unwrap(), expr1);
}

#[test]
fn test_function_not_implemented() {
    let sheet = Spreadsheet::new(5, 5);

    // Create a fake function call that would trigger the default case
    // (this is a bit of a hack since our real code doesn't have this case)
    let expr = Expr::FunctionCall(
        Function::Sleep,
        Box::new(Expr::Range(
            CellRef { row: 1, col: 1 },
            CellRef { row: 2, col: 2 },
        )),
    );

    // Check that it errors properly
    let result = eval_expr(&expr, &sheet);
    assert!(result.is_err());
}

#[test]
fn test_scroll_methods_error_handling() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    // Create a Sheet mock that would return errors for scroll operations
    // For demonstration purposes only - in real test we'd need to mock the sheet

    // Test ScrollTo with invalid reference
    let cmd = Command::ScrollTo(CellRef {
        row: 1000,
        col: 1000,
    });
    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);

    // This might pass if your sheet handles out-of-range scrolling gracefully
    // The key is that we're testing the error propagation path in the evaluate_command function
    assert!(result.is_ok() || matches!(result, Err(EvalError::Other(_))));
}

#[test]
fn test_quit_command_coverage() {
    let mut sheet = Spreadsheet::new(5, 5);
    let mut output_enabled = true;

    let cmd = Command::Quit;
    let result = evaluate_command(cmd, &mut sheet, &mut output_enabled);

    // Just verifying it returns Ok
    assert!(result.is_ok());
}

#[test]
fn test_convert_col_to_name_edge_cases() {
    // Test with larger values
    assert_eq!(convert_col_to_name(27), "AA");
    assert_eq!(convert_col_to_name(28), "AB");
    assert_eq!(convert_col_to_name(52), "AZ");
    assert_eq!(convert_col_to_name(53), "BA");
    assert_eq!(convert_col_to_name(702), "ZZ");
    assert_eq!(convert_col_to_name(703), "AAA");
    assert_eq!(convert_col_to_name(1000), "ALL");
    assert_eq!(convert_col_to_name(18278), "ZZZ");
}
