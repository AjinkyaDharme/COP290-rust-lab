use std::collections::{HashMap, HashSet};
use crate::command::{Command, Expr, CellRef};
use crate::evaluator::{eval_expr, EvalError};
use crate::sheet::{CellValue, Spreadsheet};

pub type DependencyGraph = HashMap<String, HashSet<String>>;

#[derive(Debug, Clone)]
pub struct RecalcManager {

    pub children: DependencyGraph,
    pub parents: DependencyGraph,
}

impl RecalcManager {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            parents: HashMap::new(),
        }
    }


    pub fn update_for_command(&mut self, cmd: &Command) -> Result<(), String> {

        if let Command::SetCell { cell, expr } = cmd {
            let cell_key = cell_to_string(cell);
            // Save the current state to revert if necessary.
            let old_parents = self.parents.clone();
            let old_children = self.children.clone();

            // Remove previous dependencies for this cell.
            if let Some(old_parents_set) = self.parents.get(&cell_key) {
                for p in old_parents_set {
                    if let Some(children_set) = self.children.get_mut(p) {
                        children_set.remove(&cell_key);
                    }
                }
            }
            // Clear any existing parent dependencies.
            self.parents.insert(cell_key.clone(), HashSet::new());

            // Extract new cell references from the expression.
            let mut refs = HashSet::new();
            extract_cell_refs(expr, &mut refs);

            // For each referenced cell, update parent's and children maps.
            for r in refs {
                // For the set cell, add a dependency on each referenced cell.
                self.parents.get_mut(&cell_key).unwrap().insert(r.clone());
                // And update the children list for the referenced cell.
                self.children.entry(r).or_default().insert(cell_key.clone());
            }

            // Check for cycles via topological sort.
            match self.topological_sort() {
                Ok(_order) => {
                    return Ok(());
                }
                Err(e) => {
                    // Cycle found; revert to the old dependency state.
                    self.parents = old_parents;
                    self.children = old_children;
                    return Err(e);
                }
            }
        }
        Ok(())
    }

 
    pub fn topological_sort(&self) -> Result<Vec<String>, String> {
        let mut visited = HashSet::new();
        let mut temp_marks = HashSet::new();
        let mut order = Vec::new();

        for node in self.parents.keys() {
            if !visited.contains(node) {
                self.visit(node, &mut visited, &mut temp_marks, &mut order)?;
            }
        }
        order.reverse();
        Ok(order)
    }

    fn visit(
        &self,
        node: &String,
        visited: &mut HashSet<String>,
        temp_marks: &mut HashSet<String>,
        order: &mut Vec<String>,
    ) -> Result<(), String> {
        if temp_marks.contains(node) {
            return Err(format!("Cycle found at {}", node));
        }
        if !visited.contains(node) {
            temp_marks.insert(node.clone());
            if let Some(children) = self.children.get(node) {
                for child in children {
                    self.visit(child, visited, temp_marks, order)?;
                }
            }
            temp_marks.remove(node);
            visited.insert(node.clone());
            order.push(node.clone());
        }
        Ok(())
    }
}


pub fn recalculate(sheet: &mut Spreadsheet, topo_order: Vec<String>) {
    for cell_key in topo_order {
        // Check if there is a formula for this cell.
        if let Some(expr) = sheet.get_formula(&cell_key) {
            // Evaluate the expression using our evaluator.
            let result = eval_expr(expr, sheet);
            // Based on the result, update the cell value.
            match result {
                Ok(val) => {
                    // Set the new computed value.
                    if let Err(e) = sheet.set_by_key(&cell_key, CellValue::Value(val)) {
                        eprintln!("Error updating {}: {}", cell_key, e);
                    }
                }
                Err(_err) => {
                    // If there was an error (e.g. division by zero or error propagation),
                    // mark the cell with "ERR".
                    if let Err(e) = sheet.set_by_key(&cell_key, CellValue::Error("ERR".into())) {
                        eprintln!("Error updating {}: {}", cell_key, e);
                    }
                }
            }
        }
    }
}

/// Recursively extracts all cell references from an expression and adds them to `refs`.
pub fn extract_cell_refs(expr: &Expr, refs: &mut HashSet<String>) {
    match expr {
        Expr::Constant(_) => {},
        Expr::CellRef(cell) => { refs.insert(cell_to_string(cell)); },
        Expr::BinaryOp(lhs, _op, rhs) => {
            extract_cell_refs(lhs, refs);
            extract_cell_refs(rhs, refs);
        },
        Expr::FunctionCall(_, arg) => {
            extract_cell_refs(arg, refs);
        },
        Expr::Range(start, end) => {
            refs.insert(cell_to_string(start));
            refs.insert(cell_to_string(end));
        },
    }
}

/// Helper: converts a CellRef into a cell key string (e.g., "A1").
pub fn cell_to_string(cell: &CellRef) -> String {
    let mut col = cell.col;
    let mut col_str = String::new();
    while col > 0 {
        let rem = ((col - 1) % 26) as u8;
        col_str.insert(0, (b'A' + rem) as char);
        col = (col - 1) / 26;
    }
    format!("{}{}", col_str, cell.row)
}

