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
    rows: usize,
    cols: usize,
    cells: Vec<CellValue>,
}

impl Spreadsheet {
    pub fn new(rows: usize, cols: usize) -> Self {
        let cells = vec![CellValue::default(); rows * cols];
        Self { rows, cols, cells }
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
            print!("{}\t", convert_to_column_name(col as u16));
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