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

    pub fn update_for_command(&mut self, cmd: &Command) -> Result<Vec<String>, String> {
        if let Command::SetCell { cell, expr } = cmd {
            let cell_key = cell_to_string(cell);

            // Compute new dependencies from the expression.
            let mut new_refs = HashSet::new();
            extract_cell_refs(expr, &mut new_refs);

            // Get current dependencies for this cell.
            let old_refs = self.parents.get(&cell_key).cloned().unwrap_or_default();

            // Determine which dependencies are removed and which are added.
            let removed: HashSet<String> = old_refs.difference(&new_refs).cloned().collect();
            let added: HashSet<String>   = new_refs.difference(&old_refs).cloned().collect();

            // Update the parent's entry with the new set.
            self.parents.insert(cell_key.clone(), new_refs.clone());

            // For dependencies that were removed, remove cell_key from their children.
            for dep in &removed {
                if let Some(child_set) = self.children.get_mut(dep) {
                    child_set.remove(&cell_key);
                }
            }
            // For newly added dependencies, add cell_key to their children.
            for dep in &added {
                self.children.entry(dep.clone()).or_default().insert(cell_key.clone());
            }

           
            match self.topological_sort_excluding(&cell_key) {
                Ok(order) => return Ok(order),
                Err(e) => {
                    // Roll back updates:
                    self.parents.insert(cell_key.clone(), old_refs.clone());
                    for dep in &added {
                        if let Some(child_set) = self.children.get_mut(dep) {
                            child_set.remove(&cell_key);
                        }
                    }
                    for dep in &removed {
                        self.children.entry(dep.clone()).or_default().insert(cell_key.clone());
                    }
                    return Err(e);
                }
            }
        }
        Ok(Vec::new())
    }

   
    pub fn topological_sort_excluding(&self, exclude: &String) -> Result<Vec<String>, String> {
        let mut fully_visited: HashSet<String> = HashSet::new();
        let mut in_current_path: HashSet<String> = HashSet::new();
        let mut result: Vec<String> = Vec::new();
        let mut dfs_stack: Vec<(String, bool)> = Vec::new();

        // Start with all direct children of the updated cell.
        if let Some(initial_children) = self.children.get(exclude) {
            for child in initial_children {
                dfs_stack.push((child.clone(), false));
            }
        }

        while let Some((current, expanded)) = dfs_stack.pop() {
            if expanded {
                in_current_path.remove(&current);
                if !result.contains(&current) {
                    result.push(current.clone());
                }
                fully_visited.insert(current);
            } else {
                if in_current_path.contains(&current) {
                    return Err(format!("Cycle detected at {}", current));
                }
                in_current_path.insert(current.clone());
                dfs_stack.push((current.clone(), true));
                if let Some(children) = self.children.get(&current) {
                    for child in children {
                        if !fully_visited.contains(child) {
                            dfs_stack.push((child.clone(), false));
                        }
                    }
                }
            }
        }
        result.reverse();
        result.retain(|node| node != exclude);
        Ok(result)
    }
}


pub fn recalculate(sheet: &mut Spreadsheet, topo_order: Vec<String>) {
    for cell_key in topo_order {
        if let Some(expr) = sheet.get_formula(&cell_key) {
            let result = eval_expr(expr, sheet);
            match result {
                Ok(val) => {
                    if let Err(e) = sheet.set_by_key(&cell_key, CellValue::Value(val)) {
                        eprintln!("Error updating {}: {}", cell_key, e);
                    }
                },
                Err(_err) => {
                    if let Err(e) = sheet.set_by_key(&cell_key, CellValue::Error("ERR".into())) {
                        eprintln!("Error updating {}: {}", cell_key, e);
                    }
                }
            }
        }
    }
}

/// Recursively extract all cell references from an expression and add them to `refs`.
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

/// Helper: Converts a CellRef into a cell key string (e.g. "A1").
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
