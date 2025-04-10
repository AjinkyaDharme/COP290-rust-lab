mod parser;
mod command;
mod sheet;
mod evaluator;
mod recalculation;

use std::io::{stdin, stdout, Write};
use std::time::Instant;
use command::Command;
use sheet::Spreadsheet;
use evaluator::{evaluate_command, EvalError};
use crate::recalculation::RecalcManager;
use memory_stats::memory_stats;

fn main() {
    let mut sheet = Spreadsheet::new(999, 18278);
    let mut output_enabled = true;
    
    
    let start_time = Instant::now();
    sheet.display_spreadsheet(sheet.scroll_row, sheet.scroll_col);
    let elapsed = start_time.elapsed().as_secs_f64();
    if let Some(usage) = memory_stats() {
        print!("[{:.1}s, {:.1}MB] (ok) > ", 
               elapsed, 
               usage.physical_mem as f64 / (1024.0 * 1024.0));
    }
    stdout().flush().unwrap();
    
    let mut input = String::new();
    let mut recalc_manager = RecalcManager::new();
    loop {
        input.clear();
        stdin().read_line(&mut input).unwrap();
        
        let command_start = Instant::now();     
        let input = input.trim();
        if input.is_empty() { continue; }

        match parser::parse_command(input) {
            Ok((_rem, cmd)) => {
                if let Command::Quit = cmd {
                    break;
                }
                if let Command::SetCell { .. } = cmd {
                    if let Err(err) = recalc_manager.update_for_command(&cmd) {
                        let elapsed = command_start.elapsed().as_secs_f64();
                        if let Some(usage) = memory_stats() {
                            print!("[{:.1}s, {:.1}MB] (Cycle detected) > ", 
                                   elapsed, 
                                   usage.physical_mem as f64 / (1024.0 * 1024.0));
                        }
                        stdout().flush().unwrap();
                        continue; 
                    }
                }
                match evaluator::evaluate_command(cmd, &mut sheet, &mut output_enabled) {
                    Ok(()) => {
                        if let Ok(order) = recalc_manager.topological_sort() {
                            recalculation::recalculate(&mut sheet, order);
                        }
                        if output_enabled {
                            sheet.display_spreadsheet(sheet.scroll_row, sheet.scroll_col);
                        }
                        let elapsed = command_start.elapsed().as_secs_f64();
                        if let Some(usage) = memory_stats() {
                            print!("[{:.1}s, {:.1}MB] (ok) > ", 
                                   elapsed, 
                                   usage.physical_mem as f64 / (1024.0 * 1024.0));
                        }
                        stdout().flush().unwrap();
                    }
                    Err(e) => {
                        let elapsed = command_start.elapsed().as_secs_f64();
                        if let Some(usage) = memory_stats() {
                            print!("[{:.1}s, {:.1}MB] ({}) > ", 
                                   elapsed, 
                                   usage.physical_mem as f64 / (1024.0 * 1024.0),e);
                        }
                        stdout().flush().unwrap();
                    }
                }
            }
            Err(e) => {
                
                let elapsed = command_start.elapsed().as_secs_f64();
                if let Some(usage) = memory_stats() {
                    print!("[{:.1}s, {:.1}MB] (parse error: {:?}) > ", 
                           elapsed, 
                           usage.physical_mem as f64 / (1024.0 * 1024.0),e);
                }
                stdout().flush().unwrap();
            }
        }
    }
}
