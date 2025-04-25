//! Spreadsheet data structure and display module.
//!
//! This module defines the core spreadsheet data structure, including:
//! - Cell value representation
//! - Spreadsheet grid management
//! - Cell access and modification
//! - Formula storage
//! - Spreadsheet display and scrolling functionality

use std::collections::HashMap;

/// Represents a value that can be stored in a spreadsheet cell.
#[derive(Debug, Clone, PartialEq)]
pub enum CellValue {
    /// A valid integer value.
    Value(i32),
    /// An error state, represented as an empty tuple.
    Error(()),
}

impl Default for CellValue {
    fn default() -> Self {
        CellValue::Value(0)
    }
}

/// The main spreadsheet data structure.
///
/// Stores cell values, formulas, and view state for the spreadsheet.
pub struct Spreadsheet {
    /// Number of rows in the spreadsheet.
    pub rows: usize,
    /// Number of columns in the spreadsheet.
    pub cols: usize,
    /// Linear array of cell values, indexed as row * cols + column.
    pub cells: Vec<CellValue>,
    /// Map of cell references (e.g., "A1") to their formula expressions.
    pub formulas: HashMap<String, crate::command::Expr>,
    /// Current top row in the viewing window.
    pub scroll_row: usize,
    /// Current leftmost column in the viewing window.
    pub scroll_col: usize,
}

