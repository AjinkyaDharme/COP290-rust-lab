#[derive(Debug, Clone)]
pub enum CellValue {
    Value(i32),
    Error(String),
}

impl Default for CellValue {
    fn default() -> Self {
        CellValue::Value(0)
    }
}

pub struct Spreadsheet {
   pub rows: usize,
   pub cols: usize,
   pub cells: Vec<CellValue>,
   pub scroll_row: usize,
   pub scroll_col: usize,
}

impl Spreadsheet {
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = vec![CellValue::default(); rows * cols];
        Self { 
            rows, 
            cols, 
            cells,
            scroll_row: 0,
            scroll_col: 0, }
    }

    fn index(&self, row: usize, col: usize) -> Option<usize> {
        if row < self.rows && col < self.cols {
            Some(row * self.cols + col)
        } else {
            None
        }
    }

    pub fn get(&self, row: usize, col: usize) -> Option<&CellValue> {
        self.index(row, col).map(|idx| &self.cells[idx])
    }


    pub fn set(&mut self, row: usize, col: usize, value: CellValue) -> Result<(), String> {
        if let Some(idx) = self.index(row, col) {
            self.cells[idx] = value;
            Ok(())
        } else {
            Err("Index out of bounds".to_string())
        }
    }

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
                    Some(CellValue::Error(_)) => "ERR".to_string(),
                    None => "err".to_string(),
                };
                print!("{}\t", cell_display);
            }
            println!();
        }
    }

    pub fn scroll_spreadsheet(&mut self, direction: char) -> Result<(), String> {
        match direction {
            'w' => {
                if self.scroll_row >= 10 {
                    self.scroll_row -= 10;
                } else {
                    self.scroll_row = 0;
                }
            }
            's' => {
                if self.scroll_row + 10 < self.rows {
                    self.scroll_row += 10;
                }
            }
            'a' => {
                if self.scroll_col >= 10 {
                    self.scroll_col -= 10;
                } else {
                    self.scroll_col = 0;
                }
            }
            'd' => {
                if self.scroll_col + 10 < self.cols {
                    self.scroll_col += 10;
                }
            }
            _ => return Err("Invalid direction".to_string()),
        }
        Ok(())
    }

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
     
    
    fn convert_to_column_name(mut col: u16) -> String {
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

