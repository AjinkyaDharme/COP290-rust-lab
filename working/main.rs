mod parser;
mod command;
mod sheet;
mod evaluator;

use std::io::{stdin, stdout, Write};
use std::time::Instant;
use command::Command;
use sheet::Spreadsheet;
use evaluator::{evaluate_command, EvalError};

fn main() {
    let mut sheet = Spreadsheet::new(999, 18278);
    let mut output_enabled = true;
    
   
    let start_time = Instant::now();
    sheet.display_spreadsheet(sheet.scroll_row, sheet.scroll_col);
    let elapsed = start_time.elapsed().as_secs_f64();
    print!("[{:.1}] (ok) > ", elapsed);
    stdout().flush().unwrap();

    let mut input = String::new();
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
                match evaluator::evaluate_command(cmd, &mut sheet, &mut output_enabled) {
                    Ok(()) => {
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
                print!("[{:.1}] (parse error: {:?}) > ", elapsed, e);
                stdout().flush().unwrap();
            }
        }
    }
}
