mod parser;
mod command;
mod sheet;
mod evaluator;
mod recalculation;

use std::env;
use std::process;
use std::io::{stdout, Write};
use std::sync::{Arc, Mutex, mpsc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Instant;
use command::Command;
use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use sheet::Spreadsheet;
use evaluator::evaluate_command;
use crate::recalculation::RecalcManager;

enum GuiMessage {
    Toggle,
    Quit,
}

type SharedFlag = Arc<AtomicBool>;

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

// --- COLOR MAPPING FOR GUI ---
fn color32_from_sheet_color(color: &sheet::Color) -> egui::Color32 {
    match color {
        sheet::Color::Red => egui::Color32::from_rgb(220, 44, 44),
        sheet::Color::Green => egui::Color32::from_rgb(44, 180, 44),
        sheet::Color::Blue => egui::Color32::from_rgb(66, 120, 255),
        sheet::Color::Yellow => egui::Color32::from_rgb(230, 230, 40),
        sheet::Color::Cyan => egui::Color32::from_rgb(40, 230, 230),
        sheet::Color::Magenta => egui::Color32::from_rgb(200, 40, 200),
    }
}

struct SpreadsheetApp {
    sheet: Arc<Mutex<Spreadsheet>>,
    zoom: f32,
    quit_flag: SharedFlag,
    gui_close_requested: SharedFlag,
    selected_cell: Option<(usize, usize)>,
    editing_expr: String,
    recalc_manager: Arc<Mutex<RecalcManager>>,
}

impl eframe::App for SpreadsheetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.quit_flag.load(Ordering::Relaxed) || self.gui_close_requested.load(Ordering::Relaxed) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        {
            let mut style = (*ctx.style()).clone();
            style.visuals.panel_fill = egui::Color32::from_rgb(255, 250, 205);
            style.visuals.window_fill = egui::Color32::from_rgb(255, 250, 205);
            style.visuals.extreme_bg_color = egui::Color32::from_rgb(255, 250, 205);
            style.visuals.faint_bg_color = egui::Color32::from_rgb(255, 250, 205);
            style.visuals.override_text_color = Some(egui::Color32::from_rgb(0, 0, 128));
            for (_k, font_id) in style.text_styles.iter_mut() {
                font_id.size = 24.0;
            }
            ctx.set_style(style);
        }

        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            // Lemonchiffon color
            let lemonchiffon = egui::Color32::from_rgb(255, 250, 205);
            egui::Frame::none()
                .fill(lemonchiffon)
                .inner_margin(egui::style::Margin::same(4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Zoom:");
                        ui.add(egui::Slider::new(&mut self.zoom, 0.5..=3.0).text("Zoom"));
                        ui.separator();
                        {
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
        });
        

        ctx.set_pixels_per_point(self.zoom);

        egui::CentralPanel::default().show(ctx, |ui| {
            // Draw grid and cells
            let (start_row, start_col, end_row, end_col, cols) = {
                let sheet_lock = self.sheet.lock().unwrap();
                let start_row = sheet_lock.scroll_row;
                let start_col = sheet_lock.scroll_col;
                let end_row = (start_row + 50).min(sheet_lock.rows);
                let end_col = (start_col + 50).min(sheet_lock.cols);
                let cols = sheet_lock.cols;
                (start_row, start_col, end_row, end_col, cols)
            };

            egui::Grid::new("spreadsheet_grid")
                .min_col_width(100.0)
                .striped(true)
                .show(ui, |ui| {
                    // Column headers
                    ui.monospace("");
                    for col in start_col..end_col {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.monospace(Spreadsheet::convert_to_column_name(col as u16));
                            },
                        );
                    }
                    ui.end_row();

                    // Rows and cells
                    for row in start_row..end_row {
                        ui.monospace(format!("{:>4}", row + 1));
                        for col in start_col..end_col {
                            let idx = row * cols + col;
                            let (raw_cell, cell_color) = {
                                let sheet = self.sheet.lock().unwrap();
                                match sheet.cells.get(idx) {
                                    Some(sheet::CellValue::Value(v)) => {
                                        (v.to_string(), sheet.get_cell_color(*v))
                                    }
                                    Some(sheet::CellValue::Error(_)) => ("ERR".to_string(), None),
                                    _ => ("".to_string(), None),
                                }
                            };
                            let is_selected = if let Some((s_row, s_col)) = self.selected_cell {
                                s_row == row && s_col == col
                            } else {
                                false
                            };

                            if is_selected {
                                let response = ui.add(
                                    egui::TextEdit::singleline(&mut self.editing_expr)
                                        .desired_width(90.0)
                                        .font(egui::TextStyle::Monospace),
                                );

                                if response.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    let cell_key = format!(
                                        "{}{}",
                                        Spreadsheet::convert_to_column_name(col as u16),
                                        row + 1
                                    );
                                    if let Ok((_rem, cmd)) = parser::parse_command(&format!(
                                        "{}={}",
                                        cell_key, self.editing_expr
                                    )) {
                                        match cmd {
                                            Command::SetCell { cell: _, expr: ref expr } => {
                                                if let Ok(mut recalc_manager) =
                                                    self.recalc_manager.lock()
                                                {
                                                    match recalc_manager.update_for_command(&cmd) {
                                                        Ok(order) => {
                                                            if let Ok(mut sheet) =
                                                                self.sheet.lock()
                                                            {
                                                                sheet.set_formula(
                                                                    &cell_key,
                                                                    expr.clone(),
                                                                );
                                                                match evaluator::eval_expr(
                                                                    &expr, &sheet,
                                                                ) {
                                                                    Ok(value) => {
                                                                        let _ = sheet.set(
                                                                            row,
                                                                            col,
                                                                            sheet::CellValue::Value(
                                                                                value,
                                                                            ),
                                                                        );
                                                                    }
                                                                    Err(_) => {
                                                                        let _ = sheet.set(
                                                                            row,
                                                                            col,
                                                                            sheet::CellValue::Error(
                                                                                (),
                                                                            ),
                                                                        );
                                                                    }
                                                                }
                                                                crate::recalculation::recalculate(
                                                                    &mut sheet, order,
                                                                );
                                                            }
                                                        }
                                                        Err(e) => {
                                                            eprintln!("Cycle detected: {}", e);
                                                        }
                                                    }
                                                }
                                            }
                                            _ => eprintln!("Unexpected command type for cell edit"),
                                        }
                                    } else {
                                        eprintln!("Formula parse error");
                                    }
                                    self.selected_cell = None;
                                    self.editing_expr.clear();
                                }
                            } else {
                                // --- COLORED LABEL LOGIC ---
                                let cell_text = format_cell(&raw_cell);
if let Some(color) = {
    // Only parse as i32 if not error/empty
    if let Ok(v) = raw_cell.parse::<i32>() {
        let sheet = self.sheet.lock().unwrap();
        sheet.get_cell_color(v)
    } else {
        None
    }
} {
    // Use RichText for colored, monospace label
    if ui
        .selectable_label(
            false,
            egui::RichText::new(cell_text)
                .color(color32_from_sheet_color(&color))
                .monospace(),
        )
        .clicked()
    {
        self.selected_cell = Some((row, col));
        self.editing_expr = raw_cell;
    }
} else {
    if ui
        .selectable_label(false, egui::RichText::new(cell_text).monospace())
        .clicked()
    {
        self.selected_cell = Some((row, col));
        self.editing_expr = raw_cell;
    }
}

                            }
                        }
                        ui.end_row();
                    }
                });

            // Cell editor below grid if a cell is selected
            if let Some((row, col)) = self.selected_cell {
                ui.separator();
                let cell_id =
                    format!("{}{}", Spreadsheet::convert_to_column_name(col as u16), row + 1);
                ui.horizontal(|ui| {
                    ui.label(format!("Editing cell {}:", cell_id));
                    let edit = ui.text_edit_singleline(&mut self.editing_expr);
                    if (edit.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        || ui.button("Apply").clicked()
                    {
                        match self.editing_expr.trim().parse::<i32>() {
                            Ok(new_value) => {
                                if let Ok(mut sheet) = self.sheet.lock() {
                                    let _ = sheet.set(row, col, sheet::CellValue::Value(new_value));
                                }
                            }
                            Err(_) => {
                                eprintln!("Invalid number format: '{}'", self.editing_expr);
                            }
                        }
                        self.selected_cell = None;
                        self.editing_expr.clear();
                    }
                });
            }
        });

        ctx.request_repaint();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        println!("Usage: {} <rows> <columns>", args[0]);
        println!("(Invalid Command)");
        process::exit(1);
    }
    let rows: usize = args[1].parse().unwrap_or_else(|_| {
        println!("(Invalid Command)");
        process::exit(1);
    });
    let cols: usize = args[2].parse().unwrap_or_else(|_| {
        println!("(Invalid Command)");
        process::exit(1);
    });
    if rows < 1 || rows > 999 || cols < 1 || cols > 18278 {
        println!("(Invalid Command)");
        process::exit(1);
    }

    let sheet = Spreadsheet::new(rows, cols);
    let shared_sheet = Arc::new(Mutex::new(sheet));
    let recalc_manager = Arc::new(Mutex::new(RecalcManager::new()));

    let quit_flag = Arc::new(AtomicBool::new(false));
    let gui_close_requested = Arc::new(AtomicBool::new(false));

    let (gui_tx, gui_rx) = mpsc::channel::<GuiMessage>();

    let sheet_for_terminal = Arc::clone(&shared_sheet);
    let recalc_manager_terminal = Arc::clone(&recalc_manager);
    let quit_flag_terminal = Arc::clone(&quit_flag);
    let gui_tx_terminal = gui_tx.clone();
    let terminal_thread = thread::spawn(move || -> Result<()> {
        let start_time = Instant::now();
        {
            let sheet = sheet_for_terminal.lock().unwrap();
            sheet.display_spreadsheet(sheet.scroll_row, sheet.scroll_col);
        }
        let elapsed = start_time.elapsed().as_secs_f64();
        print!("[{:.1}] (ok) > ", elapsed);
        stdout().flush().unwrap();

        let mut rl = Editor::<()>::new()?;
        let mut prompt = format!("[{:.1}] (ok) > ", Instant::now().elapsed().as_secs_f64());

        let mut output_enabled = true;

        loop {
            let readline = rl.readline(&prompt);
            match readline {
                Ok(line) => {
                    if !line.trim().is_empty() {
                        rl.add_history_entry(line.clone());
                    }
                    let command_start = Instant::now();
                    let input_trimmed = line.trim();
                    if input_trimmed.is_empty() {
                        continue;
                    }

                    if input_trimmed.eq_ignore_ascii_case("q") || input_trimmed.eq_ignore_ascii_case("quit") {
                        quit_flag_terminal.store(true, Ordering::Relaxed);
                        let _ = gui_tx_terminal.send(GuiMessage::Quit);
                        break;
                    }
                    if input_trimmed.eq_ignore_ascii_case("gui") {
                        let _ = gui_tx_terminal.send(GuiMessage::Toggle);
                        continue;
                    }

                    match parser::parse_command(input_trimmed) {
                        Ok((_rem, cmd)) => {
                            if let Command::Quit = cmd {
                                quit_flag_terminal.store(true, Ordering::Relaxed);
                                let _ = gui_tx_terminal.send(GuiMessage::Quit);
                                break;
                            }
                            let mut topo_order_option: Option<Vec<String>> = None;
                            if let Command::SetCell { cell: _, expr: _ } = &cmd {
                                if let Ok(mut recalc_manager) = recalc_manager_terminal.lock() {
                                    match recalc_manager.update_for_command(&cmd) {
                                        Ok(order) => {
                                            topo_order_option = Some(order);
                                        }
                                        Err(_err) => {
                                            let elapsed = command_start.elapsed().as_secs_f64();
                                            print!("[{:.1}] (Cycle detected) > ", elapsed);
                                            println!("\x07");
                                            stdout().flush().unwrap();
                                            continue;
                                        }
                                    }
                                }
                            }
                            {
                                let mut sheet = sheet_for_terminal.lock().unwrap();
                                match evaluate_command(cmd, &mut sheet, &mut output_enabled) {
                                    Ok(()) => {
                                        if let Some(order) = topo_order_option {
                                            crate::recalculation::recalculate(&mut sheet, order);
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
                Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
                Err(_err) => break,
            }
        }
        Ok(())
    });

    let mut gui_running = false;
    while !quit_flag.load(Ordering::Relaxed) {
        if let Ok(msg) = gui_rx.try_recv() {
            match msg {
                GuiMessage::Toggle => {
                    if !gui_running {
                        gui_running = true;
                        gui_close_requested.store(false, Ordering::Relaxed);
                        let sheet_for_gui = Arc::clone(&shared_sheet);
                        let quit_flag_gui = Arc::clone(&quit_flag);
                        let gui_close_req = Arc::clone(&gui_close_requested);
                        let recalc_manager_gui = Arc::clone(&recalc_manager);
                        let app = SpreadsheetApp {
                            sheet: sheet_for_gui,
                            zoom: 1.0,
                            quit_flag: quit_flag_gui,
                            gui_close_requested: gui_close_req,
                            selected_cell: None,
                            editing_expr: String::new(),
                            recalc_manager: recalc_manager_gui,
                        };
                        eframe::run_native("Spreadsheet", eframe::NativeOptions::default(), Box::new(|_cc| Box::new(app)));
                        gui_running = false;
                    } else {
                        gui_close_requested.store(true, Ordering::Relaxed);
                    }
                },
                GuiMessage::Quit => {
                    quit_flag.store(true, Ordering::Relaxed);
                    if gui_running {
                        gui_close_requested.store(true, Ordering::Relaxed);
                    }
                    break;
                },
            }
        }
        thread::sleep(std::time::Duration::from_millis(100));
    }

    let _ = terminal_thread.join();
}
