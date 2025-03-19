#ifndef SPREADSHEET_H
#define SPREADSHEET_H

#include <stdbool.h>
#include <time.h>
#include "avl.h"
#include "expr.h"

/* Global variables (defined in spreadsheet.c) */
extern short gl_rows;
extern short gl_cols;
extern bool *recalcVisited;
extern bool commandError;

typedef struct Expression Expression;

/* Cell structure */
typedef struct Cell {
    int value;
    Expression *expr;
    short row_cell;
    short col_cell;
    bool error;
    AVLNode *dependents;
    AVLNode *depends_on;
} Cell;

/* Spreadsheet structure */
typedef struct Spreadsheet {
    short rows;
    short cols;
    Cell **grid;
    short scroll_row;
    short scroll_col;
} Spreadsheet;

/* For dependency backup */
typedef struct {
    AVLNode* depends_on_backup;
} DependencyBackup;

/* Type for holding cell indices */
typedef struct {
    short row;
    short col;
} CellIndex;

/* Function prototypes */
Spreadsheet *initSpreadsheet(short rows, short cols);
void freeSpreadsheet(Spreadsheet *sheet);
void displaySpreadsheet(Spreadsheet *sheet);
int scrollSpreadsheet(Spreadsheet *sheet, char direction);
int cellStringToIndices(const char *cellStr, int *row, int *col);
void scrollTo(Spreadsheet *sheet, const char *cellStr);
void cellformula_to_indices(const char *cellStr, short *row, short *col);
void setCell(Spreadsheet *sheet, char input[]);

#endif // SPREADSHEET_H
