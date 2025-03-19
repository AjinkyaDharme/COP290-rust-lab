#include "spreadsheet.h"
#include "avl.h"
#include "expr.h"
#include "recalculation.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <ctype.h>
#include <time.h>

/* Global variables */
short gl_rows = 0, gl_cols = 0;             /* Global row and column counts */
bool *recalcVisited = NULL;                 /* Tracks visited cells during recalculation */
bool commandError = false;                  /* Flag to indicate errors in command processing */

/**
 * Initialize a new spreadsheet with the specified dimensions
 * 
 * @param rows Number of rows in the spreadsheet
 * @param cols Number of columns in the spreadsheet
 * @return Pointer to the newly created Spreadsheet structure
 */
Spreadsheet *initSpreadsheet(short rows, short cols) {
    Spreadsheet *sheet = (Spreadsheet *)malloc(sizeof(Spreadsheet));
    sheet->rows = rows;
    sheet->cols = cols;
    sheet->scroll_row = 0;                  /* Initialize viewing position to top-left */
    sheet->scroll_col = 0;
    
    /* Allocate memory for the grid of cells */
    sheet->grid = (Cell **)malloc(rows * sizeof(Cell *));
    for (short i = 0; i < rows; i++) {
        sheet->grid[i] = (Cell *)malloc(cols * sizeof(Cell));
        
        /* Initialize each cell with default values */
        for (short j = 0; j < cols; j++) {
            sheet->grid[i][j].value = 0;    /* Default value is 0 */
            sheet->grid[i][j].row_cell = i; /* Store cell's position */
            sheet->grid[i][j].col_cell = j;
            sheet->grid[i][j].expr = NULL;  /* No expression by default */
            sheet->grid[i][j].error = false;
            sheet->grid[i][j].dependents = NULL; /* No dependency trees initially */
            sheet->grid[i][j].depends_on = NULL;
        }
    }
    return sheet;
}

/**
 * Free all memory associated with a spreadsheet
 * 
 * @param sheet Pointer to the Spreadsheet to be freed
 */
void freeSpreadsheet(Spreadsheet *sheet) {
    /* Free all cells and their associated data */
    for (short i = 0; i < sheet->rows; i++) {
        for (short j = 0; j < sheet->cols; j++) {
            /* Free the dependency trees if they exist */
            if (sheet->grid[i][j].dependents != NULL)
                freeAVL(sheet->grid[i][j].dependents);
            if (sheet->grid[i][j].depends_on != NULL)
                freeAVL(sheet->grid[i][j].depends_on);
            
            /* Free the expression if it exists */
            if (sheet->grid[i][j].expr != NULL)
                freeExpression(sheet->grid[i][j].expr);
        }
        /* Free the row of cells */
        free(sheet->grid[i]);
    }
    /* Free the grid and the spreadsheet structure */
    free(sheet->grid);
    free(sheet);
}

/**
 * Display the current view of the spreadsheet
 * Shows a 10x10 window based on current scroll position
 * 
 * @param sheet Pointer to the Spreadsheet to display
 */
void displaySpreadsheet(Spreadsheet *sheet) {
    /* Print column headers (A, B, C, ...) */
    printf("\t");
    for (short c = sheet->scroll_col; c < sheet->scroll_col + 10 && c < sheet->cols; c++) {
        short tmp = c;
        char colName[10] = {0};
        short index = 0;
        short n = tmp;
        char buf[10];
        short pos = 0;
        
        /* Convert column number to Excel-style column name (A, B, ..., Z, AA, AB, ...) */
        while (n >= 0) {
            buf[pos++] = 'A' + (n % 26);
            n = n / 26 - 1;
            if (n < 0)
                break;
        }
        for (short i = pos - 1; i >= 0; i--)
            colName[index++] = buf[i];
        colName[index] = '\0';
        
        printf("%s\t", colName);
    }
    printf("\n");
    
    /* Display each row with its row number */
    for (int r = sheet->scroll_row; r < sheet->scroll_row + 10 && r < sheet->rows; r++) {
        printf("%d\t", r + 1);  /* Display 1-based row numbers */
        
        /* Display each cell in the current view */
        for (int c = sheet->scroll_col; c < sheet->scroll_col + 10 && c < sheet->cols; c++) {
            if (sheet->grid[r][c].error)
                printf("ERR\t");  /* Display ERR for cells with errors */
            else
                printf("%d\t", sheet->grid[r][c].value);  /* Display cell value */
        }
        printf("\n");
    }
}

/**
 * Scroll the spreadsheet view in the specified direction
 * 
 * @param sheet Pointer to the Spreadsheet
 * @param direction Direction to scroll ('w'=up, 's'=down, 'a'=left, 'd'=right)
 * @return 0 on success, 1 on error
 */
int scrollSpreadsheet(Spreadsheet *sheet, char direction) {
    if (direction == 'w') {  /* Scroll up */
        if (sheet->scroll_row >= 10)
            sheet->scroll_row -= 10;
        else
            sheet->scroll_row = 0;
    } else if (direction == 's') {  /* Scroll down */
        if (sheet->scroll_row + 10 < sheet->rows)
            sheet->scroll_row += 10;
    } else if (direction == 'a') {  /* Scroll left */
        if (sheet->scroll_col >= 10)
            sheet->scroll_col -= 10;
        else
            sheet->scroll_col = 0;
    } else if (direction == 'd') {  /* Scroll right */
        if (sheet->scroll_col + 10 < sheet->cols)
            sheet->scroll_col += 10;
    } else {
        commandError = true;
        return 1;  /* Invalid direction */
    }
    return 0;
}

