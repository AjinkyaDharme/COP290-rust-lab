# COP290 RUST LAB

A command-line spreadsheet application written in Rust that also supports gui

## First time setup
```bash
git clone {repo_link}
cd COP290-rust-lab

# Update package lists
sudo apt-get update

# Install dependencies one by one
sudo apt-get install -y pkg-config
sudo apt-get install -y libfontconfig1-dev
sudo apt-get install -y libasound2-dev
```

## Directory structure
```

ext1/
src/
├── command.rs     # Command handling
├── evaluator.rs   # Formula evaluation
├── lib.rs         # Library exports
├── main.rs        # Entry point
├── parser.rs      # Expression parsing
├── recalculation.rs # Cell dependency management
└── sheet.rs       # Core spreadsheet functionality
tests/
├── src/
Makefile
Cargo.toml
Cargo.lock
Readme.md
```
## Autograder part
1. Build the project:
```bash
make
```

2. Run the project:
```bash
make run
```

## Testing

Run the tests:
```bash
make test
```
## Extensions 


We have implemented a terminal based extension as well as gui based extension.

Run the extension directly using:
```bash
make ext1
```
After the extension has started running to open the gui type 'gui' as the command, it will open a gui displaying the spreadsheet. To close the gui and end type 'q' to end the process and exit.Here by default it creates 20*10 size spreadhsheet so that it is easy to view and operate. You can change the dimensions by changing ext1 part of Makefile


## Contributors
- [Ajinkya Dharme 2023CS10761]
- [Manvendra Rajpurohit 2023CS1036]
- [Akshit Kujur 2023CS10100]
