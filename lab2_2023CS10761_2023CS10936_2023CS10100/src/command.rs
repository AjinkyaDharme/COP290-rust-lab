/// Represents a user-issued command in the spreadsheet application.
#[derive(Debug, Clone)]
pub enum Command {
    /// Set the expression of a given cell.
    SetCell { cell: CellRef, expr: Expr },

    /// Scroll the view to a specific cell.
    ScrollTo(CellRef),

    /// Scroll the view one row up.
    ScrollUp,

    /// Scroll the view one row down.
    ScrollDown,

    /// Scroll the view one column to the left.
    ScrollLeft,

    /// Scroll the view one column to the right.
    ScrollRight,

    /// Disable output display.
    DisableOutput,

    /// Enable output display.
    EnableOutput,

    /// Quit the application.
    Quit,
}

/// A reference to a cell in the spreadsheet, identified by column and row.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellRef {
    /// Column index of the cell.
    pub col: u16,

    /// Row index of the cell.
    pub row: u16,
}

/// Represents an expression that can be evaluated in a cell.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// A constant integer value.
    Constant(i32),

    /// A reference to another cell.
    CellRef(CellRef),

    /// A binary operation between two expressions.
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),

    /// A function call on an expression.
    FunctionCall(Function, Box<Expr>),

    /// A range of cells from start to end.
    Range(CellRef, CellRef),
}

/// Supported binary operations for expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOp {
    /// Addition
    Add,

    /// Subtraction
    Subtract,

    /// Multiplication
    Multiply,

    /// Division
    Divide,
}

/// Built-in spreadsheet functions that can be applied to expressions.
#[derive(Debug, Clone, PartialEq)]
pub enum Function {
    /// Minimum value
    Min,

    /// Maximum value
    Max,

    /// Average value
    Avg,

    /// Sum of values
    Sum,

    /// Standard deviation
    Stdev,

    /// Sleep or delay (for testing or demonstration)
    Sleep,
}
