// src/main.rs

mod command;
mod evaluator;
mod parser;
mod recalculation;
mod sheet;

use command::Command;
use evaluator::evaluate_command;
use parser::parse_command;
use recalculation::RecalcManager;
use sheet::Spreadsheet;

use rustyline::{Editor, Result as RustyResult, error::ReadlineError};
use std::io::{Write, stdout};
use std::process;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread;
use std::time::Instant;

// ---------- Helpers for both CLI & GUI ----------
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

fn color32_from_sheet_color(color: &sheet::Color) -> egui::Color32 {
    match color {
        sheet::Color::Red => egui::Color32::from_rgb(255, 16, 240),
        sheet::Color::Green => egui::Color32::from_rgb(57, 255, 20),
        sheet::Color::Blue => egui::Color32::from_rgb(0, 240, 255),
        sheet::Color::Yellow => egui::Color32::from_rgb(255, 255, 0),
        sheet::Color::Cyan => egui::Color32::from_rgb(40, 230, 230),
        sheet::Color::Magenta => egui::Color32::from_rgb(200, 40, 200),
    }
}

// Messages to tell the main thread to open/close the GUI
enum GuiMessage {
    Toggle,
    Quit,
}

// Thread‐local debug session
#[derive(Clone)]
struct DebugSession {
    updates: Vec<String>,
    pos: usize,
}

#[derive(Clone)]
struct SearchSession {
    term: String,
    matches: Vec<String>,
    pos: usize,
}

// Thread‐local undo/redo state
struct StateManager {
    undo_stack: Vec<Spreadsheet>,
    redo_stack: Vec<Spreadsheet>,
    capacity: usize,
}

impl StateManager {
    fn new(capacity: usize) -> Self {
        Self {
            undo_stack: Vec::with_capacity(capacity),
            redo_stack: Vec::with_capacity(capacity),
            capacity,
        }
    }

    fn save_state(&mut self, sheet: &Spreadsheet) {
        self.redo_stack.clear();
        if self.undo_stack.len() >= self.capacity {
            self.undo_stack.remove(0);
        }
        self.undo_stack.push(sheet.clone());
    }

    fn undo(&mut self, sheet: &mut Spreadsheet) -> bool {
        if let Some(prev) = self.undo_stack.pop() {
            self.redo_stack.push(sheet.clone());
            *sheet = prev;
            true
        } else {
            false
        }
    }

    fn redo(&mut self, sheet: &mut Spreadsheet) -> bool {
        if let Some(next) = self.redo_stack.pop() {
            if self.undo_stack.len() >= self.capacity {
                self.undo_stack.remove(0);
            }
            self.undo_stack.push(sheet.clone());
            *sheet = next;
            true
        } else {
            false
        }
    }
}

// The egui application
struct SpreadsheetApp {
    sheet: Arc<Mutex<Spreadsheet>>,
    zoom: f32,
    quit_flag: Arc<AtomicBool>,
    gui_close_requested: Arc<AtomicBool>,
    selected_cell: Option<(usize, usize)>,
    editing_expr: String,
    recalc_manager: Arc<Mutex<RecalcManager>>,
    precedent_cells: Vec<(usize, usize)>,
    dependent_cells: Vec<(usize, usize)>,
}

impl SpreadsheetApp {
    fn update_highlights(&mut self) {
        if let Some((row, col)) = self.selected_cell {
            let sheet = self.sheet.lock().unwrap();
            self.precedent_cells = (*sheet).get_all_precedents(row, col);
            self.dependent_cells = (*sheet).get_all_dependents(row, col);
        } else {
            self.precedent_cells.clear();
            self.dependent_cells.clear();
        }
    }
}

