use crate::command::{CellRef, Command, Expr};
use crate::evaluator::eval_expr;
use crate::sheet::{CellValue, Spreadsheet};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RangeRef {
    pub start: CellRef,
    pub end: CellRef,
}

impl RangeRef {
    pub fn contains(&self, cell: &CellRef) -> bool {
        let (s_col, s_row) = (self.start.col, self.start.row);
        let (e_col, e_row) = (self.end.col, self.end.row);
        let (min_col, max_col) = (s_col.min(e_col), s_col.max(e_col));
        let (min_row, max_row) = (s_row.min(e_row), s_row.max(e_row));
        (cell.col >= min_col && cell.col <= max_col) && (cell.row >= min_row && cell.row <= max_row)
    }
}

pub fn string_to_cell(s: &str) -> Option<CellRef> {
    let (col_letters, row_digits) = s.split_at(s.find(|c: char| c.is_ascii_digit())?);
    if col_letters.is_empty() || row_digits.is_empty() {
        return None;
    }
    let mut col = 0;
    for c in col_letters.chars() {
        if !c.is_ascii_alphabetic() {
            return None;
        }
        col = col * 26 + ((c.to_ascii_uppercase() as u8) - b'A' + 1) as u16;
    }
    let row = row_digits.parse::<u16>().ok()?;
    Some(CellRef { col, row })
}

pub type DependencyGraph = HashMap<String, HashSet<String>>;

#[derive(Debug, Clone)]
pub struct RecalcManager {
    pub children: DependencyGraph,
    pub parents: DependencyGraph,
    pub range_parents: HashMap<String, HashSet<RangeRef>>,
    pub range_children: Vec<(RangeRef, String)>,
}

