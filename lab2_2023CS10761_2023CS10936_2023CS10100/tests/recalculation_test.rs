use spreadsheet::command::{BinaryOp, CellRef, Command, Expr};
use spreadsheet::recalculation::{
    RangeRef, RecalcManager, cell_to_string, extract_cell_refs, extract_range_refs, recalculate,
    string_to_cell,
};
use spreadsheet::sheet::{CellValue, Spreadsheet};
use std::collections::HashSet;

#[test]
fn test_cell_string_conversion() {
    // Test converting CellRef to string
    let cell = CellRef { row: 1, col: 1 };
    assert_eq!(cell_to_string(&cell), "A1");

    let cell = CellRef { row: 10, col: 26 };
    assert_eq!(cell_to_string(&cell), "Z10");

    let cell = CellRef { row: 5, col: 27 };
    assert_eq!(cell_to_string(&cell), "AA5");

    // Test converting string to CellRef
    let cell = string_to_cell("A1").unwrap();
    assert_eq!(cell.row, 1);
    assert_eq!(cell.col, 1);

    let cell = string_to_cell("Z10").unwrap();
    assert_eq!(cell.row, 10);
    assert_eq!(cell.col, 26);

    let cell = string_to_cell("AA5").unwrap();
    assert_eq!(cell.row, 5);
    assert_eq!(cell.col, 27);

    // Test edge cases
    let cell = string_to_cell("AAA100").unwrap();
    assert_eq!(cell.row, 100);
    assert_eq!(cell.col, 703);

    // Test invalid cell string
    assert!(string_to_cell("").is_none());
    assert!(string_to_cell("A").is_none());
    assert!(string_to_cell("1").is_none());
    assert!(string_to_cell("A-1").is_none());
    assert!(string_to_cell("1A").is_none());
}

#[test]
fn test_dependency_tracking() {
    let mut manager = RecalcManager::new();

    // Create a command where A1 depends on B1
    let cmd = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: Expr::CellRef(CellRef { row: 1, col: 2 }),
    };

    let result = manager.update_for_command(&cmd);
    assert!(result.is_ok());

    // Check that dependencies were recorded correctly
    assert!(manager.parents.contains_key("A1"));
    let deps = manager.parents.get("A1").unwrap();
    assert!(deps.contains("B1"));

    assert!(manager.children.contains_key("B1"));
    let children = manager.children.get("B1").unwrap();
    assert!(children.contains("A1"));

    // Test updating an existing dependency
    let cmd2 = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: Expr::CellRef(CellRef { row: 1, col: 3 }),
    };

    let result = manager.update_for_command(&cmd2);
    assert!(result.is_ok());

    // B1 should no longer have A1 as a child
    assert!(!manager.children.get("B1").unwrap().contains("A1"));

    // C1 should now have A1 as a child
    assert!(manager.children.get("C1").unwrap().contains("A1"));

    // A1's parent should now be C1, not B1
    assert!(!manager.parents.get("A1").unwrap().contains("B1"));
    assert!(manager.parents.get("A1").unwrap().contains("C1"));
}

#[test]
fn test_range_dependency() {
    let mut manager = RecalcManager::new();

    // A1 depends on range B1:C2
    let cmd = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: Expr::Range(CellRef { row: 1, col: 2 }, CellRef { row: 2, col: 3 }),
    };

    let result = manager.update_for_command(&cmd);
    assert!(result.is_ok());

    // Check that range dependencies were recorded correctly
    assert!(manager.range_parents.contains_key("A1"));
    let range_deps = manager.range_parents.get("A1").unwrap();
    assert_eq!(range_deps.len(), 1);

    let range_ref = range_deps.iter().next().unwrap();
    assert_eq!(range_ref.start.col, 2);
    assert_eq!(range_ref.start.row, 1);
    assert_eq!(range_ref.end.col, 3);
    assert_eq!(range_ref.end.row, 2);

    // Check range_children
    assert_eq!(manager.range_children.len(), 1);
    let (range, child) = &manager.range_children[0];
    assert_eq!(child, "A1");
    assert_eq!(range.start.col, 2);
    assert_eq!(range.start.row, 1);
    assert_eq!(range.end.col, 3);
    assert_eq!(range.end.row, 2);

    // Test modifying range dependency
    let cmd2 = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: Expr::Range(CellRef { row: 3, col: 4 }, CellRef { row: 4, col: 5 }),
    };

    let result = manager.update_for_command(&cmd2);
    assert!(result.is_ok());

    // Check that old range dependency is removed
    let range_deps = manager.range_parents.get("A1").unwrap();
    assert_eq!(range_deps.len(), 1);
    let range_ref = range_deps.iter().next().unwrap();
    assert_eq!(range_ref.start.row, 3);
    assert_eq!(range_ref.start.col, 4);

    // Check range_children is updated
    assert_eq!(manager.range_children.len(), 1);
    let (range, child) = &manager.range_children[0];
    assert_eq!(child, "A1");
    assert_eq!(range.start.row, 3);
    assert_eq!(range.start.col, 4);
}