impl Spreadsheet {
    /// Creates a new spreadsheet with the specified dimensions.
    ///
    /// # Arguments
    /// * `rows` - Number of rows in the spreadsheet
    /// * `cols` - Number of columns in the spreadsheet
    ///
    /// # Returns
    /// A new spreadsheet with all cells initialized to zero
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = vec![CellValue::default(); rows * cols];
        Self {
            rows,
            cols,
            cells,
            formulas: HashMap::new(),
            scroll_row: 0,
            scroll_col: 0,
        }
    }

    /// Retrieves the formula expression for a given cell key.
    ///
    /// # Arguments
    /// * `key` - The cell reference as a string (e.g., "A1")
    ///
    /// # Returns
    /// * `Some(&Expr)` - The formula expression if the cell has one
    /// * `None` - If the cell has no formula
    pub fn get_formula(&self, key: &str) -> Option<&crate::command::Expr> {
        self.formulas.get(key)
    }

    /// Sets a formula expression for a given cell key.
    ///
    /// # Arguments
    /// * `key` - The cell reference as a string (e.g., "A1")
    /// * `expr` - The expression to store as the cell's formula
    pub fn set_formula(&mut self, key: &str, expr: crate::command::Expr) {
        self.formulas.insert(key.to_string(), expr);
    }

    /// Updates a cell's value based on its key.
    ///
    /// This function computes the index from the key (e.g., "A1") and updates that cell.
    ///
    /// # Arguments
    /// * `key` - The cell reference as a string (e.g., "A1")
    /// * `value` - The new value to set for the cell
    ///
    /// # Returns
    /// * `Ok(())` - If the cell was updated successfully
    /// * `Err(String)` - If the cell reference is invalid or out of bounds
    pub fn set_by_key(&mut self, key: &str, value: CellValue) -> Result<(), String> {
        let (row, col) = Self::cell_key_to_index(key)?;
        let idx = row * self.cols + col;
        if idx < self.cells.len() {
            self.cells[idx] = value;
            Ok(())
        } else {
            Err(format!("Cell {} is out of bounds", key))
        }
    }

    /// Helper: converts a cell key like "A1" to 0-based row and col indices.
    ///
    /// # Arguments
    /// * `key` - The cell reference as a string (e.g., "A1")
    ///
    /// # Returns
    /// * `Ok((row, col))` - The 0-based row and column indices
    /// * `Err(String)` - If the cell reference is invalid
    pub fn cell_key_to_index(key: &str) -> Result<(usize, usize), String> {
        // Split the key into the column letters and the row number.
        let mut col_part = String::new();
        let mut row_part = String::new();
        for ch in key.chars() {
            if ch.is_ascii_alphabetic() {
                col_part.push(ch);
            } else if ch.is_ascii_digit() {
                row_part.push(ch);
            } else {
                return Err(format!("Invalid character in cell key: {}", ch));
            }
        }
        if col_part.is_empty() || row_part.is_empty() {
            return Err(format!("Invalid cell key format: {}", key));
        }
        let mut col = 0;
        for ch in col_part.chars() {
            col = col * 26 + (ch.to_ascii_uppercase() as usize) - ('A' as usize) + 1;
        }
        let row: usize = row_part
            .parse::<usize>()
            .map_err(|_| format!("Invalid row in cell key: {}", key))?
            - 1;
        Ok((row, col - 1))
    }

    /// Calculates the linear index for a given row and column.
    ///
    /// # Arguments
    /// * `row` - The 0-based row index
    /// * `col` - The 0-based column index
    ///
    /// # Returns
    /// * `Some(usize)` - The linear index if within bounds
    /// * `None` - If the row or column is out of bounds
    fn index(&self, row: usize, col: usize) -> Option<usize> {
        if row < self.rows && col < self.cols {
            Some(row * self.cols + col)
        } else {
            None
        }
    }

    /// Gets the value at a specific cell.
    ///
    /// # Arguments
    /// * `row` - The 0-based row index
    /// * `col` - The 0-based column index
    ///
    /// # Returns
    /// * `Some(&CellValue)` - The cell value if the indices are valid
    /// * `None` - If the row or column is out of bounds
    pub fn get(&self, row: usize, col: usize) -> Option<&CellValue> {
        self.index(row, col).map(|idx| &self.cells[idx])
    }

    /// Sets the value at a specific cell.
    ///
    /// # Arguments
    /// * `row` - The 0-based row index
    /// * `col` - The 0-based column index
    /// * `value` - The new cell value to set
    ///
    /// # Returns
    /// * `Ok(())` - If the cell was updated successfully
    /// * `Err(String)` - If the row or column is out of bounds
    pub fn set(&mut self, row: usize, col: usize, value: CellValue) -> Result<(), String> {
        if let Some(idx) = self.index(row, col) {
            self.cells[idx] = value;
            Ok(())
        } else {
            Err("Index out of bounds".to_string())
        }
    }

    /// Displays a portion of the spreadsheet in the console.
    ///
    /// Shows a 10x10 window of cells starting from the given row and column.
    ///
    /// # Arguments
    /// * `start_row` - The 0-based starting row for display
    /// * `start_col` - The 0-based starting column for display
    pub fn display_spreadsheet(&self, start_row: usize, start_col: usize) {
        let end_row = (start_row + 10).min(self.rows);
        let end_col = (start_col + 10).min(self.cols);

        // Print column headers (A, B, C, ...)
        print!("\t");
        for col in start_col..end_col {
            print!("{}\t", Spreadsheet::convert_to_column_name(col as u16));
        }
        println!();

        // Print each row, starting with its 1-indexed row number.
        for row in start_row..end_row {
            print!("{}\t", row + 1);
            for col in start_col..end_col {
                let idx = row * self.cols + col;
                let cell_display = match self.cells.get(idx) {
                    Some(CellValue::Value(v)) => v.to_string(),
                    Some(CellValue::Error(())) => "ERR".to_string(),
                    None => "err".to_string(),
                };
                print!("{}\t", cell_display);
            }
            println!();
        }
    }

    /// Scrolls the spreadsheet view in the specified direction.
    ///
    /// # Arguments
    /// * `direction` - The direction to scroll:
    ///   - 'w': up
    ///   - 's': down
    ///   - 'a': left
    ///   - 'd': right
    ///
    /// # Returns
    /// * `Ok(())` - If scrolling was successful
    /// * `Err(String)` - If an invalid direction was provided
    pub fn scroll_spreadsheet(&mut self, direction: char) -> Result<(), String> {
        let display_rows = 10;
        let display_cols = 10;

        match direction {
            'w' => {
                if self.scroll_row >= display_rows {
                    self.scroll_row -= display_rows;
                } else {
                    self.scroll_row = 0;
                }
            }
            's' => {
                self.scroll_row =
                    (self.scroll_row + display_rows).min(self.rows.saturating_sub(display_rows));
            }
            'a' => {
                if self.scroll_col >= display_cols {
                    self.scroll_col -= display_cols;
                } else {
                    self.scroll_col = 0;
                }
            }
            'd' => {
                self.scroll_col =
                    (self.scroll_col + display_cols).min(self.cols.saturating_sub(display_cols));
            }
            _ => return Err("Invalid direction".to_string()),
        }
        Ok(())
    }

    /// Scrolls the view to center on a specific cell.
    ///
    /// # Arguments
    /// * `cell` - The cell reference as a string (e.g., "A1")
    ///
    /// # Returns
    /// * `Ok(())` - If scrolling was successful
    /// * `Err(String)` - If the cell reference is invalid or out of bounds
    pub fn scroll_to(&mut self, cell: &str) -> Result<(), String> {
        if let Some((row, col)) = Spreadsheet::cell_string_to_indices(cell) {
            if row < self.rows && col < self.cols {
                self.scroll_row = row;
                self.scroll_col = col;
                Ok(())
            } else {
                Err("Cell reference out of bounds".to_string())
            }
        } else {
            Err("Invalid cell reference format".to_string())
        }
    }

    /// Converts a cell reference string to 0-based row and column indices.
    ///
    /// # Arguments
    /// * `cell` - The cell reference as a string (e.g., "A1")
    ///
    /// # Returns
    /// * `Some((row, col))` - The 0-based row and column indices if the reference is valid
    /// * `None` - If the cell reference format is invalid
    pub fn cell_string_to_indices(cell: &str) -> Option<(usize, usize)> {
        if cell.is_empty() || !cell.chars().next()?.is_alphabetic() {
            return None;
        }

        let mut col = 0usize;
        let mut i = 0;
        let chars: Vec<char> = cell.chars().collect();

        while i < chars.len() && chars[i].is_alphabetic() {
            col = col * 26 + (chars[i].to_ascii_uppercase() as usize - 'A' as usize + 1);
            i += 1;
        }

        if col == 0 || i == chars.len() || !chars[i].is_ascii_digit() {
            return None;
        }

        let mut row = 0usize;
        while i < chars.len() && chars[i].is_ascii_digit() {
            row = row * 10 + (chars[i] as usize - '0' as usize);
            i += 1;
        }

        if i != chars.len() {
            return None;
        }

        Some((row - 1, col - 1))
    }

    /// Converts a 0-based column index to an Excel-style column name (A, B, ..., Z, AA, AB, ...).
    ///
    /// # Arguments
    /// * `col` - The 0-based column index to convert
    ///
    /// # Returns
    /// A string representing the column name
    pub fn convert_to_column_name(mut col: u16) -> String {
        let mut name = String::new();
        loop {
            let rem = col % 26;
            name.insert(0, (b'A' + rem as u8) as char);
            if col < 26 {
                break;
            }
            col = (col / 26) - 1;
        }
        name
    }
}