/**
 * Convert a cell reference string (e.g., "A1") to row and column indices
 * 
 * @param cellStr Cell reference string (e.g., "A1", "BC23")
 * @param row Pointer to store the parsed row index
 * @param col Pointer to store the parsed column index
 * @return 1 on success, 0 on failure
 */
int cellStringToIndices(const char *cellStr, int *row, int *col) {
    if (cellStr == NULL || !isalpha(cellStr[0]))
        return 0;  /* Invalid cell reference */
        
    int i = 0;
    *col = 0;
    
    /* Parse column letters (A=0, B=1, ..., Z=25, AA=26, ...) */
    while (cellStr[i] && isalpha(cellStr[i])) {
        char ch = (char)toupper(cellStr[i]);
        *col = *col * 26 + (ch - 'A' + 1);
        i++;
    }
    
    if (i == 0)
        return 0;  /* No column letters found */
        
    *col = (*col) - 1;  /* Convert to 0-based indexing */
    
    if (!isdigit(cellStr[i]))
        return 0;  /* No row number found */
        
    /* Parse row number */
    *row = 0;
    while (cellStr[i] && isdigit(cellStr[i])) {
        *row = *row * 10 + (cellStr[i] - '0');
        i++;
    }
    
    *row -= 1;  /* Convert to 0-based indexing */
    
    /* Check if we consumed the entire string */
    if (cellStr[i] != '\0')
        return 0;  /* Extra characters found */
        
    return 1;  /* Successful parsing */
}

/**
 * Scroll the spreadsheet view to center on a specific cell
 * 
 * @param sheet Pointer to the Spreadsheet
 * @param cellStr Cell reference string (e.g., "A1")
 */
void scrollTo(Spreadsheet *sheet, const char *cellStr) {
    int row, col;
    /* Parse the cell reference and validate it's within bounds */
    if (!cellStringToIndices(cellStr, &row, &col) || 
        row < 0 || row >= sheet->rows || 
        col < 0 || col >= sheet->cols) {
        commandError = true;
        return;
    }
    
    /* Set the scroll position to the specified cell */
    sheet->scroll_row = row;
    sheet->scroll_col = col;
}

/**
 * Convert a cell reference string to row and column indices for formula processing
 * Sets commandError flag if parsing fails
 * 
 * @param cellStr Cell reference string (e.g., "A1")
 * @param row Pointer to store the parsed row index
 * @param col Pointer to store the parsed column index
 */
void cellformula_to_indices(const char *cellStr, short *row, short *col) {
    short i = 0;
    *col = 0;
    
    /* Validate input */
    if (!cellStr || !cellStr[0]) { commandError = true; return; }
    
    /* Parse column letters */
    while (cellStr[i] && isalpha(cellStr[i])) {
        *col = (*col) * 26 + (cellStr[i] - 'A' + 1);
        i++;
    }
    
    if (i == 0) { commandError = true; return; }  /* No column letters found */
    
    *col = (*col) - 1;  /* Convert to 0-based indexing */
    
    if (!isdigit(cellStr[i])) { commandError = true; return; }  /* No row number found */
    
    /* Parse row number */
    *row = atoi(&cellStr[i]) - 1;  /* Convert to 0-based indexing */
    
    /* Validate cell coordinates are within bounds */
    if (*row < 0 || *col < 0 || *row >= gl_rows || *col >= gl_cols) { 
        commandError = true; 
    }
    
    /* Validate all remaining characters are digits */
    while (cellStr[i]) {
        if (!isdigit(cellStr[i])) { commandError = true; return; }
        i++;
    }
}

/**
 * Set a cell's formula and update dependencies
 * 
 * @param sheet Pointer to the Spreadsheet
 * @param input Input string in format "CELL=FORMULA"
 */
void setCell(Spreadsheet *sheet, char input[]) {
    /* Reset recalculation tracking array */
    memset(recalcVisited, 0, sheet->rows * sheet->cols * sizeof(bool));
    
    /* Find the equals sign that separates cell reference from formula */
    char *eq = strchr(input, '=');
    if (!eq) { commandError = true; return; }  /* No equals sign found */
    
    *eq = '\0';  /* Split the string */
    
    short row, col;
    /* Parse the cell reference */
    cellformula_to_indices(input, &row, &col);
    if (commandError) return;
    
    Cell *cell = &sheet->grid[row][col];
    
    /* Backup current cell state in case formula compilation fails */
    int backupValue = cell->value;
    bool backupError = cell->error;
    Expression *backupExpr = cell->expr;
    
    DependencyBackup depBackup;
    depBackup.depends_on_backup = copyAVL(cell->depends_on);
    
    /* Clear current dependencies */
    clear_dependencies(cell);
    
    /* Compile the new expression */
    Expression *newExpr = compileExpression(sheet, eq + 1, row, col);
    
    /* If there was an error, restore the cell to its previous state */
    if (commandError) {
        cell->value = backupValue;
        cell->error = backupError;
        cell->expr = backupExpr;
        if (cell->depends_on) freeAVL(cell->depends_on);
        cell->depends_on = depBackup.depends_on_backup;
        return;
    }
    
    /* Update the cell with the new expression */
    cell->expr = newExpr;
    cell->error = false;
    
    /* Evaluate the new expression */
    int result = evaluateExpression(sheet, cell->expr, row, col);
    cell->value = result;
    
    /* Propagate changes to dependent cells */
    recalc_topological(sheet, row, col);
    
    /* Free the backup expression if it exists */
    if (backupExpr)
        freeExpression(backupExpr);
}