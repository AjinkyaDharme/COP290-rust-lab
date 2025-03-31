// mod spreadsheet;
mod parser;
mod graph;
use std::time::Instant;
use graph::update_expr;
// Global start time
static mut START_TIME: Option<Instant> = None;
// let current_expr:
//let old_expr: initally None in graph.rs
fn start_timer() {
    unsafe {
        START_TIME = Some(Instant::now());
    }
}
fn get_elapsed_time() -> f64 {
    unsafe {
        match START_TIME {
            Some(start) => start.elapsed().as_secs_f64(),
            None => 0.0,
        }
    }
}
use std::env::args;
use std::io::stdin;
use crate::parser::parse;
fn main() {
    start_timer();
    let args: Vec<String> = args().collect();
    if args.len()!=3{
        print!("invalid command");
        return;
    }
    let no_row = match args[1].parse::<i32>() {
        Ok(x) => x,
        Err(_) => {
            print!("invalid dimension");
            return;
        }
    };
    
    let no_col = match args[2].parse::<i32>() {
        Ok(x) => x,
        Err(_) => {
            print!("invalid dimension");
            return;
        }
    };
    if no_row<=0 || no_col<=0 || no_row>999 || no_col>18278{
        print!("invalid dimension");
        return;
    }

    // initialise the sheet and print it
    let elapsed_time = get_elapsed_time();
    println!("[{:.1}] (ok) > ", elapsed_time);
    let mut output_enabled=true;
    let mut command_error = false;
    loop {
        let mut user_input = String::new();
        
        if stdin().read_line(&mut user_input).is_err() {
            break;
        }
        start_timer();
        let user_input = user_input.trim();
        command_error = false;
        parse(user_input, &mut command_error);
        //while doing parsing update the old_expr to the old one which you are replacing
        //and current_expr to the new one
        if command_error{
            let elapsed_time = get_elapsed_time();
            if output_enabled{
                // displaySpreadsheet;
            }
            println!("[{:.1}] (Invalid command) > ", elapsed_time);
            continue;
        }
        //the parse function will store the command in expr

        let x=update_expr();
        if output_enabled {
            // displaySpreadsheet;
            // Placeholder for displaySpreadsheet function
        }
        if x==false{
            let elapsed_time = get_elapsed_time();
            println!("[{:.1}] cycle detected > ", elapsed_time);
            continue;
        }
        else{
            let elapsed_time = get_elapsed_time();
            println!("[{:.1}] (ok) > ", elapsed_time);
        }
        

    }
    // free the spreadsheet at the end
}