impl RecalcManager {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            parents: HashMap::new(),
            range_parents: HashMap::new(),
            range_children: Vec::new(),
        }
    }

    pub fn update_for_command(&mut self, cmd: &Command) -> Result<Vec<String>, String> {
        if let Command::SetCell { cell, expr } = cmd {
            let cell_key = cell_to_string(cell);
            let mut new_refs = HashSet::new();
            extract_cell_refs(expr, &mut new_refs);
            let mut new_range_refs: HashSet<RangeRef> = HashSet::new();
            extract_range_refs(expr, &mut new_range_refs);
            let old_refs = self.parents.get(&cell_key).cloned().unwrap_or_default();
            let old_range_refs = self
                .range_parents
                .get(&cell_key)
                .cloned()
                .unwrap_or_default();
            let removed = old_refs
                .difference(&new_refs)
                .cloned()
                .collect::<HashSet<_>>();
            let added = new_refs
                .difference(&old_refs)
                .cloned()
                .collect::<HashSet<_>>();
            let removed_range = old_range_refs
                .difference(&new_range_refs)
                .cloned()
                .collect::<HashSet<_>>();
            let added_range = new_range_refs
                .difference(&old_range_refs)
                .cloned()
                .collect::<HashSet<_>>();
            self.parents.insert(cell_key.clone(), new_refs.clone());
            self.range_parents
                .insert(cell_key.clone(), new_range_refs.clone());
            for dep in &removed {
                if let Some(child_set) = self.children.get_mut(dep) {
                    child_set.remove(&cell_key);
                }
            }
            for dep in &added {
                self.children
                    .entry(dep.clone())
                    .or_default()
                    .insert(cell_key.clone());
            }
            self.range_children
                .retain(|(range, child)| !(child == &cell_key && removed_range.contains(range)));
            for range in &added_range {
                self.range_children.push((range.clone(), cell_key.clone()));
            }
            match self.topological_sort_excluding(&cell_key) {
                Ok(order) => return Ok(order),
                Err(e) => {
                    self.parents.insert(cell_key.clone(), old_refs);
                    self.range_parents.insert(cell_key.clone(), old_range_refs);
                    for dep in &added {
                        if let Some(child_set) = self.children.get_mut(dep) {
                            child_set.remove(&cell_key);
                        }
                    }
                    for dep in &removed {
                        self.children
                            .entry(dep.clone())
                            .or_default()
                            .insert(cell_key.clone());
                    }
                    self.range_children.retain(|(range, child)| {
                        !(child == &cell_key && added_range.contains(range))
                    });
                    for range in &removed_range {
                        self.range_children.push((range.clone(), cell_key.clone()));
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

        // Track the current path to reconstruct the cycle if detected
        let mut path: Vec<String> = Vec::new();

        // Helper to push all dependents (both direct and via range) for a given cell key.
        let push_dependents = |cell_key: &String,
                               stack: &mut Vec<(String, bool)>,
                               fully_visited: &HashSet<String>| {
            // Direct children.
            if let Some(children) = self.children.get(cell_key) {
                for child in children {
                    if !fully_visited.contains(child) {
                        stack.push((child.clone(), false));
                    }
                }
            }
            // Range dependencies: if cell_key can be parsed to a CellRef, then search range_children.
            if let Some(cell_ref) = string_to_cell(cell_key) {
                for (range, dependent) in &self.range_children {
                    // Use dependent only if not already visited.
                    if range.contains(&cell_ref) && !fully_visited.contains(dependent) {
                        stack.push((dependent.clone(), false));
                    }
                }
            }
        };

        // Start from all direct children (and those from range deps) of the updated cell.
        push_dependents(exclude, &mut dfs_stack, &fully_visited);

        while let Some((current, expanded)) = dfs_stack.pop() {
            if expanded {
                // Finished processing this node
                in_current_path.remove(&current);
                path.pop();
                if !result.contains(&current) {
                    result.push(current.clone());
                }
                fully_visited.insert(current);
            } else {
                // About to process this node
                if in_current_path.contains(&current) {
                    // Cycle detected
                    // Find the start of the cycle in our current path
                    let cycle_start_idx = path.iter().position(|x| x == &current).unwrap_or(0);

                    // Extract the cycle nodes (including the node that completes the cycle)
                    let mut cycle = path[cycle_start_idx..].to_vec();
                    cycle.push(current.clone()); // Complete the cycle

                    // Format as A1->A3->A5->A1
                    let cycle_str = cycle.join("->");
                    println!("Cycle detected: {}", cycle_str);
                    return Err(format!("Cycle detected: {}", cycle_str));
                }

                // Mark as being visited in the current DFS path
                in_current_path.insert(current.clone());
                path.push(current.clone());

                // Push back to stack in expanded state
                dfs_stack.push((current.clone(), true));

                // Continue DFS with children
                push_dependents(&current, &mut dfs_stack, &fully_visited);
            }
        }

        // Reverse the result to get the correct order.
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
                }
                Err(_err) => {
                    if let Err(e) = sheet.set_by_key(&cell_key, CellValue::Error(())) {
                        eprintln!("Error updating {}: {}", cell_key, e);
                    }
                }
            }
        }
    }
}

pub fn extract_cell_refs(expr: &Expr, refs: &mut HashSet<String>) {
    match expr {
        Expr::Constant(_) => {}
        Expr::CellRef(cell) => {
            refs.insert(cell_to_string(cell));
        }
        Expr::BinaryOp(lhs, _op, rhs) => {
            extract_cell_refs(lhs, refs);
            extract_cell_refs(rhs, refs);
        }
        Expr::FunctionCall(_, args) => {
            for arg in args {
                extract_cell_refs(arg, refs);
            }
        }
        Expr::Range(start, end) => {
            refs.insert(cell_to_string(start));
            refs.insert(cell_to_string(end));
        }
    }
}

pub fn extract_range_refs(expr: &Expr, range_refs: &mut HashSet<RangeRef>) {
    match expr {
        Expr::Range(start, end) => {
            range_refs.insert(RangeRef {
                start: start.clone(),
                end: end.clone(),
            });
        }
        Expr::BinaryOp(lhs, _op, rhs) => {
            extract_range_refs(lhs, range_refs);
            extract_range_refs(rhs, range_refs);
        }
        Expr::FunctionCall(_, args) => {
            for arg in args {
                extract_range_refs(arg, range_refs);
            }
        }
        _ => {}
    }
}

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