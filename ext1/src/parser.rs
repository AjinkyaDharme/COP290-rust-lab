use crate::sheet::{Color, Condition};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_till},
    character::complete::{alpha1, char, digit1, multispace0},
    combinator::{all_consuming, map, map_res, opt, peek},
    error::{ParseError, VerboseError},
    multi::{fold_many0, separated_list0},
    sequence::{delimited, preceded, tuple},
};

use crate::command::{BinaryOp, CellRef, Command, Expr, Function};

fn parse_private(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("private")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, cell) = parse_cell_ref(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, Command::Private(cell)))
}

fn parse_gui(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag_no_case("gui")(input)?;
    Ok((input, Command::Gui))
}

pub fn parse_command(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    all_consuming(alt((
        parse_quit,
        parse_gui,
        parse_disable_output,
        parse_enable_output,
        parse_scroll_to,
        parse_scroll_single,
        parse_private,
        parse_format_command,
        parse_looping,
        parse_formula_command,
        parse_plot,
        parse_clear_format_where_command,
        parse_clear_format_command,
        parse_if_else_command,
        parse_input,
        parse_flight,
        parse_bar,
        parse_output,
    )))(input)
}

pub fn parse_bar(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("Bar")(input)?;
    let (input, _) = peek(tag("("))(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, arg) = alt((parse_range, parse_expr))(input)?;
    let (input, _) = tag(")")(input)?;
    let expr = match arg {
        Expr::Range(start, end) => Expr::Range(start, end),
        _ => {
            return Err(nom::Err::Failure(VerboseError::from_error_kind(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };
    Ok((input, Command::Bar(expr)))
}

pub fn parse_output(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("Output")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("(")(input)?;
    // Allow any characters until ')' (and trim whitespace)
    let (input, file) = map_res(take_till(|c| c == ')'), |s: &str| {
        Ok::<String, VerboseError<&str>>(s.trim().to_string())
    })(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, Command::Output(file)))
}

pub fn parse_flight(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("flight")(input)?;
    let (input, _) = multispace0(input)?;
    // Consume the outer opening parenthesis.
    let (input, _) = char('(')(input)?;
    // Parse the entire itinerary string with nested parentheses.
    let (input, itinerary_str) = parse_itinerary_string(input)?;
    // Consume the final closing parenthesis.
    let (input, _) = char(')')(input)?;
    Ok((input, Command::Flight(itinerary_str.to_string())))
}

fn parse_itinerary_string(input: &str) -> IResult<&str, &str, VerboseError<&str>> {
    let mut depth = 1;
    for (idx, ch) in input.char_indices() {
        if ch == '(' {
            depth += 1;
        } else if ch == ')' {
            depth -= 1;
            if depth == 0 {
                // Return substring from the start until here (not including this matching ')').
                return Ok((&input[idx..], &input[..idx]));
            }
        }
    }
    Err(nom::Err::Error(VerboseError::from_error_kind(
        input,
        nom::error::ErrorKind::Tag,
    )))
}

fn parse_if_else_command(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("if")(input)?;
    let (input, _) = multispace0(input)?;

    // Parse the cell reference for the condition
    let (input, cell) = parse_cell_ref(input)?;

    // Parse the comparison operator and value
    let (input, op) = alt((tag("<"), tag(">"), tag("=")))(input)?;
    let (input, value) = parse_integer(input)?;

    // Create condition as an Expr
    let condition = match op {
        "<" => Expr::BinaryOp(
            Box::new(Expr::CellRef(cell)),
            BinaryOp::LessThan,
            Box::new(Expr::Constant(value)),
        ),
        ">" => Expr::BinaryOp(
            Box::new(Expr::CellRef(cell)),
            BinaryOp::GreaterThan,
            Box::new(Expr::Constant(value)),
        ),
        "=" => Expr::BinaryOp(
            Box::new(Expr::CellRef(cell)),
            BinaryOp::Equal,
            Box::new(Expr::Constant(value)),
        ),
        _ => unreachable!(),
    };

    let (input, _) = multispace0(input)?;
    let (input, _) = tag("then")(input)?;
    let (input, _) = multispace0(input)?;

    // Parse the 'then' branch formula
    let (input, then_cmd) = parse_formula_command(input)?;

    let (input, _) = multispace0(input)?;
    let (input, _) = tag("else")(input)?;
    let (input, _) = multispace0(input)?;

    // Parse the 'else' branch formula
    let (input, else_cmd) = parse_formula_command(input)?;

    Ok((
        input,
        Command::IfElse {
            condition,
            then_cmd: Box::new(then_cmd),
            else_cmd: Box::new(else_cmd),
        },
    ))
}
fn parse_cell_ref_looping(input: &str) -> IResult<&str, u16, VerboseError<&str>> {
    let (input, col_letters) = alpha1(input)?;
    // Extract only the column part (without the potential variable)
    let col_letters = if col_letters.len() > 1 && col_letters.chars().last().unwrap().is_lowercase()
    {
        &col_letters[..col_letters.len() - 1]
    } else {
        col_letters
    };
    let mut col: u16 = 0;
    for c in col_letters.chars() {
        if !c.is_ascii_alphabetic() {
            return Err(nom::Err::Failure(VerboseError::from_error_kind(
                input,
                nom::error::ErrorKind::Alpha,
            )));
        }
        col = col * 26 + ((c.to_ascii_uppercase() as u16) - ('A' as u16) + 1);
    }
    Ok((input, col))
}
fn parse_looping(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    // eg: :i in 1..10: Ai=Bi+1
    let (input, _) = tag(":")(input)?;
    let (input, _) = alpha1(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("in")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, start) = parse_integer(input)?;
    let (input, _) = tag("..")(input)?;
    let (input, end) = parse_integer(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, _) = multispace0(input)?;

    // Check for the loop variable in the cell reference part
    let (input, start_col) = parse_cell_ref_looping(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, end_col) = parse_cell_ref_looping(input)?;
    let (input, _) = multispace0(input)?;
    let (input, op) = alt((tag("+"), tag("-"), tag("*"), tag("/")))(input)?;
    let (input, _) = multispace0(input)?;
    let (input, value) = parse_integer(input)?;

    // Generate the list of commands based on the loop range
    let mut commands = Vec::new();
    for i in start..=end {
        // Create the appropriate expression based on the operator
        let expr = match op {
            "+" => Expr::BinaryOp(
                Box::new(Expr::CellRef(CellRef {
                    col: end_col,
                    row: i as u16,
                })),
                BinaryOp::Add,
                Box::new(Expr::Constant(value)),
            ),
            "-" => Expr::BinaryOp(
                Box::new(Expr::CellRef(CellRef {
                    col: end_col,
                    row: i as u16,
                })),
                BinaryOp::Subtract,
                Box::new(Expr::Constant(value)),
            ),
            "*" => Expr::BinaryOp(
                Box::new(Expr::CellRef(CellRef {
                    col: end_col,
                    row: i as u16,
                })),
                BinaryOp::Multiply,
                Box::new(Expr::Constant(value)),
            ),
            "/" => Expr::BinaryOp(
                Box::new(Expr::CellRef(CellRef {
                    col: end_col,
                    row: i as u16,
                })),
                BinaryOp::Divide,
                Box::new(Expr::Constant(value)),
            ),
            _ => unreachable!(),
        };

        commands.push(Command::SetCell {
            cell: CellRef {
                col: start_col,
                row: i as u16,
            },
            expr: expr.clone(),
        });
    }
    Ok((input, Command::LoopCommands { commands }))
}

pub fn parse_plot(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("Plot")(input)?;
    let (input, _) = peek(tag("("))(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, arg) = alt((parse_range, parse_expr))(input)?;
    let (input, _) = tag(")")(input)?;
    let expr = match arg {
        Expr::Range(start, end) => Expr::Range(start, end),
        _ => {
            return Err(nom::Err::Failure(VerboseError::from_error_kind(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };
    Ok((input, Command::Plot(expr)))
}

fn parse_input(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("Input")(input)?;
    let (input, _) = peek(tag("("))(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, cell) = parse_cell_ref(input)?;
    let (input, _) = tag(",")(input)?;
    // Allow any characters until ')' (and trim whitespace)
    let (input, file) = map_res(take_till(|c| c == ')'), |s: &str| {
        Ok::<String, VerboseError<&str>>(s.trim().to_string())
    })(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, Command::Input { cell, file }))
}

fn parse_clear_format_where_command(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("clear_format_where")(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, condition) = parse_condition(input)?;
    let (input, _) = tag(")")(input)?;

    Ok((input, Command::ClearFormatWhere { condition }))
}
fn parse_clear_format_command(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("clear_format")(input)?;
    Ok((input, Command::ClearFormat))
}
fn parse_format_command(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("format")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, color) = parse_color(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, condition) = parse_condition(input)?;
    let (input, _) = tag(")")(input)?;

    Ok((input, Command::Format { condition, color }))
}
fn parse_color(input: &str) -> IResult<&str, Color, VerboseError<&str>> {
    let (input, color_str) = alpha1(input)?;
    let color = match color_str.to_lowercase().as_str() {
        "red" => Color::Red,
        "green" => Color::Green,
        "blue" => Color::Blue,
        "yellow" => Color::Yellow,
        "cyan" => Color::Cyan,
        "magenta" => Color::Magenta,
        _ => {
            return Err(nom::Err::Failure(VerboseError::from_error_kind(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };
    Ok((input, color))
}
fn parse_condition(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    alt((
        parse_between_condition,
        parse_comparison_condition,
        map(tag("negative"), |_| Condition::Between(i32::MIN, 0)),
        map(tag("positive"), |_| Condition::Between(0, i32::MAX)),
    ))(input)
}

fn parse_between_condition(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    // This structure should handle things like -1<x<1
    let (input, min) = parse_integer(input)?;
    let (input, _) = tag("<")(input)?;
    let (input, _) = tag("x")(input)?;
    let (input, _) = tag("<")(input)?;
    let (input, max) = parse_integer(input)?;

    Ok((input, Condition::Between(min, max)))
}

fn parse_comparison_condition(input: &str) -> IResult<&str, Condition, VerboseError<&str>> {
    let (input, op) = alt((tag("<"), tag(">"), tag("=")))(input)?;
    let (input, value) = parse_integer(input)?;

    let condition = match op {
        "<" => Condition::LessThan(value),
        ">" => Condition::GreaterThan(value),
        "=" => Condition::Equal(value),
        _ => unreachable!(),
    };

    Ok((input, condition))
}

fn parse_integer(input: &str) -> IResult<&str, i32, VerboseError<&str>> {
    let (input, sign_opt) = opt(tag("-"))(input)?;
    let (input, digits) = digit1(input)?;

    let is_negative = sign_opt.is_some();
    let value = digits.parse::<i32>().map_err(|_| {
        nom::Err::Failure(VerboseError::from_error_kind(
            input,
            nom::error::ErrorKind::Digit,
        ))
    })?;

    Ok((input, if is_negative { -value } else { value }))
}

fn parse_quit(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    map(tag("q"), |_| Command::Quit)(input)
}

fn parse_disable_output(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    map(tag("disable_output"), |_| Command::DisableOutput)(input)
}

fn parse_enable_output(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    map(tag("enable_output"), |_| Command::EnableOutput)(input)
}

fn parse_scroll_to(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("scroll_to")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, cell) = parse_cell_ref(input)?;
    Ok((input, Command::ScrollTo(cell)))
}

fn parse_scroll_single(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, cmd) = alt((tag("w"), tag("a"), tag("s"), tag("d")))(input)?;
    let command = match cmd {
        "w" => Command::ScrollUp,
        "a" => Command::ScrollLeft,
        "s" => Command::ScrollDown,
        "d" => Command::ScrollRight,
        _ => unreachable!(),
    };
    Ok((input, command))
}

fn parse_formula_command(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, cell) = parse_cell_ref(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, expr) = parse_expr(input)?;
    Ok((input, Command::SetCell { cell, expr }))
}

pub fn parse_cell_ref(input: &str) -> IResult<&str, CellRef, VerboseError<&str>> {
    let (input, col_letters) = alpha1(input)?;
    let (input, row_digits) = digit1(input)?;
    let mut col: usize = 0;
    for c in col_letters.chars() {
        if !c.is_ascii_alphabetic() {
            return Err(nom::Err::Failure(VerboseError::from_error_kind(
                input,
                nom::error::ErrorKind::Alpha,
            )));
        }
        col = col * 26 + ((c.to_ascii_uppercase() as usize) - ('A' as usize) + 1);
    }
    let row: usize = row_digits.parse().map_err(|_| {
        nom::Err::Failure(VerboseError::from_error_kind(
            input,
            nom::error::ErrorKind::Digit,
        ))
    })?;
    if col > 65535 || row > 65535 {
        return Err(nom::Err::Failure(VerboseError::from_error_kind(
            input,
            nom::error::ErrorKind::Verify,
        )));
    }

    let col = col as u16;

    let row = row as u16;

    Ok((input, CellRef { col, row }))
}

// New: Parse a comma-separated argument list.
fn parse_argument_list(input: &str) -> IResult<&str, Vec<Expr>, VerboseError<&str>> {
    separated_list0(delimited(multispace0, tag(","), multispace0), parse_expr)(input)
}

/// Updated function call parser that supports one or more arguments.
fn parse_function_call(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, func_name) = alpha1(input)?;
    let (input, _) = peek(tag("("))(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, args) = parse_argument_list(input)?;
    let (input, _) = tag(")")(input)?;
    let func = match func_name.to_uppercase().as_str() {
        "MIN" => Function::Min,
        "MAX" => Function::Max,
        "AVG" => Function::Avg,
        "SUM" => Function::Sum,
        "STDEV" => Function::Stdev,
        "SLEEP" => Function::Sleep,
        "SQRT" => Function::Sqrt,
        "NTHROOT" => Function::NthRoot,
        "ABS" => Function::Abs,
        "CEIL" => Function::Ceil,
        "FLOOR" => Function::Floor,
        "SIN" => Function::Sin,
        "COS" => Function::Cos,
        "TAN" => Function::Tan,
        _ => {
            return Err(nom::Err::Failure(VerboseError::from_error_kind(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };
    Ok((input, Expr::FunctionCall(func, args)))
}

fn parse_constant(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, sign) = opt(tag("-"))(input)?;
    let (input, digit_str) = digit1(input)?;
    let number = digit_str.parse::<i32>().map_err(|_| {
        nom::Err::Failure(VerboseError::from_error_kind(
            input,
            nom::error::ErrorKind::Digit,
        ))
    })?;
    let value = if sign.is_some() { -number } else { number };
    Ok((input, Expr::Constant(value)))
}

fn parse_cell_ref_expr(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(parse_cell_ref, Expr::CellRef)(input)
}

fn parse_parenthesized_expr(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    delimited(tag("("), parse_expr, tag(")"))(input)
}

// New: Bitwise parser with lower precedence than add/sub.
fn parse_bitwise(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, init) = parse_add_sub(input)?;
    fold_many0(
        preceded(
            multispace0,
            tuple((alt((tag("&"), tag("^"), tag("|"))), parse_add_sub)),
        ),
        || init.clone(),
        |acc, (operator, next)| {
            let op = match operator {
                "&" => BinaryOp::BitAnd,
                "^" => BinaryOp::BitXor,
                "|" => BinaryOp::BitOr,
                _ => unreachable!(),
            };
            Expr::BinaryOp(Box::new(acc), op, Box::new(next))
        },
    )(input)
}

fn parse_expr(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    alt((parse_bitwise, parse_add_sub))(input)
}

fn parse_add_sub(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, init) = parse_mul_div(input)?;
    fold_many0(
        preceded(
            multispace0,
            tuple((alt((tag("+"), tag("-"))), parse_mul_div)),
        ),
        || init.clone(),
        |acc, (operator, next)| {
            let op = if operator == "+" {
                BinaryOp::Add
            } else {
                BinaryOp::Subtract
            };
            Expr::BinaryOp(Box::new(acc), op, Box::new(next))
        },
    )(input)
}

fn parse_mul_div(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, init) = parse_factor(input)?;
    fold_many0(
        preceded(
            multispace0,
            tuple((alt((tag("*"), tag("/"))), parse_factor)),
        ),
        || init.clone(),
        |acc, (operator, next)| {
            let op = if operator == "*" {
                BinaryOp::Multiply
            } else {
                BinaryOp::Divide
            };
            Expr::BinaryOp(Box::new(acc), op, Box::new(next))
        },
    )(input)
}

fn parse_factor(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    preceded(
        multispace0,
        alt((
            parse_function_call,
            parse_parenthesized_expr,
            parse_constant,
            parse_range, // Moved before parse_cell_ref_expr
            parse_cell_ref_expr,
        )),
    )(input)
}

fn parse_range(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, start) = parse_cell_ref(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, end) = parse_cell_ref(input)?;
    Ok((input, Expr::Range(start, end)))
}
