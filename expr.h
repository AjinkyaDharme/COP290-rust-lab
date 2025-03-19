#ifndef EXPR_H
#define EXPR_H

/* Forward declaration for Spreadsheet */
typedef struct Spreadsheet Spreadsheet;

#include <stdbool.h>

/* Expression type definitions */
typedef enum {
    EXPR_CONSTANT,
    EXPR_CELL,
    EXPR_BINARY,
    EXPR_FUNCTION
} ExprType;

typedef enum {
    FUNC_MIN,
    FUNC_MAX,
    FUNC_AVG,
    FUNC_SUM,
    FUNC_STDEV,
    FUNC_SLEEP
} FunctionType;

typedef struct Expression {
    ExprType type;
    union {
        int constant;
        struct { short row, col; } cell;
        struct { 
            char op; 
            struct Expression *left, *right;
        } binary;
        struct {
            FunctionType funcType;
            union {
                struct { short startRow, startCol, endRow, endCol; } range;
                struct Expression *arg;
            } args;
        } func;
    } data;
} Expression;

Expression* compileExpression(Spreadsheet *sheet, const char *formula, short curr_row, short curr_col);
void freeExpression(Expression *expr);
int evaluateExpression(Spreadsheet *sheet, Expression *expr, short curr_row, short curr_col);

#endif // EXPR_H