impl eframe::App for SpreadsheetApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if self.quit_flag.load(Ordering::Relaxed)
            || self.gui_close_requested.load(Ordering::Relaxed)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }

        // Style tweaks for neon theme
        {
            let mut style = (*ctx.style()).clone();
            style.visuals.panel_fill = egui::Color32::from_rgb(18, 20, 25);
            style.visuals.window_fill = egui::Color32::from_rgb(22, 24, 30);
            style.visuals.extreme_bg_color = egui::Color32::from_rgb(15, 17, 22);
            style.visuals.faint_bg_color = egui::Color32::from_rgb(25, 28, 35);
            style.visuals.override_text_color = Some(egui::Color32::from_rgb(240, 240, 255));
            style.visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(30, 34, 45);
            style.visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(35, 40, 55);
            style.visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(45, 50, 70);
            style.visuals.widgets.active.bg_fill = egui::Color32::from_rgb(0, 240, 255);
            style.visuals.widgets.active.fg_stroke.color = egui::Color32::from_rgb(0, 0, 0);
            style.visuals.selection.bg_fill = egui::Color32::from_rgb(255, 16, 240);
            style.visuals.selection.stroke.color = egui::Color32::from_rgb(255, 255, 255);
            for (_k, font_id) in style.text_styles.iter_mut() {
                font_id.size = 24.0;
            }
            ctx.set_style(style);
        }

        // Top controls panel
        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            let top_panel_bg = egui::Color32::from_rgb(30, 34, 45);
            let accent_color = egui::Color32::from_rgb(0, 240, 255);

            egui::Frame::none()
                .fill(top_panel_bg)
                .stroke(egui::Stroke::new(1.0, accent_color))
                .inner_margin(egui::style::Margin::same(4.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Zoom:");
                        ui.add(egui::Slider::new(&mut self.zoom, 0.5..=3.0).text("Zoom"));
                        ui.separator();
                        let mut sheet = self.sheet.lock().unwrap();
                        let mut sc = sheet.scroll_col as u32;
                        let mut sr = sheet.scroll_row as u32;
                        ui.label("H-Scroll:");
                        if ui.add(egui::DragValue::new(&mut sc).speed(1)).changed() {
                            sheet.scroll_col = sc as usize;
                        }
                        ui.label("V-Scroll:");
                        if ui.add(egui::DragValue::new(&mut sr).speed(1)).changed() {
                            sheet.scroll_row = sr as usize;
                        }
                    });
                });
        });

        ctx.set_pixels_per_point(self.zoom);

        // Main grid
        egui::CentralPanel::default().show(ctx, |ui| {
            let (sr, sc, er, ec, cols) = {
                let s = self.sheet.lock().unwrap();
                let sr = s.scroll_row;
                let sc = s.scroll_col;
                let er = (sr + 50).min(s.rows);
                let ec = (sc + 50).min(s.cols);
                (sr, sc, er, ec, s.cols)
            };

            egui::Grid::new("sheet_grid")
                .min_col_width(100.0)
                .striped(true)
                .show(ui, |ui| {
                    // Column headers
                    ui.monospace("");
                    for c in sc..ec {
                        ui.with_layout(
                            egui::Layout::centered_and_justified(egui::Direction::LeftToRight),
                            |ui| {
                                ui.monospace(Spreadsheet::convert_to_column_name(c as u16));
                            },
                        );
                    }
                    ui.end_row();

                    // Rows & cells
                    for r in sr..er {
                        ui.monospace(format!("{:>4}", r + 1));
                        for c in sc..ec {
                            let idx = r * cols + c;
                            let (raw, col_opt) = {
                                let s = self.sheet.lock().unwrap();
                                match s.cells.get(idx) {
                                    Some(sheet::CellValue::Value(v)) => {
                                        (v.to_string(), s.get_cell_color(*v))
                                    }
                                    Some(sheet::CellValue::Error(_)) => ("ERR".into(), None),
                                    _ => ("".into(), None),
                                }
                            };

                            let is_sel = self.selected_cell == Some((r, c));
                            let is_precedent = self.precedent_cells.contains(&(r, c));
                            let is_dependent = self.dependent_cells.contains(&(r, c));

                            let mut bg_color = None;
                            if is_sel {
                                bg_color = Some(egui::Color32::YELLOW);
                            } else if is_precedent {
                                bg_color = Some(egui::Color32::from_rgb(0, 80, 255));
                            } else if is_dependent {
                                bg_color = Some(egui::Color32::from_rgb(255, 0, 60)); // Electric red with slight magenta tone
                            }

                            if is_sel {
                                let resp = ui.add(
                                    egui::TextEdit::singleline(&mut self.editing_expr)
                                        .desired_width(90.0)
                                        .font(egui::TextStyle::Monospace),
                                );

                                if resp.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    let key = format!(
                                        "{}{}",
                                        Spreadsheet::convert_to_column_name(c as u16),
                                        r + 1
                                    );
                                    if let Ok((_rem, cmd)) =
                                        parse_command(&format!("{}={}", key, self.editing_expr))
                                    {
                                        if let Command::SetCell { cell: _, ref expr } = cmd {
                                            if let Ok(mut rm) = self.recalc_manager.lock() {
                                                if let Ok(order) = rm.update_for_command(&cmd) {
                                                    if let Ok(mut s) = self.sheet.lock() {
                                                        s.set_formula(&key, expr.clone());
                                                        match evaluator::eval_expr(expr, &s) {
                                                            Ok(val) => {
                                                                let _ = s.set(
                                                                    r,
                                                                    c,
                                                                    sheet::CellValue::Value(val),
                                                                );
                                                            }
                                                            Err(_) => {
                                                                let _ = s.set(
                                                                    r,
                                                                    c,
                                                                    sheet::CellValue::Error(()),
                                                                );
                                                            }
                                                        }
                                                        crate::recalculation::recalculate(
                                                            &mut s, order, false,
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    self.selected_cell = None;
                                    self.editing_expr.clear();
                                    self.precedent_cells.clear();
                                    self.dependent_cells.clear();
                                }
                            } else {
                                let text = format_cell(&raw);
                                let mut label = egui::RichText::new(text).monospace();
                                if let Some(col) = col_opt {
                                    label = label.color(color32_from_sheet_color(&col));
                                }
                                if let Some(bg) = bg_color {
                                    label = label.background_color(bg);
                                }
                                if ui.selectable_label(false, label).clicked() {
                                    self.selected_cell = Some((r, c));
                                    self.editing_expr = raw.clone();
                                    self.update_highlights();
                                }
                            }
                        }
                        ui.end_row();
                    }
                });

            // Bottom editor if a cell is selected
            if let Some((r, c)) = self.selected_cell {
                ui.separator();
                let id = format!("{}{}", Spreadsheet::convert_to_column_name(c as u16), r + 1);
                ui.horizontal(|ui| {
                    ui.label(format!("Editing {}:", id));
                    let edit = ui.text_edit_singleline(&mut self.editing_expr);
                    let apply = (edit.lost_focus()
                        && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        || ui.button("Apply").clicked();
                    if apply {
                        let cmd_str = format!("{}={}", id, self.editing_expr.trim());
                        if let Ok((_rem, cmd)) = parse_command(&cmd_str) {
                            if let Command::SetCell { cell: _, ref expr } = cmd {
                                if let Ok(mut rm) = self.recalc_manager.lock() {
                                    if let Ok(order) = rm.update_for_command(&cmd) {
                                        if let Ok(mut s) = self.sheet.lock() {
                                            s.set_formula(&id, expr.clone());
                                            match evaluator::eval_expr(expr, &s) {
                                                Ok(val) => {
                                                    let _ =
                                                        s.set(r, c, sheet::CellValue::Value(val));
                                                }
                                                Err(_) => {
                                                    let _ =
                                                        s.set(r, c, sheet::CellValue::Error(()));
                                                }
                                            }
                                            crate::recalculation::recalculate(&mut s, order, false);
                                        }
                                    }
                                }
                            }
                        }
                        self.update_highlights();
                        self.selected_cell = None;
                        self.editing_expr.clear();
                        self.precedent_cells.clear();
                        self.dependent_cells.clear();
                    }
                });
            }
        });

        ctx.request_repaint();
    }
}

fn main() {
    // — parse CLI args —
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <rows> <columns>", args[0]);
        process::exit(1);
    }
    let rows: usize = args[1].parse().unwrap_or_else(|_| {
        eprintln!("Invalid rows");
        process::exit(1)
    });
    let cols: usize = args[2].parse().unwrap_or_else(|_| {
        eprintln!("Invalid cols");
        process::exit(1)
    });
    if (!(1..=999).contains(&rows)) || (!(1..=18278).contains(&cols)) {
        eprintln!("Rows must be 1–999, cols 1–18278");
        process::exit(1);
    }

    // — shared app state for both CLI & GUI —
    let sheet = Arc::new(Mutex::new(Spreadsheet::new(rows, cols)));
    let recalc_mgr = Arc::new(Mutex::new(RecalcManager::new()));
    let quit_flag = Arc::new(AtomicBool::new(false));
    let gui_close_req = Arc::new(AtomicBool::new(false));

    // — channel to toggle the GUI from CLI thread —
    let (gui_tx, gui_rx) = mpsc::channel::<GuiMessage>();

    // — spawn the terminal thread —
    let terminal_handle = {
        let sheet_t = Arc::clone(&sheet);
        let recalc_t = Arc::clone(&recalc_mgr);
        let quit_t = Arc::clone(&quit_flag);
        let gui_tx_t = gui_tx.clone();

        thread::spawn(move || -> RustyResult<()> {
            let mut rl = Editor::<()>::new()?;
            let mut state_mgr = StateManager::new(5);
            let mut debug_session = None::<DebugSession>;
            let mut search_session = None::<SearchSession>;
            let mut debugger_mode = false;
            let mut output_enabled = true;

            // initial display
            {
                let s = sheet_t.lock().unwrap();
                s.display_spreadsheet(s.scroll_row, s.scroll_col);
                print!("[0.0] (ok) > ");
                stdout().flush().unwrap();
            }
            let mut prompt = "[0.0] (ok) > ".to_string();

            loop {
                let readline = rl.readline(&prompt);
                let elapsed = Instant::now();
                match readline {
                    Ok(line) => {
                        let input = line.trim();
                        // Only add non-empty, non-search commands to history
                        if !input.is_empty() && !input.starts_with('/') {
                            rl.add_history_entry(input);
                        }

                        // — step through an active debug session? —
                        if let Some(sess) = &mut debug_session {
                            if input.eq_ignore_ascii_case("end") {
                                println!("Debug session ended.");
                                debug_session = None;
                                let s = sheet_t.lock().unwrap();
                                s.display_spreadsheet(s.scroll_row, s.scroll_col);
                                prompt =
                                    format!("[{:.1}] (ok) > ", elapsed.elapsed().as_secs_f64());
                                continue;
                            }
                            if input.eq_ignore_ascii_case("n") {
                                // show next update
                                println!("{}", sess.updates[sess.pos]);
                                sess.pos += 1;
                                if sess.pos >= sess.updates.len() {
                                    println!("Debug session ended.");
                                    debug_session = None;
                                    let s = sheet_t.lock().unwrap();
                                    s.display_spreadsheet(s.scroll_row, s.scroll_col);
                                }
                                prompt = if let Some(ss) = &debug_session {
                                    format!("Debug [{} / {}] > ", ss.pos + 1, ss.updates.len())
                                } else {
                                    format!("[{:.1}] (ok) > ", elapsed.elapsed().as_secs_f64())
                                };
                                continue;
                            }
                        }

                        // — navigate through search results? —
                        if let Some(sess) = &mut search_session {
                            if input.eq_ignore_ascii_case("end") {
                                println!("Search session ended.");
                                search_session = None;
                                prompt =
                                    format!("[{:.1}] (ok) > ", elapsed.elapsed().as_secs_f64());
                                continue;
                            }

                            if input.eq_ignore_ascii_case("p") {
                                // Navigate to next match (forward)
                                if sess.pos < sess.matches.len() - 1 {
                                    sess.pos += 1;
                                    println!(
                                        "[{}/{}]: {}",
                                        sess.pos + 1,
                                        sess.matches.len(),
                                        sess.matches[sess.pos]
                                    );
                                } else {
                                    println!("At last match.");
                                }
                                prompt = format!("Search [{}] > ", sess.term);
                                continue;
                            }

                            if input.eq_ignore_ascii_case("q") {
                                // Navigate to previous match (backward)
                                if sess.pos > 0 {
                                    sess.pos -= 1;
                                    println!(
                                        "[{}/{}]: {}",
                                        sess.pos + 1,
                                        sess.matches.len(),
                                        sess.matches[sess.pos]
                                    );
                                } else {
                                    println!("At first match.");
                                }
                                prompt = format!("Search [{}] > ", sess.term);
                                continue;
                            }
                        }

                        // — start a search session? —
                        if input.starts_with("/") {
                            let term = input.trim_start_matches("/").trim();
                            if !term.is_empty() {
                                // Get history and find matches
                                let history: Vec<String> =
                                    rl.history().iter().map(|s| s.to_string()).collect();
                                let matches: Vec<String> = history
                                    .into_iter()
                                    .filter(|cmd| cmd.contains(term))
                                    .collect();

                                if matches.is_empty() {
                                    println!("No matches found for '{}'", term);
                                    prompt =
                                        format!("[{:.1}] (ok) > ", elapsed.elapsed().as_secs_f64());
                                } else {
                                    search_session = Some(SearchSession {
                                        term: term.to_string(),
                                        matches,
                                        pos: 0,
                                    });

                                    let sess = search_session.as_ref().unwrap();
                                    println!(
                                        "Found {} matches. Navigate with 'p' (prev) and 'q' (next), 'end' to exit.",
                                        sess.matches.len()
                                    );
                                    println!("[1/{}]: {}", sess.matches.len(), sess.matches[0]);
                                    prompt = format!("Search [{}] > ", term);
                                }
                                continue;
                            }
                        }

                        // — "debugger" arm for the next command —
                        if input.eq_ignore_ascii_case("debugger") {
                            debugger_mode = true;
                            println!("Entered debugger mode for next command.");
                            prompt = "[debugger] > ".into();
                            continue;
                        }

                        // — quit or gui toggle?
                        if input.eq_ignore_ascii_case("quit") || input.eq_ignore_ascii_case("q") {
                            quit_t.store(true, Ordering::Relaxed);
                            let _ = gui_tx_t.send(GuiMessage::Quit);
                            break;
                        }
                        if input.eq_ignore_ascii_case("gui") {
                            let _ = gui_tx_t.send(GuiMessage::Toggle);
                            continue;
                        }

                        // — undo / redo —
                        if input.eq_ignore_ascii_case("u") || input.eq_ignore_ascii_case("undo") {
                            let mut s = sheet_t.lock().unwrap();
                            let ok = state_mgr.undo(&mut s);
                            if output_enabled {
                                s.display_spreadsheet(s.scroll_row, s.scroll_col);
                            }
                            prompt = format!(
                                "[{:.1}] ({}) > ",
                                elapsed.elapsed().as_secs_f64(),
                                if ok { "undo" } else { "nothing to undo" }
                            );
                            continue;
                        }
                        if input.eq_ignore_ascii_case("y") || input.eq_ignore_ascii_case("redo") {
                            let mut s = sheet_t.lock().unwrap();
                            let ok = state_mgr.redo(&mut s);
                            if output_enabled {
                                s.display_spreadsheet(s.scroll_row, s.scroll_col);
                            }
                            prompt = format!(
                                "[{:.1}] ({}) > ",
                                elapsed.elapsed().as_secs_f64(),
                                if ok { "redo" } else { "nothing to redo" }
                            );
                            continue;
                        }

                        // — parse & execute a spreadsheet command —
                        match parse_command(input) {
                            Ok((_rem, cmd)) => {
                                if let Command::Quit = cmd {
                                    quit_t.store(true, Ordering::Relaxed);
                                    let _ = gui_tx_t.send(GuiMessage::Quit);
                                    break;
                                }
                                let mut topo_order = None;

                                // if it's a SetCell, save state & ask for a recalc order
                                if let Command::SetCell { .. } = &cmd {
                                    state_mgr.save_state(&sheet_t.lock().unwrap());
                                    match recalc_t.lock().unwrap().update_for_command(&cmd) {
                                        Ok(order) => topo_order = Some(order),
                                        Err(_) => {
                                            let s = sheet_t.lock().unwrap();
                                            s.display_spreadsheet(s.scroll_row, s.scroll_col);
                                            prompt = format!(
                                                "[{:.1}] (Cycle detected) > ",
                                                elapsed.elapsed().as_secs_f64()
                                            );
                                            continue;
                                        }
                                    }
                                }
                                if let Command::LoopCommands { .. } = &cmd {
                                    state_mgr.save_state(&sheet_t.lock().unwrap());
                                    match recalc_t.lock().unwrap().update_for_command(&cmd) {
                                        Ok(order) => topo_order = Some(order),
                                        Err(_) => {
                                            let s = sheet_t.lock().unwrap();
                                            s.display_spreadsheet(s.scroll_row, s.scroll_col);
                                            prompt = format!(
                                                "[{:.1}] (Cycle detected) > ",
                                                elapsed.elapsed().as_secs_f64()
                                            );
                                            continue;
                                        }
                                    }
                                }

                                // evaluate the command
                                let mut s = sheet_t.lock().unwrap();
                                match evaluate_command(cmd, &mut s, &mut output_enabled) {
                                    Ok(()) => {
                                        // if we have recalc order, do the recalc
                                        if let Some(order) = topo_order.take() {
                                            let updates = crate::recalculation::recalculate(
                                                &mut s,
                                                order,
                                                debugger_mode,
                                            );
                                            // if debugger_mode, this returns Some(updates)
                                            if let Some(u) = updates {
                                                debug_session =
                                                    Some(DebugSession { updates: u, pos: 0 });
                                                debugger_mode = false;
                                                println!(
                                                    "Debugger paused. Enter 'n' to step or 'end' to exit."
                                                );
                                                prompt = format!(
                                                    "Debug [1 / {}] > ",
                                                    debug_session.as_ref().unwrap().updates.len()
                                                );
                                                continue;
                                            }
                                        }
                                        if output_enabled {
                                            s.display_spreadsheet(s.scroll_row, s.scroll_col);
                                        }
                                        prompt = format!(
                                            "[{:.1}] (ok) > ",
                                            elapsed.elapsed().as_secs_f64()
                                        );
                                    }
                                    Err(e) => {
                                        prompt = format!(
                                            "[{:.1}] ({}) > ",
                                            elapsed.elapsed().as_secs_f64(),
                                            e
                                        );
                                    }
                                }
                            }
                            Err(e) => {
                                let elapsed_time = elapsed.elapsed().as_secs_f64();
                                // Check if it's a Verify error from nom, which indicates cell reference out of bounds
                                if let nom::Err::Failure(e) = &e {
                                    if e.errors.iter().any(|(_, kind)| {
                                        matches!(
                                            kind,
                                            nom::error::VerboseErrorKind::Nom(
                                                nom::error::ErrorKind::Verify
                                            )
                                        )
                                    }) {
                                        print!(
                                            "[{:.1}] (Cell reference out of bounds) > ",
                                            elapsed_time
                                        );
                                    } else {
                                        print!("[{:.1}] ({}) > ", elapsed_time, e);
                                    }
                                } else {
                                    print!("[{:.1}] ({}) > ", elapsed_time, e);
                                }
                                stdout().flush().unwrap();
                            }
                        }
                    }
                    Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => break,
                    Err(_) => break,
                }

                stdout().flush().unwrap();
            }

            Ok(())
        })
    };

    while !quit_flag.load(Ordering::Relaxed) {
        if let Ok(msg) = gui_rx.try_recv() {
            match msg {
                GuiMessage::Toggle => {
                    // launch the GUI
                    gui_close_req.store(false, Ordering::Relaxed);
                    let app = SpreadsheetApp {
                        sheet: Arc::clone(&sheet),
                        zoom: 1.0,
                        quit_flag: Arc::clone(&quit_flag),
                        gui_close_requested: Arc::clone(&gui_close_req),
                        selected_cell: None,
                        editing_expr: String::new(),
                        recalc_manager: Arc::clone(&recalc_mgr),
                        precedent_cells: Vec::new(),
                        dependent_cells: Vec::new(),
                    };
                    let mut options = eframe::NativeOptions::default();
                    options.viewport.fullscreen = Some(true);
                    eframe::run_native("Spreadsheet GUI", options, Box::new(|_cc| Box::new(app)))
                        .unwrap();
                }
                GuiMessage::Quit => {
                    quit_flag.store(true, Ordering::Relaxed);
                    gui_close_req.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
        thread::sleep(std::time::Duration::from_millis(50));
    }

    // — finally, wait for the terminal thread to finish cleanly —
    if let Err(e) = terminal_handle.join() {
        eprintln!("Terminal thread panicked: {:?}", e);
    }
}
