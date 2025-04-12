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

    // if let Some(usage) = memory_stats() {
    //     print!("[{:.1}s, {:.1}MB] (ok) > ", 
    //            elapsed, 
    //            usage.physical_mem as f64 / (1024.0 * 1024.0));
    // }
    stdout().flush().unwrap();
    
    let mut input = String::new();
    let mut recalc_manager = RecalcManager::new();
    loop {
        input.clear();
        stdin().read_line(&mut input).unwrap();
        
        let command_start = Instant::now();     
        let input = input.trim();
        if input.is_empty() { continue; }
        // let mut current_cell_key: Option<String> = None;

        match parser::parse_command(input) {
            Ok((_rem, cmd)) => {
                if let Command::Quit = cmd {
                    break;
                }
                if let Command::SetCell { cell, expr: _ } = &cmd {
                    // current_cell_key = Some(recalculation::cell_to_string(cell));
                    if let Err(err) = recalc_manager.update_for_command(&cmd) {
                        if output_enabled {
                            sheet.display_spreadsheet(sheet.scroll_row, sheet.scroll_col);
                        }
                        let elapsed = command_start.elapsed().as_secs_f64();
                        print!("[{:.1}] (Cycle detected) > ", elapsed);
                        // if let Some(usage) = memory_stats() {
                        //     print!("[{:.1}s, {:.1}MB] (Cycle detected) > ", 
                        //            elapsed, 
                        //            usage.physical_mem as f64 / (1024.0 * 1024.0));
                        // }
                        stdout().flush().unwrap();
                        continue; 
                    }
                }
                match evaluator::evaluate_command(cmd, &mut sheet, &mut output_enabled) {
                    Ok(()) => {
                        // if let Some(current) = &current_cell_key {
                        //     // Get the set of affected descendants.
                        //     let descendants = recalc_manager.descendants(current);
                        //     // Get the full topologically sorted order.
                        //     if let Ok(mut order) = recalc_manager.topological_sort() {
                        //         // Retain only the cells that are in the descendants set.
                        //         order.retain(|cell_key| descendants.contains(cell_key));
                        //         recalculation::recalculate(&mut sheet, order);
                        //     }
                        // }
                        if let Ok(order) = recalc_manager.topological_sort() {
                            recalculation::recalculate(&mut sheet, order);
                        }
                        if output_enabled {
                            sheet.display_spreadsheet(sheet.scroll_row, sheet.scroll_col);
                        }
                        let elapsed = command_start.elapsed().as_secs_f64();
                        print!("[{:.1}] (ok) > ", elapsed);
                        // if let Some(usage) = memory_stats() {
                        //     print!("[{:.1}s, {:.1}MB] (ok) > ", 
                        //            elapsed, 
                        //            usage.physical_mem as f64 / (1024.0 * 1024.0));
                        // }
                        stdout().flush().unwrap();
                    }
                    Err(e) => {
                        let elapsed = command_start.elapsed().as_secs_f64();
                        print!("[{:.1}] ({}) > ", elapsed,e);
                        // if let Some(usage) = memory_stats() {
                        //     print!("[{:.1}s, {:.1}MB] ({}) > ", 
                        //            elapsed, 
                        //            usage.physical_mem as f64 / (1024.0 * 1024.0),e);
                        // }
                        stdout().flush().unwrap();
                    }
                }
            }
            Err(e) => {
                
                let elapsed = command_start.elapsed().as_secs_f64();
                print!("[{:.1}] ({}) > ", elapsed,e);
                // if let Some(usage) = memory_stats() {
                //     print!("[{:.1}s, {:.1}MB] (parse error: {:?}) > ", 
                //            elapsed, 
                //            usage.physical_mem as f64 / (1024.0 * 1024.0),e);
                // }
                stdout().flush().unwrap();
            }
        }
    }
}
