# Rust Spreadsheet

A command-line spreadsheet application written in Rust that supports a 999 x 18278 cell grid.

## Features

- Large spreadsheet capacity (999 rows x 18278 columns)
- Formula evaluation
- Cell recalculation
- Command parsing

## Project Structure

```
src/
├── command.rs     # Command handling
├── evaluator.rs   # Formula evaluation
├── lib.rs         # Library exports
├── main.rs        # Entry point
├── parser.rs      # Expression parsing
├── recalculation.rs # Cell dependency management
└── sheet.rs       # Core spreadsheet functionality
```

## Installation

1. Clone the repository:
```bash
git clone {repo_link}
cd lab2_2023CS10761_2023CS10936_2023CS10100
```

2. Build the project:
```bash
make
```

3. Run the project:
```bash
cargo run
```

## Usage

Run the extension:
```bash
make ext1
```

## Testing

Run the tests:
```bash
make test
```


## Contributors

Ajinkya Dharme
Manvendra Rajpurohit
Akshit Kujur
