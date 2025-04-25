.PHONY: all clean run test coverage docs ext1 ext1-clean

# Main project targets
all: clean target/release/spreadsheet

target/release/spreadsheet:
	cargo build --release

clean:
	cargo clean

run: target/release/spreadsheet
	./target/release/spreadsheet 999 18278

test:
	cargo test

coverage:
	cargo tarpaulin --workspace --exclude-files "ext1/*"

docs:
	cargo rustdoc
	pdflatex report.tex
	pdflatex report.tex

	

# Extension: build & run
ext1: ext1-clean ext1/target/release/spreadsheet
	./ext1/target/release/spreadsheet 999 18278

ext1/target/release/spreadsheet:
	cargo build --release --manifest-path ext1/Cargo.toml

ext1-clean:
	cargo clean --manifest-path ext1/Cargo.toml
