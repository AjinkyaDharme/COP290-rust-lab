//! This crate is organized into several modules that separate concerns
//! for command definitions, expression evaluation, input parsing, recalculation logic, and the spreadsheet implementation.

/// Contains definitions for commands, including enums and structures representing user actions.
pub mod command;

/// Provides functionalities for evaluating expressions and executing commands
/// within the spreadsheet context.
pub mod evaluator;

/// Implements parsing logic for converting user input or data into expressions or commands.
pub mod parser;

/// Handles the recalculation logic, including dependency management and updating cells
/// when changes occur.
pub mod recalculation;

/// Implements the spreadsheet data structure along with its associated operations,
/// such as setting and retrieving cell values, scrolling, and navigation.
pub mod sheet;
