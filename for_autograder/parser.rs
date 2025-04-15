use nom::{
    IResult,
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, digit1, multispace0},
    combinator::opt,
    combinator::{all_consuming, map, peek},
    error::{ParseError, VerboseError},
    multi::fold_many0,
    sequence::{delimited, preceded, tuple},
};

use crate::command::{BinaryOp, CellRef, Command, Expr, Function};

pub fn parse_command(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    all_consuming(alt((
        parse_quit,
        parse_disable_output,
        parse_enable_output,
        parse_scroll_to,
        parse_scroll_single,
        parse_formula_command,
    )))(input)
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

fn parse_cell_ref(input: &str) -> IResult<&str, CellRef, VerboseError<&str>> {
    let (input, col_letters) = alpha1(input)?;
    let (input, row_digits) = digit1(input)?;

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
    let row: u16 = row_digits.parse().map_err(|_| {
        nom::Err::Failure(VerboseError::from_error_kind(
            input,
            nom::error::ErrorKind::Digit,
        ))
    })?;
    Ok((input, CellRef { col, row }))
}

fn parse_expr(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    parse_add_sub(input)
}

fn parse_add_sub(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, init) = parse_mul_div(input)?;
    fold_many0(
        preceded(
            multispace0,
            tuple((alt((tag("+"), tag("-"))), parse_mul_div)),
        ),
        || init.clone(),
        |acc, (op, val)| {
            let op = if op == "+" {
                BinaryOp::Add
            } else {
                BinaryOp::Subtract
            };
            Expr::BinaryOp(Box::new(acc), op, Box::new(val))
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
        |acc, (op, val)| {
            let op = if op == "*" {
                BinaryOp::Multiply
            } else {
                BinaryOp::Divide
            };
            Expr::BinaryOp(Box::new(acc), op, Box::new(val))
        },
    )(input)
}

/// Parses primary expressions: function calls, parenthesized expressions, constants, or cell references.
fn parse_factor(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    preceded(
        multispace0,
        alt((
            parse_function_call,
            parse_parenthesized_expr,
            parse_constant,
            parse_cell_ref_expr,
        )),
    )(input)
}

/// Parses a parenthesized expression.
fn parse_parenthesized_expr(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    delimited(tag("("), parse_expr, tag(")"))(input)
}

/// Parses a function call, e.g., MIN(expr)
fn parse_function_call(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, func_name) = alpha1(input)?;
    let (input, _) = peek(tag("("))(input)?;
    let (input, _) = tag("(")(input)?;
    let (input, arg) = alt((parse_range, parse_expr))(input)?;
    let (input, _) = tag(")")(input)?;
    let function = match func_name.to_uppercase().as_str() {
        "MIN" => Function::Min,
        "MAX" => Function::Max,
        "AVG" => Function::Avg,
        "SUM" => Function::Sum,
        "STDEV" => Function::Stdev,
        "SLEEP" => Function::Sleep,
        _ => {
            return Err(nom::Err::Failure(VerboseError::from_error_kind(
                input,
                nom::error::ErrorKind::Tag,
            )));
        }
    };
    Ok((input, Expr::FunctionCall(function, Box::new(arg))))
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

fn parse_range(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, start) = parse_cell_ref(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, end) = parse_cell_ref(input)?;
    Ok((input, Expr::Range(start, end)))
}
