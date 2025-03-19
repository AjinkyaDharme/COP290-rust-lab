 #include "expr.h"
 #include "dfs.h"
 #include "avl.h"
 #include "spreadsheet.h"   
 #include <stdlib.h>
 #include <ctype.h>
 #include <string.h>
 #include <stdio.h>
 #include <math.h>
 #include <unistd.h>
 #include <limits.h>        
 
 
 // Global pointer used during compilation to track current position in formula string
 static char *comp_ptr;
 
 // Forward declarations for recursive descent parser functions
 static Expression* compile_expr(Spreadsheet *sheet, short curr_row, short curr_col);
 static Expression* compile_adv(Spreadsheet *sheet, short curr_row, short curr_col);
 static bool compile_cellRef(short *row, short *col);
 
 /**
  * @brief Compiles a formula string into an Expression tree
  *
  * @param sheet Pointer to the spreadsheet
  * @param formula The formula string to compile
  * @param curr_row Current cell's row
  * @param curr_col Current cell's column
  * @return Pointer to the compiled Expression tree or NULL if compilation failed
  */
 Expression* compileExpression(Spreadsheet *sheet, const char *formula, short curr_row, short curr_col) {
     comp_ptr = (char *)formula;
     Expression *expr = compile_expr(sheet, curr_row, curr_col);
     
     // Check if the entire formula was consumed
     if (*comp_ptr != '\0') {
         commandError = true;
         freeExpression(expr); 
         return NULL;
     }
     return expr;
 }
 
 /**
  * @brief Compiles an expression, which may be a binary operation or a factor
  *
  * This function handles the parsing of expressions with binary operators (+, -, *, /).
  * It first parses the left operand, then checks for an operator, then parses the right operand.
  *
  * @param sheet Pointer to the spreadsheet
  * @param curr_row Current cell's row
  * @param curr_col Current cell's column
  * @return Pointer to Expression tree node or NULL if parsing failed
  */
 static Expression* compile_expr(Spreadsheet *sheet, short curr_row, short curr_col) {
     // Parse the left operand
     Expression *left = compile_adv(sheet, curr_row, curr_col);
     if (!left) {
        return NULL;
     }
 
     // If we reached the end of the string, return the left operand
     if (*comp_ptr == '\0')
         return left;
 
     // This is not the end of RHS and LHS comes out to be function so we flag commandError
     if (left->type == EXPR_FUNCTION) {
          if (*comp_ptr != '\0') {
              commandError = true;
              freeExpression(left);
              return NULL;
          }
          return left;
     }
     
     // Check for valid binary operator
     if (*comp_ptr != '+' && *comp_ptr != '-' && *comp_ptr != '*' && *comp_ptr != '/') {
          commandError = true;
          freeExpression(left);
          return NULL;
     }
     
     // Save the operator and move past it
     char op = *comp_ptr;
     comp_ptr++;
 
     // Parse the right operand
     Expression *right = compile_adv(sheet, curr_row, curr_col);
     if (!right) {
          freeExpression(left);
          return NULL;
     }
 
     // Function expressions can't be the right operand of a binary expression
     if (right->type == EXPR_FUNCTION) {
         commandError = true;
         freeExpression(left);
         freeExpression(right);
         return NULL;
     }
     
     // There should be nothing after a complete expression
     if (*comp_ptr != '\0') {
         commandError = true;
         freeExpression(left);
         freeExpression(right);
         return NULL;
     }
     
     // Create a binary expression node
     Expression *node = malloc(sizeof(Expression));
     node->type = EXPR_BINARY;
     node->data.binary.op = op;
     node->data.binary.left = left;
     node->data.binary.right = right;
     return node;
 }
 
 /**
  * @brief Compiles a factor, which may be a constant, cell reference, or function call
  *
  * This function handles unary plus/minus signs, numeric constants, cell references,
  * and function calls like MIN, MAX, SUM, AVG, STDEV, and SLEEP.
  *
  * @param sheet Pointer to the spreadsheet
  * @param curr_row Current cell's row
  * @param curr_col Current cell's column
  * @return Pointer to Expression tree node or NULL if parsing failed
  */
 static Expression* compile_adv(Spreadsheet *sheet, short curr_row, short curr_col) {
     // Handle leading +/- signs
     int sign = 1;
     while (*comp_ptr == '+' || *comp_ptr == '-') {
         if (*comp_ptr == '-')
             sign = -sign;
         comp_ptr++;
     }
     
     // Cell references can't have a negative sign
     if (sign < 0 && isalpha(*comp_ptr)) {
         commandError= true;
         return NULL;
     }
     
     // Handle numeric constants
     if (isdigit(*comp_ptr)) {
         int value = 0;
         while (isdigit(*comp_ptr)) {
             value = value * 10 + (*comp_ptr - '0');
             comp_ptr++;
         }
         Expression *node = malloc(sizeof(Expression));
         node->type = EXPR_CONSTANT;
         node->data.constant = sign * value;
         return node;
     }
     
     // Handle cell references and function calls
     if (isalpha(*comp_ptr)) {
         char id[10] = {0};
         int idLen = 0;
         char *start = comp_ptr;
         
         // Extract the identifier
         while (isalpha(*comp_ptr)) {
             if (idLen < 9)
                 id[idLen++] = *comp_ptr;
             comp_ptr++;
         }
         id[idLen] = '\0';
         
         // Check if it's a function call (has parentheses)
         if (*comp_ptr == '(') {
             comp_ptr++; // skip '('
             Expression *node = malloc(sizeof(Expression));
             node->type = EXPR_FUNCTION;
             
             // Handle range functions (MIN, MAX, SUM, AVG, STDEV)
             if (strcmp(id, "MIN") == 0 || strcmp(id, "MAX") == 0 ||
                 strcmp(id, "SUM") == 0 || strcmp(id, "AVG") == 0 ||
                 strcmp(id, "STDEV") == 0) {
                 // Determine function type
                 FunctionType ftype;
                 if (strcmp(id, "MIN") == 0) ftype = FUNC_MIN;
                 else if (strcmp(id, "MAX") == 0) ftype = FUNC_MAX;
                 else if (strcmp(id, "SUM") == 0) ftype = FUNC_SUM;
                 else if (strcmp(id, "AVG") == 0) ftype = FUNC_AVG;
                 else ftype = FUNC_STDEV;
                 node->data.func.funcType = ftype;
                 
                 // Parse range arguments (e.g., A1:B5)
                 short startRow, startCol, endRow, endCol;
                 if (!compile_cellRef(&startRow, &startCol)) { commandError=true; free(node); return NULL; }
                 if (*comp_ptr != ':') { commandError=true; free(node); return NULL; }
                 comp_ptr++; // skip ':'
                 if (!compile_cellRef(&endRow, &endCol)) { commandError=true; free(node); return NULL; }
                 if (*comp_ptr != ')') { commandError=true; free(node); return NULL; }
                 comp_ptr++; // skip ')'
                 
                 // Ensure range is valid (start <= end)
                 if (startRow > endRow || startCol > endCol) { commandError=true; free(node); return NULL; }
                 
                 // Set up cell dependencies for all cells in the range
                 for (short i = startRow; i <= endRow && i < sheet->rows; i++) {
                     for (short j = startCol; j <= endCol && j < sheet->cols; j++) {
                         add_dependent(&sheet->grid[i][j], curr_row, curr_col);
                     }
                 }
                 add_range_depends_on(&sheet->grid[curr_row][curr_col], startRow, startCol, endRow, endCol);
                 
                 // Store range information in node
                 node->data.func.args.range.startRow = startRow;
                 node->data.func.args.range.startCol = startCol;
                 node->data.func.args.range.endRow = endRow;
                 node->data.func.args.range.endCol = endCol;
                 
                 // Check for circular dependencies
                 if (check_dfs_cycle(sheet, curr_row, curr_col)) {
                     // Remove dependencies if cycle is detected
                     remove_range_depends_on(&sheet->grid[curr_row][curr_col], startRow, startCol, endRow, endCol);
                     for (short i = startRow; i <= endRow && i < sheet->rows; i++) {
                         for (short j = startCol; j <= endCol && j < sheet->cols; j++) {
                             remove_dependent(&sheet->grid[i][j], curr_row, curr_col);
                         }
                     }
                     free(node);
                     commandError = true;
                     return NULL;
                 }
                 return node;
             } 
             // Handle SLEEP function
             else if (strcmp(id, "SLEEP") == 0) {
                 node->data.func.funcType = FUNC_SLEEP;
                 Expression *arg = compile_adv(sheet, curr_row, curr_col);
                 if (commandError) { free(node); return NULL; }
                 if (*comp_ptr != ')') { commandError=true; free(node); return NULL; }
                 comp_ptr++; // skip ')'
                 node->data.func.args.arg = arg;
                 return node;
             } 
             // Unknown function
             else {
                commandError = true;
                 free(node);
                 return NULL;
             }
         }  
         // Handle cell reference   A1=B1
         else {
             comp_ptr = start;
             short cellRow, cellCol;
             if (!compile_cellRef(&cellRow, &cellCol)) { commandError=true; return NULL; }
             
             // Set up cell dependency tracking
             add_dependent(&sheet->grid[cellRow][cellCol], curr_row, curr_col);
             add_depends_on(&sheet->grid[curr_row][curr_col], cellRow, cellCol);
             
             // Check for circular dependencies
             if (check_dfs_cycle(sheet, curr_row, curr_col)) {
                 remove_dependent(&sheet->grid[cellRow][cellCol], curr_row, curr_col);
                 remove_depends_on(&sheet->grid[curr_row][curr_col], cellRow, cellCol);
                 commandError = true;
                 return NULL;
             }
             
             // Create cell reference node
             Expression *node = malloc(sizeof(Expression));
             node->type = EXPR_CELL;
             node->data.cell.row = cellRow;
             node->data.cell.col = cellCol;
             return node;
         }
     }
     
     // Invalid expression
     commandError = true;
     return NULL;
 }
 
 /**
  * @brief Parses a cell reference in the format of A1, B2, etc.
  *
  * @param row Pointer to store the parsed row index (0-based)
  * @param col Pointer to store the parsed column index (0-based)
  * @return true if parsing succeeded, false otherwise
  */
 static bool compile_cellRef(short *row, short *col) {
     // Cell references start with column letters
     if (!isalpha(*comp_ptr))
         return false;
     
     // Parse column (A=0, B=1, ..., Z=25, AA=26, etc.)
     *col = 0;
     while (isalpha(*comp_ptr)) {
         *col = *col * 26 + (toupper(*comp_ptr) - 'A' + 1);
         comp_ptr++;
     }
     *col = (*col) - 1;
     
     // Next should be row numbers
     if (!isdigit(*comp_ptr))
         return false;
     
     // Parse row (1-based in input, converted to 0-based)
     *row = 0;
     while (isdigit(*comp_ptr)) {
         *row = *row * 10 + (*comp_ptr - '0');
         comp_ptr++;
     }
     *row = (*row) - 1;
     
     // Check if cell reference is within bounds
     if (*row < 0 || *col < 0 || *row >= gl_rows || *col >= gl_cols){
         commandError = true;
         return false;
     }
     return true;
 }
 
 /**
  * @brief Recursively frees memory used by an Expression tree
  *
  * @param expr Pointer to the Expression tree to free
  */
 void freeExpression(Expression *expr) {
     if (!expr) return;
     
     // Free child nodes first based on expression type
     if (expr->type == EXPR_BINARY) {
         freeExpression(expr->data.binary.left);
         freeExpression(expr->data.binary.right);
     } else if (expr->type == EXPR_FUNCTION) {
         if (expr->data.func.funcType == FUNC_SLEEP)
             freeExpression(expr->data.func.args.arg);
     }
     
     // Free the node itself
     free(expr);
 }
 
 /**
  * @brief Evaluates an expression tree and returns the result
  *
  * This function recursively evaluates the expression tree, handling constants,
  * cell references, binary operations, and functions. All combinations handled successfully.
  *
  * @param sheet Pointer to the spreadsheet
  * @param expr Pointer to the Expression tree to evaluate
  * @param curr_row Current cell's row
  * @param curr_col Current cell's column
  * @return The evaluated value of the expression
  */
 int evaluateExpression(Spreadsheet *sheet, Expression *expr, short curr_row, short curr_col) {
     if (!expr) return 0;
     
     // Evaluate based on expression type
     switch(expr->type) {
         // Constant expressions just return their value
         case EXPR_CONSTANT:
             return expr->data.constant;
             
         // Cell references evaluate to the value of the referenced cell
         case EXPR_CELL: {
             short r = expr->data.cell.row, c = expr->data.cell.col;
             // Propagate errors from referenced cells
             if (sheet->grid[r][c].error) {
                 sheet->grid[curr_row][curr_col].error = true;
                 return 0;
             }
             return sheet->grid[r][c].value;
         }
         
         // Binary operations evaluate both sides and apply the operator
         case EXPR_BINARY: {
             int left = evaluateExpression(sheet, expr->data.binary.left, curr_row, curr_col);
             int right = evaluateExpression(sheet, expr->data.binary.right, curr_row, curr_col);
             
             // Check if an error occurred during evaluation
             if (sheet->grid[curr_row][curr_col].error)
                 return 0;
                 
             // Apply the appropriate operation
             switch(expr->data.binary.op) {
                 case '+': return left + right;
                 case '-': return left - right;
                 case '*': return left * right;
                 case '/': 
                     // Handle division by zero
                     if (right == 0) { 
                         sheet->grid[curr_row][curr_col].error = true; 
                         return 0; 
                     }
                     return left / right;
             }
             break;
         }
         
         // Function calls evaluate differently based on function type
         case EXPR_FUNCTION: {
             FunctionType ft = expr->data.func.funcType;
             
             // Handle SLEEP function
             if (ft == FUNC_SLEEP) {
                 int seconds = evaluateExpression(sheet, expr->data.func.args.arg, curr_row, curr_col);
                 // Sleep for the specified number of seconds if non-negative
                 if (seconds >= 0)
                     sleep(seconds);
                 return seconds;
             } 
             // Handle range functions (MIN, MAX, SUM, AVG, STDEV)
             else {
                 short r1 = expr->data.func.args.range.startRow;
                 short c1 = expr->data.func.args.range.startCol;
                 short r2 = expr->data.func.args.range.endRow;
                 short c2 = expr->data.func.args.range.endCol;
                 
                 // Validate range bounds
                 if (r1 < 0 || c1 < 0 || r2 >= sheet->rows || c2 >= sheet->cols) {
                     commandError=1;
                     return 0;
                 }
                 
                 // Compute values needed for all functions
                 int sum = 0, count = 0;
                 int minv = INT_MAX, maxv = INT_MIN;
                 
                 // Process each cell in the range
                 for (short i = r1; i <= r2 && i < sheet->rows; i++) {
                     for (short j = c1; j <= c2 && j < sheet->cols; j++) {
                         // Propagate errors from cells in the range
                         if (sheet->grid[i][j].error) {
                             sheet->grid[curr_row][curr_col].error = true;
                             return 0;
                         }
                         
                         // Collect statistics from the cell
                         int v = sheet->grid[i][j].value;
                         sum += v;
                         if (v < minv) minv = v;
                         if (v > maxv) maxv = v;
                         count++;
                     }
                 }
                 
                 // Return appropriate result based on function type
                 switch(ft) {
                     case FUNC_MIN: return minv;
                     case FUNC_MAX: return maxv;
                     case FUNC_SUM: return sum;
                     case FUNC_AVG: return (count ? (sum / count) : 0);
                     case FUNC_STDEV: {
                         // Calculate standard deviation
                         if (count <= 1) return 0;
                         int mean = sum / count;
                         double variance = 0.0;
                         
                         // Calculate sum of squared differences from mean
                         for (short i = r1; i <= r2 && i < sheet->rows; i++) {
                             for (short j = c1; j <= c2 && j < sheet->cols; j++) {
                                 int v = sheet->grid[i][j].value;
                                 variance += (v - mean) * (v - mean);
                             }
                         }
                         
                         // Calculate variance and standard deviation
                         variance /= count;
                         return (int)round(sqrt(variance));
                     }
                     default: return 0;
                 }
             }
         }
     }
     return 0;
 }