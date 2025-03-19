CC = gcc
CFLAGS = -Wall -Wextra -std=c99 -D_POSIX_C_SOURCE=199309L -O3
OBJ_DIR = target/release
OBJS = main.o avl.o dfs.o expr.o spreadsheet.o recalculation.o

all: $(OBJ_DIR)/spreadsheet

$(OBJ_DIR)/spreadsheet: $(OBJS)
	mkdir -p $(OBJ_DIR)
	$(CC) $(CFLAGS) -o $(OBJ_DIR)/spreadsheet $(OBJS) -lm -lrt


%.o: %.c
	$(CC) $(CFLAGS) -c $< -o $@

clean:
	rm -f $(OBJS) $(OBJ_DIR)/spreadsheet tester report.pdf *.aux *.log

test: all
	$(CC) $(CFLAGS) tester.c -o tester -lm -lrt
	./tester

report:
	pdflatex report.tex
	pdflatex report.tex