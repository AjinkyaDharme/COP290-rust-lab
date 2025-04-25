//! Command-line interface for the spreadsheet application.
//!
//! This module provides the entry point for the spreadsheet application, handling:
//! - Command-line argument parsing for spreadsheet dimensions
//! - User input processing and command execution
//! - Dependency tracking and recalculation of cells
//! - Display of the spreadsheet and execution results
//! - Timing of command execution

mod command;
mod evaluator;
mod parser;
mod recalculation;
mod sheet;

use crate::recalculation::RecalcManager;
use command::Command;
use evaluator::evaluate_command;
use sheet::Spreadsheet;
use std::env::args;
use std::io::{Write, stdin, stdout};
use std::process;
use std::time::Instant;

/// Main entry point for the spreadsheet application.
///
/// # Process Flow
/// 1. Parses and validates command line arguments for spreadsheet dimensions
/// 2. Initializes the spreadsheet with the specified dimensions
/// 3. Enters a loop to process user commands:
///    - Parses user input into commands
///    - Updates cell dependencies for cell modification commands
///    - Executes commands and performs necessary recalculations
///    - Displays results and timing information
///
/// # Command Line Arguments
/// - First argument: Number of rows (1-999)
/// - Second argument: Number of columns (1-18278)
///
/// # Exit Codes
/// - 1: Invalid command line arguments
fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        println!("Usage: {} <rows> <columns>", args[0]);
        println!("(Invalid Command)");
        process::exit(1);
    }

    // Parse dimensions
    let rows: usize = match args[1].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("(Invalid Command)");
            process::exit(1);
        }
    };
    let cols: usize = match args[2].parse() {
        Ok(n) => n,
        Err(_) => {
            println!("(Invalid Command)");
            process::exit(1);
        }
    };

    // Validate dimensions
    if !(1..=999).contains(&rows) || !(1..=18278).contains(&cols) {
        println!("(Invalid Command)");
        process::exit(1);
    }

    let start_time = Instant::now();
    let mut sheet = Spreadsheet::new(rows, cols);
    let mut output_enabled = true;

    sheet.display_spreadsheet(sheet.scroll_row, sheet.scroll_col);
    let elapsed = start_time.elapsed().as_secs_f64();
    print!("[{:.1}] (ok) > ", elapsed);
    stdout().flush().unwrap();

    let mut input = String::new();
    let mut recalc_manager = RecalcManager::new();

    // Main command processing loop
    loop {
        input.clear();
        stdin().read_line(&mut input).unwrap();

        let command_start = Instant::now();
        let input = input.trim();
        if input.is_empty() {
            continue;
        }

        // This variable will optionally hold the computed topological order.
        // For SetCell commands, we can obtain the order at dependency update time.
        let mut topo_order_option: Option<Vec<String>> = None;

        match parser::parse_command(input) {
            Ok((_rem, cmd)) => {
                if let Command::Quit = cmd {
                    break;
                }

                if let Command::SetCell { cell: _, expr: _ } = &cmd {
                    match recalc_manager.update_for_command(&cmd) {
                        Ok(order) => {
                            topo_order_option = Some(order);
                            //I want to print the order here
                            // println!("Topological order: {:?}", topo_order_option);
                        }
                        Err(_err) => {
                            let elapsed = command_start.elapsed().as_secs_f64();
                            print!("[{:.1}] (Cycle detected) > ", elapsed);
                            stdout().flush().unwrap();
                            continue;
                        }
                    }
                }

                match evaluate_command(cmd, &mut sheet, &mut output_enabled) {
                    Ok(()) => {
                        if let Some(order) = topo_order_option {
                            recalculation::recalculate(&mut sheet, order);
                        }

                        if output_enabled {
                            sheet.display_spreadsheet(sheet.scroll_row, sheet.scroll_col);
                        }
                        let elapsed = command_start.elapsed().as_secs_f64();
                        print!("[{:.1}] (ok) > ", elapsed);
                        stdout().flush().unwrap();
                    }
                    Err(e) => {
                        let elapsed = command_start.elapsed().as_secs_f64();
                        print!("[{:.1}] ({}) > ", elapsed, e);
                        stdout().flush().unwrap();
                    }
                }
            }
            Err(e) => {
                let elapsed = command_start.elapsed().as_secs_f64();
                // Check if it's a Verify error from nom, which indicates cell reference out of bounds
                if let nom::Err::Failure(e) = &e {
                    if e.errors.iter().any(|(_, kind)| {
                        matches!(
                            kind,
                            nom::error::VerboseErrorKind::Nom(nom::error::ErrorKind::Verify)
                        )
                    }) {
                        print!("[{:.1}] (Cell reference out of bounds) > ", elapsed);
                    } else {
                        print!("[{:.1}] ({}) > ", elapsed, e);
                    }
                } else {
                    print!("[{:.1}] ({}) > ", elapsed, e);
                }
                stdout().flush().unwrap();
            }
        }
    }
}