#[test]
fn test_range_contains() {
    // Normal range
    let range = RangeRef {
        start: CellRef { row: 1, col: 1 },
        end: CellRef { row: 3, col: 3 },
    };

    // Test cells inside range
    assert!(range.contains(&CellRef { row: 1, col: 1 }));
    assert!(range.contains(&CellRef { row: 2, col: 2 }));
    assert!(range.contains(&CellRef { row: 3, col: 3 }));

    // Test cells outside range
    assert!(!range.contains(&CellRef { row: 4, col: 4 }));
    assert!(!range.contains(&CellRef { row: 0, col: 0 }));

    // Test inverted range (end before start)
    let inverted_range = RangeRef {
        start: CellRef { row: 3, col: 3 },
        end: CellRef { row: 1, col: 1 },
    };

    // Should still contain the same cells
    assert!(inverted_range.contains(&CellRef { row: 1, col: 1 }));
    assert!(inverted_range.contains(&CellRef { row: 2, col: 2 }));
    assert!(inverted_range.contains(&CellRef { row: 3, col: 3 }));
}

#[test]
fn test_cycle_detection() {
    let mut manager = RecalcManager::new();

    // Set up A1 = B1
    let cmd1 = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: Expr::CellRef(CellRef { row: 1, col: 2 }),
    };
    let result = manager.update_for_command(&cmd1);
    assert!(result.is_ok());

    // Set up B1 = C1
    let cmd2 = Command::SetCell {
        cell: CellRef { row: 1, col: 2 },
        expr: Expr::CellRef(CellRef { row: 1, col: 3 }),
    };
    let result = manager.update_for_command(&cmd2);
    assert!(result.is_ok());

    // Try to set up C1 = A1, which would create a cycle
    let cmd3 = Command::SetCell {
        cell: CellRef { row: 1, col: 3 },
        expr: Expr::CellRef(CellRef { row: 1, col: 1 }),
    };
    let result = manager.update_for_command(&cmd3);
    assert!(result.is_err());

    // Make sure the cycle error contains the appropriate cell
    let err = result.unwrap_err();
    assert!(err.contains("Cycle detected"));

    // Test cycle through range dependencies
    let mut manager = RecalcManager::new();

    // A1 = B1
    manager
        .update_for_command(&Command::SetCell {
            cell: CellRef { row: 1, col: 1 },
            expr: Expr::CellRef(CellRef { row: 1, col: 2 }),
        })
        .unwrap();

    // B1 = range(C1:D1)
    manager
        .update_for_command(&Command::SetCell {
            cell: CellRef { row: 1, col: 2 },
            expr: Expr::Range(CellRef { row: 1, col: 3 }, CellRef { row: 1, col: 4 }),
        })
        .unwrap();

    // Try C1 = A1 which would create a cycle through range
    let result = manager.update_for_command(&Command::SetCell {
        cell: CellRef { row: 1, col: 3 },
        expr: Expr::CellRef(CellRef { row: 1, col: 1 }),
    });

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Cycle detected"));
}

