.PHONY: all clean run

all: clean target/release/spreadsheet

target/release/spreadsheet:
	cargo build --release

clean:
	cargo clean

run: target/release/spreadsheet
	./target/release/spreadsheet 999 18278
