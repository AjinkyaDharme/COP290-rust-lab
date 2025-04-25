use std::thread::sleep;
use std::time::Duration;

use crate::command::{BinaryOp, CellRef, Command, Expr, Function};
use crate::recalculation;
use crate::sheet::{CellValue, Spreadsheet};
/// Represents possible errors that can occur during expression evaluation.
#[derive(Debug)]
pub enum EvalError {
    /// Division by zero error.
    DivByZero,
    /// Referenced cell contains an error.
    CellError,
    /// Cell reference is out of spreadsheet bounds.
    OutOfBounds,
    /// Catch-all for any other error, with an associated message.
    Other(String),
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::DivByZero => write!(f, "Division by zero"),
            EvalError::CellError => write!(f, "Referenced cell error"),
            EvalError::OutOfBounds => write!(f, "Cell reference out of bounds"),
            EvalError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

/// Evaluates an `Expr` expression in the context of the provided `Spreadsheet`.
///
/// Returns the computed value or an `EvalError` if evaluation fails.
pub fn eval_expr(expr: &Expr, sheet: &Spreadsheet) -> Result<i32, EvalError> {
    match expr {
        Expr::Constant(v) => Ok(*v),
        Expr::CellRef(CellRef { row, col }) => {
            let r = (*row as usize).saturating_sub(1);
            let c = (*col as usize).saturating_sub(1);
            match sheet.get(r, c) {
                Some(CellValue::Value(v)) => Ok(*v),
                Some(CellValue::Error(_)) => Err(EvalError::CellError),
                None => Err(EvalError::OutOfBounds),
            }
        }
        Expr::BinaryOp(lhs, op, rhs) => {
            let l = eval_expr(lhs, sheet)?;
            let r = eval_expr(rhs, sheet)?;
            match op {
                BinaryOp::Add => Ok(l + r),
                BinaryOp::Subtract => Ok(l - r),
                BinaryOp::Multiply => Ok(l * r),
                BinaryOp::Divide => {
                    if r == 0 {
                        Err(EvalError::DivByZero)
                    } else {
                        Ok(l / r)
                    }
                }
            }
        }
        Expr::FunctionCall(func, arg) => {
            match func {
                Function::Sleep => {
                    let v = eval_expr(arg, sheet)?;
                    sleep(Duration::from_secs(v as u64));
                    Ok(v)
                }

                Function::Sum | Function::Min | Function::Max | Function::Avg | Function::Stdev => {
                    match &**arg {
                        Expr::Range(start, end) => {
                            if start.row > end.row || start.col > end.col {
                                return Err(EvalError::Other("Invalid range".into()));
                            }

                            let start_row = (start.row - 1) as usize;
                            let end_row = (end.row - 1) as usize;
                            let start_col = (start.col - 1) as usize;
                            let end_col = (end.col - 1) as usize;
                            let mut values = Vec::new();
                            for r in start_row..=end_row {
                                for c in start_col..=end_col {
                                    match sheet.get(r, c) {
                                        Some(CellValue::Value(v)) => values.push(*v),
                                        Some(CellValue::Error(_)) => {
                                            return Err(EvalError::CellError);
                                        }
                                        None => return Err(EvalError::OutOfBounds),
                                    }
                                }
                            }
                            if values.is_empty() {
                                return Err(EvalError::Other("Empty range".into()));
                            }
                            match func {
                                Function::Sum => Ok(values.iter().sum()),
                                Function::Min => values
                                    .into_iter()
                                    .min()
                                    .ok_or_else(|| EvalError::Other("Empty range".into())),
                                Function::Max => values
                                    .into_iter()
                                    .max()
                                    .ok_or_else(|| EvalError::Other("Empty range".into())),
                                Function::Avg => {
                                    let sum: i32 = values.iter().sum();
                                    Ok(sum / (values.len() as i32))
                                }
                                Function::Stdev => {
                                    let n = values.len();
                                    if n <= 1 {
                                        return Ok(0); // Avoid division by zero
                                    }

                                    let sum: i64 = values.iter().map(|&v| v as i64).sum();
                                    let sum_of_squares: i64 =
                                        values.iter().map(|&v| (v as i64) * (v as i64)).sum();

                                    // Using the formula: sqrt((n * sum_of_squares - sum * sum) / n²)
                                    let n_f64 = n as f64;
                                    let variance = ((n_f64 * sum_of_squares as f64)
                                        - (sum as f64 * sum as f64))
                                        / (n_f64 * n_f64);
                                    let std_dev = variance.sqrt();

                                    // Round to nearest integer
                                    Ok(std_dev.round() as i32)
                                }
                                _ => Err(EvalError::Other(format!(
                                    "Function {:?} not implemented for range",
                                    func
                                ))),
                            }
                        }
                        _ => Err(EvalError::Other(format!(
                            "Function {:?} requires a range argument",
                            func
                        ))),
                    }
                }
            }
        }
        Expr::Range(_, _) => Err(EvalError::Other("Cannot evaluate a range by itself".into())),
    }
}

/// Evaluates and executes a `Command`, updating the spreadsheet or output state.
///
/// Returns an error if evaluation or execution fails.
pub fn evaluate_command(
    cmd: Command,
    sheet: &mut Spreadsheet,
    output_enabled: &mut bool,
) -> Result<(), EvalError> {
    match cmd {
        Command::SetCell { cell, expr } => {
            let key = recalculation::cell_to_string(&cell);
            sheet.set_formula(&key, expr.clone());

            match eval_expr(&expr, sheet) {
                Ok(v) => {
                    sheet
                        .set(
                            (cell.row as usize) - 1,
                            (cell.col as usize) - 1,
                            CellValue::Value(v),
                        )
                        .map_err(EvalError::Other)?;
                }
                Err(EvalError::DivByZero) => {
                    sheet
                        .set(
                            (cell.row as usize) - 1,
                            (cell.col as usize) - 1,
                            CellValue::Error(()),
                        )
                        .map_err(EvalError::Other)?;
                }
                Err(EvalError::CellError) => {
                    sheet
                        .set(
                            (cell.row as usize) - 1,
                            (cell.col as usize) - 1,
                            CellValue::Error(()),
                        )
                        .map_err(EvalError::Other)?;
                    return Ok(());
                }
                Err(e) => {
                    sheet
                        .set(
                            (cell.row as usize) - 1,
                            (cell.col as usize) - 1,
                            CellValue::Error(()),
                        )
                        .map_err(EvalError::Other)?;
                    return Err(e);
                }
            }
        }

        Command::ScrollUp => {
            sheet.scroll_spreadsheet('w').map_err(EvalError::Other)?;
        }
        Command::ScrollDown => {
            sheet.scroll_spreadsheet('s').map_err(EvalError::Other)?;
        }
        Command::ScrollLeft => {
            sheet.scroll_spreadsheet('a').map_err(EvalError::Other)?;
        }
        Command::ScrollRight => {
            sheet.scroll_spreadsheet('d').map_err(EvalError::Other)?;
        }
        Command::ScrollTo(cell) => {
            let cell_str = format!("{}{}", convert_col_to_name(cell.col), cell.row);
            sheet.scroll_to(&cell_str).map_err(EvalError::Other)?;
        }
        Command::DisableOutput => {
            *output_enabled = false;
        }
        Command::EnableOutput => {
            *output_enabled = true;
        }
        Command::Quit => {
            // handled in main loop.
        }
    }
    Ok(())
}

/// Converts a 1-based column number to its spreadsheet-style name (e.g., 1 → A, 27 → AA).
pub fn convert_col_to_name(mut col: u16) -> String {
    let mut name = String::new();
    while col > 0 {
        let rem = ((col - 1) % 26) as u8;
        name.insert(0, (b'A' + rem) as char);
        col = (col - 1) / 26;
    }
    name
}