#[test]
fn test_topological_sort() {
    let mut manager = RecalcManager::new();

    // Set up A1 = B1 + C1
    let cmd1 = Command::SetCell {
        cell: CellRef { row: 1, col: 1 },
        expr: Expr::BinaryOp(
            Box::new(Expr::CellRef(CellRef { row: 1, col: 2 })),
            BinaryOp::Add,
            Box::new(Expr::CellRef(CellRef { row: 1, col: 3 })),
        ),
    };
    let result = manager.update_for_command(&cmd1);
    assert!(result.is_ok());

    // Set up D1 = A1
    let cmd2 = Command::SetCell {
        cell: CellRef { row: 1, col: 4 },
        expr: Expr::CellRef(CellRef { row: 1, col: 1 }),
    };
    let result = manager.update_for_command(&cmd2);
    assert!(result.is_ok());

    // Check that updating B1 correctly identifies cells to recalculate
    let exclude = "B1".to_string();
    let result = manager.topological_sort_excluding(&exclude);
    assert!(result.is_ok());

    let order = result.unwrap();
    // A1 depends on B1, and D1 depends on A1, so both should be recalculated
    assert!(order.contains(&"A1".to_string()));
    assert!(order.contains(&"D1".to_string()));
    // And A1 must come before D1
    let pos_a1 = order.iter().position(|s| s == "A1").unwrap();
    let pos_d1 = order.iter().position(|s| s == "D1").unwrap();
    assert!(pos_a1 < pos_d1);

    // Test with range dependency
    let mut manager = RecalcManager::new();

    // E1 = SUM(A1:B1)
    let cmd = Command::SetCell {
        cell: CellRef { row: 1, col: 5 },
        expr: Expr::Range(CellRef { row: 1, col: 1 }, CellRef { row: 1, col: 2 }),
    };
    manager.update_for_command(&cmd).unwrap();

    // F1 = E1
    let cmd2 = Command::SetCell {
        cell: CellRef { row: 1, col: 6 },
        expr: Expr::CellRef(CellRef { row: 1, col: 5 }),
    };
    manager.update_for_command(&cmd2).unwrap();

    // Updating A1 should cause E1 and F1 to be recalculated
    let order = manager
        .topological_sort_excluding(&"A1".to_string())
        .unwrap();
    assert!(order.contains(&"E1".to_string()));
    assert!(order.contains(&"F1".to_string()));
    let pos_e1 = order.iter().position(|s| s == "E1").unwrap();
    let pos_f1 = order.iter().position(|s| s == "F1").unwrap();
    assert!(pos_e1 < pos_f1);
}

#[test]
fn test_extract_cell_refs() {
    let mut refs = HashSet::new();

    // Test simple cell ref
    let expr = Expr::CellRef(CellRef { row: 1, col: 1 });
    extract_cell_refs(&expr, &mut refs);
    assert!(refs.contains("A1"));

    // Test binary op with multiple refs
    refs.clear();
    let expr = Expr::BinaryOp(
        Box::new(Expr::CellRef(CellRef { row: 1, col: 1 })),
        BinaryOp::Add,
        Box::new(Expr::CellRef(CellRef { row: 2, col: 2 })),
    );
    extract_cell_refs(&expr, &mut refs);
    assert!(refs.contains("A1"));
    assert!(refs.contains("B2"));

    // Test function call
    refs.clear();
    let expr = Expr::FunctionCall(
        spreadsheet::command::Function::Sum,
        Box::new(Expr::CellRef(CellRef { row: 3, col: 3 })),
    );
    extract_cell_refs(&expr, &mut refs);
    assert!(refs.contains("C3"));

    // Test range
    refs.clear();
    let expr = Expr::Range(CellRef { row: 1, col: 1 }, CellRef { row: 3, col: 3 });
    extract_cell_refs(&expr, &mut refs);
    assert!(refs.contains("A1"));
    assert!(refs.contains("C3"));

    // Test constant - should not add any refs
    refs.clear();
    let expr = Expr::Constant(42);
    extract_cell_refs(&expr, &mut refs);
    assert!(refs.is_empty());
}

