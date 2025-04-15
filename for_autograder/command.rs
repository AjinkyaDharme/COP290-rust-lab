#[derive(Debug, Clone)]
pub enum Command {
    SetCell { cell: CellRef, expr: Expr },
    ScrollTo(CellRef),
    ScrollUp,
    ScrollDown,
    ScrollLeft,
    ScrollRight,
    DisableOutput,
    EnableOutput,
    Quit,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CellRef {
    pub col: u16,
    pub row: u16,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Constant(i32),
    CellRef(CellRef),
    BinaryOp(Box<Expr>, BinaryOp, Box<Expr>),
    FunctionCall(Function, Box<Expr>),
    Range(CellRef, CellRef),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
pub enum Function {
    Min,
    Max,
    Avg,
    Sum,
    Stdev,
    Sleep,
}
