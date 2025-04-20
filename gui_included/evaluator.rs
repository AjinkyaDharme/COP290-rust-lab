use crate::command::{BinaryOp, CellRef, Command, Expr, Function};
use crate::recalculation;
use crate::sheet::CellFormat;
use crate::sheet::{CellValue, Spreadsheet};
use std::io::{Write, stdin, stdout};
use std::process;
use std::thread::sleep;
use std::time::Duration;
use plotters::prelude::*;
use std::io::Read;

#[derive(Debug)]
pub enum EvalError {
    DivByZero,
    CellError,
    OutOfBounds,
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
                BinaryOp::BitAnd => Ok(l & r),
                BinaryOp::BitXor => Ok(l ^ r),
                BinaryOp::BitOr => Ok(l | r),
            }
        }
        Expr::FunctionCall(func, args) => {
            match func {
                Function::Sleep => {
                    if args.len() != 1 {
                        return Err(EvalError::Other("Sleep requires one argument".into()));
                    }
                    let v = eval_expr(&args[0], sheet)?;
                    sleep(Duration::from_secs(v as u64));
                    Ok(v)
                }
                Function::Sqrt => {
                    if args.len() != 1 {
                        return Err(EvalError::Other("Sqrt requires one argument".into()));
                    }
                    let x = eval_expr(&args[0], sheet)? as f64;
                    if x < 0.0 {
                        return Err(EvalError::Other("Sqrt of negative".into()));
                    }
                    Ok(x.sqrt().round() as i32)
                }
                Function::NthRoot => {
                    if args.len() != 2 {
                        return Err(EvalError::Other("NthRoot requires two arguments".into()));
                    }
                    let x = eval_expr(&args[0], sheet)? as f64;
                    let n = eval_expr(&args[1], sheet)? as f64;
                    if n == 0.0 {
                        return Err(EvalError::Other("Root degree cannot be zero".into()));
                    }
                    if x < 0.0 && (n % 2.0 == 0.0) {
                        return Err(EvalError::Other("Even root of negative".into()));
                    }
                    Ok(x.abs().powf(1.0 / n).round() as i32)
                }
                Function::Abs => {
                    if args.len() != 1 {
                        return Err(EvalError::Other("Abs requires one argument".into()));
                    }
                    Ok(eval_expr(&args[0], sheet)?.abs())
                }
                Function::Ceil => {
                    if args.len() != 1 {
                        return Err(EvalError::Other("Ceil requires one argument".into()));
                    }
                    let x = eval_expr(&args[0], sheet)? as f64;
                    Ok(x.ceil() as i32)
                }
                Function::Floor => {
                    if args.len() != 1 {
                        return Err(EvalError::Other("Floor requires one argument".into()));
                    }
                    let x = eval_expr(&args[0], sheet)? as f64;
                    Ok(x.floor() as i32)
                }
                Function::Sin => {
                    if args.len() != 1 {
                        return Err(EvalError::Other("Sin requires one argument".into()));
                    }
                    let x = eval_expr(&args[0], sheet)? as f64;
                    Ok(x.to_radians().sin().round() as i32)
                }
                Function::Cos => {
                    if args.len() != 1 {
                        return Err(EvalError::Other("Cos requires one argument".into()));
                    }
                    let x = eval_expr(&args[0], sheet)? as f64;
                    Ok(x.to_radians().cos().round() as i32)
                }
                Function::Tan => {
                    if args.len() != 1 {
                        return Err(EvalError::Other("Tan requires one argument".into()));
                    }
                    let x = eval_expr(&args[0], sheet)? as f64;
                    Ok(x.to_radians().tan().round() as i32)
                }
                // Existing range functions.
                Function::Min | Function::Max | Function::Avg | Function::Sum | Function::Stdev => {
                    if args.len() != 1 {
                        return Err(EvalError::Other(format!(
                            "Function {:?} requires one argument",
                            func
                        )));
                    }
                    match &args[0] {
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
                                    Ok(values.iter().sum::<i32>() / (values.len() as i32))
                                }
                                Function::Stdev => {
                                    let n = values.len();
                                    if n <= 1 {
                                        return Ok(0);
                                    }
                                    let sum: i64 = values.iter().map(|&v| v as i64).sum();
                                    let sum_sq: i64 =
                                        values.iter().map(|&v| (v as i64) * (v as i64)).sum();
                                    let n_f64 = n as f64;
                                    let variance = ((n_f64 * sum_sq as f64)
                                        - (sum as f64 * sum as f64))
                                        / (n_f64 * n_f64);
                                    Ok(variance.sqrt().round() as i32)
                                }
                                _ => Err(EvalError::Other("Unknown range function".into())),
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

pub fn evaluate_command(
    cmd: Command,
    sheet: &mut Spreadsheet,
    output_enabled: &mut bool,
) -> Result<(), EvalError> {
    match cmd {
        Command::Private(cell) => {
            let key = recalculation::cell_to_string(&cell);
            sheet.mark_private(&key);
            println!("Cell {} marked as private.", key);
        }
        Command::SetCell { ref cell, ref expr } => {
            let key = recalculation::cell_to_string(cell);
            if sheet.is_private(&key) {
                for attempt in 0..3 {
                    print!("Enter password for {}: ", key);
                    stdout().flush().unwrap();
                    let mut pw = String::new();
                    stdin().read_line(&mut pw).unwrap();
                    if pw.trim() == "secret" {
                        break;
                    } else if attempt == 2 {
                        println!("Incorrect password. Exiting.");
                        process::exit(1);
                    }
                }
            }
            sheet.set_formula(&key, expr.clone());
            match eval_expr(expr, sheet) {
                Ok(v) => {
                    sheet
                        .set(
                            (cell.row as usize) - 1,
                            (cell.col as usize) - 1,
                            CellValue::Value(v),
                        )
                        .map_err(EvalError::Other)?;
                }
                Err(EvalError::DivByZero) | Err(EvalError::CellError) => {
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
        Command::Format { condition, color } => {
            let format = CellFormat { condition, color };
            sheet.add_format(format);
        }
        Command::ClearFormat => {
            sheet.clear_formats();
        }
        Command::ClearFormatWhere { condition } => {
            sheet.clear_formats_where(&condition);
        }
        Command::ScrollUp => sheet.scroll_spreadsheet('w').map_err(EvalError::Other)?,
        Command::ScrollDown => sheet.scroll_spreadsheet('s').map_err(EvalError::Other)?,
        Command::ScrollLeft => sheet.scroll_spreadsheet('a').map_err(EvalError::Other)?,
        Command::ScrollRight => sheet.scroll_spreadsheet('d').map_err(EvalError::Other)?,
        Command::ScrollTo(cell) => {
            let cell_str = format!("{}{}", convert_col_to_name(cell.col), cell.row);
            sheet.scroll_to(&cell_str).map_err(EvalError::Other)?;
        }
        Command::Plot(expr) => {
            match expr {
                Expr::Range(start, end) => {
                    let start_row = (start.row - 1) as usize;
                    let end_row   = (end.row - 1) as usize;
                    let start_col = (start.col - 1) as usize;
                    let end_col   = (end.col - 1) as usize;

                    println!("Plotting range: {} {} {} {}", start_row,start_col, end_row,end_col);
            
                    let mut points: Vec<(f32, f32)> = Vec::new();
            
                    // Vertical range with exactly 2 columns (for instance, A1:B5)
                    if end_col - start_col == 1 && end_row >= start_row {
                        for r in start_row..=end_row {
                            let x = match sheet.get(r, start_col) {
                                Some(CellValue::Value(v)) => *v as f32,
                                Some(CellValue::Error(_)) => return Err(EvalError::CellError),
                                None => return Err(EvalError::OutOfBounds),
                            };
                            let y = match sheet.get(r, start_col + 1) {
                                Some(CellValue::Value(v)) => *v as f32,
                                Some(CellValue::Error(_)) => return Err(EvalError::CellError),
                                None => return Err(EvalError::OutOfBounds),
                            };
                            points.push((x, y));
                        }
                    }
                    // Horizontal range with exactly 2 rows (for instance, A1:E2)
                    else if end_row - start_row == 1 && end_col >= start_col {
                        for c in start_col..=end_col {
                            let x = match sheet.get(start_row, c) {
                                Some(CellValue::Value(v)) => *v as f32,
                                Some(CellValue::Error(_)) => return Err(EvalError::CellError),
                                None => return Err(EvalError::OutOfBounds),
                            };
                            let y = match sheet.get(start_row + 1, c) {
                                Some(CellValue::Value(v)) => *v as f32,
                                Some(CellValue::Error(_)) => return Err(EvalError::CellError),
                                None => return Err(EvalError::OutOfBounds),
                            };
                            points.push((x, y));
                        }
                    } else {
                        return Err(EvalError::Other("Plot requires a range spanning exactly 2 rows or 2 columns".into()));
                    }
            
                    if points.len() < 2 {
                        return Err(EvalError::Other("Plot needs at least 2 points".into()));
                    }
                    println!("Plotting points: {:?}", points);
            
                    // Generate a detailed image of the plot via plotters.
                    generate_plot_image(&points)?;
                    println!("Plot image generated as 'plot.png'.");
                    return Ok(());
                }
                _ => return Err(EvalError::Other("Plot requires a range argument".into())),
            }
        }
        Command::DisableOutput => {
            *output_enabled = false;
        }
        Command::EnableOutput => {
            *output_enabled = true;
        }
        Command::Input{cell: cell, file: file} => {
            let key = recalculation::cell_to_string(&cell);
            sheet.set_formula(&key, Expr::Constant(0));
            
            let mut file = std::fs::File::open(file).map_err(|e| EvalError::Other(format!("Failed to open file: {}", e)))?;
            let mut contents = String::new();
            file.read_to_string(&mut contents).map_err(|e| EvalError::Other(format!("Failed to read file: {}", e)))?;
            
            let start_row = (cell.row as usize) - 1;
            let start_col = (cell.col as usize) - 1;
            for (row_offset, line) in contents.lines().enumerate() {
                for (col_offset, word) in line.split_whitespace().enumerate() {
                    let value = word.parse::<i32>().map_err(|_| EvalError::Other("Invalid number format".into()))?;
                    sheet.set(start_row + row_offset, start_col + col_offset, CellValue::Value(value))
                        .map_err(|e| EvalError::Other(e))?;
                }
            }
        },
        Command::Gui => (),
        Command::Quit => (),
    }
    Ok(())
}

fn convert_col_to_name(mut col: u16) -> String {
    let mut name = String::new();
    while col > 0 {
        let rem = ((col - 1) % 26) as u8;
        name.insert(0, (b'A' + rem) as char);
        col = (col - 1) / 26;
    }
    name
}
fn generate_plot_image(points: &[(f32, f32)]) -> Result<(), EvalError> {
    // Determine x and y ranges with some margins:
    let x_min = points.iter().map(|(x, _)| *x).fold(f32::INFINITY, f32::min);
    let x_max = points.iter().map(|(x, _)| *x).fold(f32::NEG_INFINITY, f32::max);
    let y_min = points.iter().map(|(_, y)| *y).fold(f32::INFINITY, f32::min);
    let y_max = points.iter().map(|(_, y)| *y).fold(f32::NEG_INFINITY, f32::max);

    // Build the output file path using CARGO_MANIFEST_DIR so that plot.png is alongside Cargo.toml.
    let project_dir = env!("CARGO_MANIFEST_DIR");
    let output_path = format!("{}/plot.png", project_dir);

    // Create a drawing area using BitMapBackend with the new output path.
    let backend = BitMapBackend::new(&output_path, (640, 480));
    let root_area = backend.into_drawing_area();

    // Fill background with white.
    root_area.fill(&WHITE)
        .map_err(|e| EvalError::Other(format!("Plotters fill error: {}", e)))?;

    // Build the chart with axis labels and margins.
    let mut chart = ChartBuilder::on(&root_area)
        .caption("Detailed Plot", ("sans-serif", 40))
        .margin(10)
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d((x_min - 1.0)..(x_max + 1.0), (y_min - 1.0)..(y_max + 1.0))
        .map_err(|e| EvalError::Other(format!("ChartBuilder error: {}", e)))?;

    // Configure mesh (axes, grid lines, labels)
    chart.configure_mesh()
        .x_desc("X-axis")
        .y_desc("Y-axis")
        .draw()
        .map_err(|e| EvalError::Other(format!("Mesh drawing error: {}", e)))?;

    // Draw the series as a line in RED.
    chart.draw_series(LineSeries::new(points.iter().cloned(), &RED))
        .map_err(|e| EvalError::Other(format!("LineSeries error: {}", e)))?;

    // Present (flush) the drawing area to file.
    root_area.present()
        .map_err(|e| EvalError::Other(format!("Backend present error: {}", e)))?;

    println!("Plot image successfully generated as '{}'.", output_path);

    Ok(())
}