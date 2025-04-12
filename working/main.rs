mod parser;
mod command;
mod sheet;
mod evaluator;
mod recalculation;

use std::env;
use std::process;
use std::io::{stdin, stdout, Write};
use std::time::Instant;
use command::Command;
use sheet::Spreadsheet;
use evaluator::{evaluate_command, EvalError};
use crate::recalculation::RecalcManager;
use memory_stats::memory_stats;

fn main() {
    let args: Vec<String> = env::args().collect();
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
    if rows < 1 || rows > 999 || cols < 1 || cols > 18278 {
        println!("(Invalid command)");
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
    
    loop {
        input.clear();
        stdin().read_line(&mut input).unwrap();
        
        let command_start = Instant::now();     
        let input = input.trim();
        if input.is_empty() { continue; }
        
        
        let mut topo_order_option: Option<Vec<String>> = None;
        
        match parser::parse_command(input) {
            Ok((_rem, cmd)) => {
                if let Command::Quit = cmd {
                    break;
                }
                
                
                if let Command::SetCell { cell, expr: _ } = &cmd {
                    
                    match recalc_manager.update_for_command(&cmd) {
                        Ok(order) => {
                            topo_order_option = Some(order);
                            
                        }
                        Err(err) => {
                            let elapsed = command_start.elapsed().as_secs_f64();
                            print!("[{:.1}] (Cycle detected) > ", elapsed);
                            stdout().flush().unwrap();
                            continue;
                        }
                        _ => {}
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
                print!("[{:.1}] ({}) > ", elapsed, e);
                stdout().flush().unwrap();
            }
        }
    }
}
