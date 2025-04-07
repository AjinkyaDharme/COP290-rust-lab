mod parser;
mod command;
mod sheet;

use std::io::{stdin, stdout, Write};
use command::Command;
use sheet::Spreadsheet;

fn main() {

    let mut sheet = Spreadsheet::new(999, 18278);
    sheet.display_spreadsheet(0, 0);
    
    print!("[0.0] (ok) > ");
    stdout().flush().unwrap();
    let mut input = String::new();

    loop {
        input.clear();
        stdin().read_line(&mut input).expect("Failed to read line");
        let input = input.trim();
        if input.is_empty() {
            continue;
        }   
        // Using the nom-based parser
        match parser::parse_command(input) {
            Ok((_remaining, cmd)) => {
                println!("Parsed: {:?}", cmd);
                if let Command::Quit = cmd {
                    break;
                }
            }
            Err(e) => println!("Error: {:?}", e),
        }
        // Evaluate the command
        // set()

    }
}
