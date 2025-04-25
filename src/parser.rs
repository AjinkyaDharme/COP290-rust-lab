//! Parser module for the spreadsheet application.
//!
//! This module provides parsing functionality to convert user input into commands and expressions.
//! It uses the `nom` parsing library to implement a recursive descent parser for:
//! - Commands (like cell assignments, scrolling, etc.)
//! - Cell references (e.g., A1, B2)
//! - Expressions (constants, cell references, operations, function calls)
//! - Ranges (for aggregate functions)

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

/// Parses a user input string into a spreadsheet command.
///
/// This is the main entry point for the parser, which attempts to match the input
/// against all supported command patterns.
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and parsed `Command`, or an error
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

/// Parses the quit command ('q').
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and `Command::Quit`, or an error
pub fn parse_quit(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    map(tag("q"), |_| Command::Quit)(input)
}

/// Parses the disable output command.
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and `Command::DisableOutput`, or an error
pub fn parse_disable_output(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    map(tag("disable_output"), |_| Command::DisableOutput)(input)
}

/// Parses the enable output command.
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and `Command::EnableOutput`, or an error
pub fn parse_enable_output(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    map(tag("enable_output"), |_| Command::EnableOutput)(input)
}

/// Parses the scroll to command (e.g., "scroll_to A1").
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and `Command::ScrollTo`, or an error
pub fn parse_scroll_to(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, _) = tag("scroll_to")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, cell) = parse_cell_ref(input)?;
    Ok((input, Command::ScrollTo(cell)))
}

/// Parses single character scroll commands (w, a, s, d).
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and a scroll command, or an error
///
/// # Examples
/// * "w" -> `Command::ScrollUp`
/// * "a" -> `Command::ScrollLeft`
/// * "s" -> `Command::ScrollDown`
/// * "d" -> `Command::ScrollRight`
pub fn parse_scroll_single(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
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

/// Parses a formula command (cell assignment, e.g., "A1=5+B2").
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and a `Command::SetCell`, or an error
pub fn parse_formula_command(input: &str) -> IResult<&str, Command, VerboseError<&str>> {
    let (input, cell) = parse_cell_ref(input)?;
    let (input, _) = tag("=")(input)?;
    let (input, expr) = parse_expr(input)?;
    Ok((input, Command::SetCell { cell, expr }))
}

/// Parses a cell reference (e.g., "A1", "BC23").
///
/// Cell references consist of:
/// - One or more alphabetic characters for the column (A-Z, AA-ZZ, etc.)
/// - One or more numeric digits for the row
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and a `CellRef`, or an error
///
/// # Errors
/// Returns an error if the column or row is invalid or exceeds u16 bounds.
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

/// Parses an expression, which could be a constant, cell reference, or operation.
///
/// This is the main entry point for expression parsing.
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and an `Expr`, or an error
pub fn parse_expr(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    parse_add_sub(input)
}

/// Parses addition and subtraction expressions.
///
/// This handles the lowest precedence operations (+ and -).
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and an `Expr`, or an error
pub fn parse_add_sub(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
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

/// Parses multiplication and division expressions.
///
/// This handles the higher precedence operations (* and /).
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and an `Expr`, or an error
pub fn parse_mul_div(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
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
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and an `Expr`, or an error
pub fn parse_factor(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
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
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and an `Expr`, or an error
pub fn parse_parenthesized_expr(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    delimited(tag("("), parse_expr, tag(")"))(input)
}

/// Parses a function call (e.g., MIN(A1:B5), SUM(A1:A10), STDEV(1)).
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and a function call `Expr`, or an error
///
/// # Supported Functions
/// - MIN: Minimum value in a range
/// - MAX: Maximum value in a range
/// - AVG: Average of values in a range
/// - SUM: Sum of values in a range
/// - STDEV: Standard deviation of values in a range
/// - SLEEP: Pause execution for specified seconds
pub fn parse_function_call(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
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

/// Parses a constant integer value, possibly with a negative sign.
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and a constant `Expr`, or an error
pub fn parse_constant(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
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

/// Parses a cell reference as an expression.
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and a cell reference `Expr`, or an error
pub fn parse_cell_ref_expr(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    map(parse_cell_ref, Expr::CellRef)(input)
}

/// Parses a cell range (e.g., A1:B5).
///
/// # Arguments
/// * `input` - The string input to parse
///
/// # Returns
/// * A nom `IResult` containing either the remaining input and a range `Expr`, or an error
pub fn parse_range(input: &str) -> IResult<&str, Expr, VerboseError<&str>> {
    let (input, start) = parse_cell_ref(input)?;
    let (input, _) = tag(":")(input)?;
    let (input, end) = parse_cell_ref(input)?;
    Ok((input, Expr::Range(start, end)))
}
