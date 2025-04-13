mod parser;
mod command;
mod sheet;
mod evaluator;
mod recalculation;

use std::env;
use std::process;
use std::io::{stdin, stdout, Write};
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;
use command::Command;
use sheet::Spreadsheet;
use evaluator::{evaluate_command, EvalError};
use crate::recalculation::RecalcManager;
use memory_stats::memory_stats;

fn format_cell(cell_str: &str) -> String {
    if let Ok(n) = cell_str.parse::<f64>() {
        if n.abs() >= 1e10 {
            format!("{:>10.3e}", n)
        } else {
            format!("{:>10}", n)
        }
    } else {
        format!("{:>10}", cell_str)
    }
}

// --- GUI Application using eframe/egui ---
struct SpreadsheetApp {
    sheet: Arc<Mutex<Spreadsheet>>,
    zoom: f32,
    quit_flag: Arc<AtomicBool>,
}

impl eframe::App for SpreadsheetApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // If the quit flag is set, close the GUI.
        if self.quit_flag.load(Ordering::Relaxed) {
            frame.close();
            return;
        }

        // Set a yellow background and blue text.
        {
            let mut style = (*ctx.style()).clone();
            // Set the entire background to yellow
            style.visuals.panel_fill = egui::Color32::from_rgb(255, 250, 205); // LemonChiffon background
            style.visuals.window_fill = egui::Color32::from_rgb(255, 250, 205); // LemonChiffon window
            style.visuals.extreme_bg_color = egui::Color32::from_rgb(255, 250, 205); // LemonChiffon extreme bg
            style.visuals.faint_bg_color = egui::Color32::from_rgb(255, 250, 205); // LemonChiffon faint bg

            // Override text color to navy blue
            style.visuals.override_text_color = Some(egui::Color32::from_rgb(0, 0, 128));
            // Increase font sizes and use monospaced fonts
            for (_k, font_id) in style.text_styles.iter_mut() {
                font_id.size = 24.0;
            }
            ctx.set_style(style);
        }

        // Top panel: zoom and scroll controls.
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("Zoom:");
                ui.add(egui::Slider::new(&mut self.zoom, 0.5..=3.0).text("Zoom"));
                ui.separator();
                {
                    // Grab scroll offsets from the shared spreadsheet.
                    let mut sheet = self.sheet.lock().unwrap();
                    let mut scroll_col = sheet.scroll_col as u32;
                    let mut scroll_row = sheet.scroll_row as u32;
                    ui.label("H-Scroll:");
                    if ui.add(egui::DragValue::new(&mut scroll_col).speed(1)).changed() {
                        sheet.scroll_col = scroll_col as usize;
                    }
                    ui.label("V-Scroll:");
                    if ui.add(egui::DragValue::new(&mut scroll_row).speed(1)).changed() {
                        sheet.scroll_row = scroll_row as usize;
                    }
                }
            });
        });

        // Apply the zoom level.
        ctx.set_pixels_per_point(self.zoom);

        // Central panel: render a fixed 50×50 region.
        egui::CentralPanel::default().show(ctx, |ui| {
            let sheet = self.sheet.lock().unwrap();
            let start_row = sheet.scroll_row;
            let start_col = sheet.scroll_col;
            let end_row = (start_row + 50).min(sheet.rows);
            let end_col = (start_col + 50).min(sheet.cols);

            // Use a grid layout for fixed alignment.
            egui::Grid::new("spreadsheet_grid")
                .min_col_width(100.0)
                .striped(true)
                .show(ui, |ui| {
                    // First row: empty top-left, then center-justified column headers.
                    ui.monospace(""); // Top-left corner empty.
                    for col in start_col..end_col {
                        ui.with_layout(egui::Layout::centered_and_justified(egui::Direction::LeftToRight), |ui| {
                            ui.monospace(Spreadsheet::convert_to_column_name(col as u16));
                        });
                    }
                    ui.end_row();

                    // Render each row.
                    for row in start_row..end_row {
                        ui.monospace(format!("{:>4}", row + 1)); // Row header.
                        for col in start_col..end_col {
                            let idx = row * sheet.cols + col;
                            let raw_cell = match sheet.cells.get(idx) {
                                Some(sheet::CellValue::Value(v)) => v.to_string(),
                                Some(sheet::CellValue::Error(_)) => "ERR".to_string(),
                                _ => "".to_string(),
                            };
                            let cell_text = format_cell(&raw_cell);
                            ui.monospace(cell_text);
                        }
                        ui.end_row();
                    }
                });
        });
        ctx.request_repaint(); // Continuously update the GUI.
    }
}