#[test]
fn test_extract_range_refs() {
    let mut range_refs = HashSet::new();

    // Test simple range
    let expr = Expr::Range(CellRef { row: 1, col: 1 }, CellRef { row: 3, col: 3 });
    extract_range_refs(&expr, &mut range_refs);
    assert_eq!(range_refs.len(), 1);
    let range = range_refs.iter().next().unwrap();
    assert_eq!(range.start.row, 1);
    assert_eq!(range.start.col, 1);
    assert_eq!(range.end.row, 3);
    assert_eq!(range.end.col, 3);

    // Test binary op with range
    range_refs.clear();
    let expr = Expr::BinaryOp(
        Box::new(Expr::Range(
            CellRef { row: 1, col: 1 },
            CellRef { row: 2, col: 2 },
        )),
        BinaryOp::Add,
        Box::new(Expr::Range(
            CellRef { row: 3, col: 3 },
            CellRef { row: 4, col: 4 },
        )),
    );
    extract_range_refs(&expr, &mut range_refs);
    assert_eq!(range_refs.len(), 2);

    // Test function call with range
    range_refs.clear();
    let expr = Expr::FunctionCall(
        spreadsheet::command::Function::Sum,
        Box::new(Expr::Range(
            CellRef { row: 1, col: 1 },
            CellRef { row: 5, col: 5 },
        )),
    );
    extract_range_refs(&expr, &mut range_refs);
    assert_eq!(range_refs.len(), 1);

    // Test with cell ref - should not add any range
    range_refs.clear();
    let expr = Expr::CellRef(CellRef { row: 1, col: 1 });
    extract_range_refs(&expr, &mut range_refs);
    assert!(range_refs.is_empty());

    // Test with constant - should not add any range
    range_refs.clear();
    let expr = Expr::Constant(42);
    extract_range_refs(&expr, &mut range_refs);
    assert!(range_refs.is_empty());
}

#[test]
fn test_recalculate() {
    let mut sheet = Spreadsheet::new(10, 10); // Create a 10x10 spreadsheet

    // Set up B1 = 10, C1 = 5
    sheet.set_by_key("B1", CellValue::Value(10)).unwrap();
    sheet.set_by_key("C1", CellValue::Value(5)).unwrap();

    // Set A1 = B1 + C1 (formula)
    let a1_cell_str = cell_to_string(&CellRef { row: 1, col: 1 });
    sheet.set_formula(
        &a1_cell_str,
        Expr::BinaryOp(
            Box::new(Expr::CellRef(CellRef { row: 1, col: 2 })),
            BinaryOp::Add,
            Box::new(Expr::CellRef(CellRef { row: 1, col: 3 })),
        ),
    );

    // Set D1 = A1 * 2 (formula)
    let d1_cell_str = cell_to_string(&CellRef { row: 1, col: 4 });
    sheet.set_formula(
        &d1_cell_str,
        Expr::BinaryOp(
            Box::new(Expr::CellRef(CellRef { row: 1, col: 1 })),
            BinaryOp::Multiply,
            Box::new(Expr::Constant(2)),
        ),
    );

    // Recalculate in topological order
    recalculate(&mut sheet, vec!["A1".to_string(), "D1".to_string()]);

    // Check results
    if let CellValue::Value(val) = sheet.get(0, 0).unwrap() {
        assert_eq!(*val, 15); // B1 + C1 = 10 + 5 = 15
    } else {
        panic!("A1 should be a Value");
    }

    if let CellValue::Value(val) = sheet.get(0, 3).unwrap() {
        assert_eq!(*val, 30); // A1 * 2 = 15 * 2 = 30
    } else {
        panic!("D1 should be a Value");
    }

    // Test error handling in recalculate
    // Make A1 refer to a non-existent cell (outside the 10x10 grid we created)
    sheet.set_formula(&a1_cell_str, Expr::CellRef(CellRef { row: 20, col: 20 }));

    recalculate(&mut sheet, vec!["A1".to_string(), "D1".to_string()]);

    // A1 should now be an error
    if let CellValue::Error(_) = sheet.get(0, 0).unwrap() {
        // Test passed
    } else {
        panic!("A1 should be an Error");
    }

    // And D1 should also propagate the error
    if let CellValue::Error(_) = sheet.get(0, 3).unwrap() {
        // Test passed
    } else {
        panic!("D1 should be an Error");
    }
}