fn main() {
    // Parse command-line arguments.
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <rows> <columns>", args[0]);
        println!("(Invalid Command)");
        process::exit(1);
    }

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

    // Validate dimensions.
    if rows < 1 || rows > 999 || cols < 1 || cols > 18278 {
        println!("(Invalid command)");
        process::exit(1);
    }

    // Create the spreadsheet and wrap it in an Arc<Mutex> for sharing.
    let sheet = Spreadsheet::new(rows, cols);
    let shared_sheet = Arc::new(Mutex::new(sheet));

    // Create a shared quit flag.
    let quit_flag = Arc::new(AtomicBool::new(false));

    // Spawn a thread for the terminal loop.
    let sheet_for_terminal = Arc::clone(&shared_sheet);
    let quit_flag_terminal = Arc::clone(&quit_flag);
    let terminal_thread = thread::spawn(move || {
        let start_time = Instant::now();
        {
            let sheet = sheet_for_terminal.lock().unwrap();
            sheet.display_spreadsheet(sheet.scroll_row, sheet.scroll_col);
        }
        let elapsed = start_time.elapsed().as_secs_f64();
        print!("[{:.1}] (ok) > ", elapsed);
        stdout().flush().unwrap();

        let mut input = String::new();
        let mut recalc_manager = RecalcManager::new();
        let mut output_enabled = true;

        loop {
            input.clear();
            if stdin().read_line(&mut input).is_err() {
                continue;
            }
            let command_start = Instant::now();
            let input = input.trim();
            if input.is_empty() {
                continue;
            }
            // If the terminal receives "q" or "Quit", set the flag and break.
            if input.eq_ignore_ascii_case("q") || input.eq_ignore_ascii_case("quit") {
                quit_flag_terminal.store(true, Ordering::Relaxed);
                break;
            }

            let mut topo_order_option: Option<Vec<String>> = None;

            match parser::parse_command(input) {
                Ok((_rem, cmd)) => {
                    if let Command::Quit = cmd {
                        quit_flag_terminal.store(true, Ordering::Relaxed);
                        break;
                    }

                    if let Command::SetCell { cell: _, expr: _ } = &cmd {
                        match recalc_manager.update_for_command(&cmd) {
                            Ok(order) => {
                                topo_order_option = Some(order);
                            }
                            Err(_err) => {
                                let elapsed = command_start.elapsed().as_secs_f64();
                                print!("[{:.1}] (Cycle detected) > ", elapsed);
                                stdout().flush().unwrap();
                                continue;
                            }
                        }
                    }

                    {
                        let mut sheet = sheet_for_terminal.lock().unwrap();
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
                            }
                            Err(e) => {
                                let elapsed = command_start.elapsed().as_secs_f64();
                                print!("[{:.1}] ({}) > ", elapsed, e);
                            }
                        }
                    }
                    stdout().flush().unwrap();
                }
                Err(e) => {
                    let elapsed = command_start.elapsed().as_secs_f64();
                    print!("[{:.1}] ({}) > ", elapsed, e);
                    stdout().flush().unwrap();
                }
            }
        }
    });

    // Configure native options for eframe.
    let native_options = eframe::NativeOptions::default();

    // Start the GUI on the main thread with initial zoom 1.0.
    let app = SpreadsheetApp {
        sheet: Arc::clone(&shared_sheet),
        zoom: 1.0,
        quit_flag: Arc::clone(&quit_flag),
    };
    eframe::run_native("Spreadsheet", native_options, Box::new(|_cc| Box::new(app)));

    // Optionally wait for the terminal thread to finish.
    let _ = terminal_thread.join();
}
