 #include "recalculation.h"
 #include <stdlib.h>
 
 /**
  * @brief Traverses an AVL tree of cell dependencies and collects affected cells
  *
  * This function performs an in-order traversal of the AVL tree of cell dependencies,
  * collecting cells that have not been visited yet. For each cell, it also traverses
  * its dependents to collect all cells in the dependency chain.
  *
  * @param sheet Pointer to the spreadsheet
  * @param node Current AVL tree node
  * @param visited Array tracking which cells have been visited
  * @param list Array to store the collected cells
  * @param list_count Pointer to the count of collected cells
  */
 void traverse_and_collect(Spreadsheet *sheet, AVLNode *node, bool *visited, CellIndex *list, int *list_count) {
     if (node == NULL) return;
     
     // In-order traversal: left subtree first
     traverse_and_collect(sheet, node->left, visited, list, list_count);
     
     // Process current node if not visited
     int idx = node->row * sheet->cols + node->col;
     if (!visited[idx]) {
         // Mark as visited and add to list
         visited[idx] = true;
         list[*list_count].row = node->row;
         list[*list_count].col = node->col;
         (*list_count)++;
         
         // Recursively collect dependents of this cell
         if (sheet->grid[node->row][node->col].dependents != NULL)
             traverse_and_collect(sheet, sheet->grid[node->row][node->col].dependents, visited, list, list_count);
     }
     
     // In-order traversal: right subtree last
     traverse_and_collect(sheet, node->right, visited, list, list_count);
 }
 
 /**
  * @brief Collects all cells affected by a change to a specific cell
  *
  * This function identifies and collects all cells that depend directly or indirectly
  * on the cell at the specified row and column.
  *
  * @param sheet Pointer to the spreadsheet
  * @param row Row of the changed cell
  * @param col Column of the changed cell
  * @param visited Array tracking which cells have been visited
  * @param list Array to store the collected cells
  * @param list_count Pointer to the count of collected cells
  */
 void collect_affected(Spreadsheet *sheet, short row, short col, bool *visited, CellIndex *list, int *list_count) {
     // Get the AVL tree of dependents for the cell
     AVLNode *node = sheet->grid[row][col].dependents;
     
     // If there are dependents, traverse and collect them
     if (node != NULL)
         traverse_and_collect(sheet, node, visited, list, list_count);
 }
 
 /**
  * @brief Computes the indegree (number of dependencies) for a cell based on an AVL tree node
  *
  * This function traverses the AVL tree of cell dependencies and computes
  * the indegree for a specific cell, which is the number of affected cells
  * that this cell depends on. It handles both individual cell dependencies
  * and range dependencies.
  *
  * @param sheet Pointer to the spreadsheet
  * @param node Current AVL tree node
  * @param r Row of the cell to compute indegree for
  * @param c Column of the cell to compute indegree for
  * @param affected Array indicating which cells are affected by the change
  * @param indegree Array to store the indegree values
  */
 void compute_indegree_avl(Spreadsheet *sheet, AVLNode *node, short r, short c, bool *affected, int *indegree) {
     if (node == NULL)
         return;
     
     // In-order traversal: left subtree first
     compute_indegree_avl(sheet, node->left, r, c, affected, indegree);
     
     // Process range dependencies
     if (node->is_range) {
         for (short i = node->row; i <= node->end_row && i < sheet->rows; i++) {
             for (short j = node->col; j <= node->end_col && j < sheet->cols; j++) {
                 // Check if the cell in the range is affected
                 int idx = i * sheet->cols + j;
                 if (affected[idx])
                     indegree[r * sheet->cols + c]++;
             }
         }
     } 
     // Process individual cell dependencies
     else {
         int dep_idx = node->row * sheet->cols + node->col;
         if (affected[dep_idx])
             indegree[r * sheet->cols + c]++;
     }
     
     // In-order traversal: right subtree last
     compute_indegree_avl(sheet, node->right, r, c, affected, indegree);
 }
 
 /**
  * @brief Computes the indegree for a specific cell
  *
  * This function computes the number of dependencies a specific cell has
  * on other affected cells.
  *
  * @param sheet Pointer to the spreadsheet
  * @param r Row of the cell
  * @param c Column of the cell
  * @param affected Array indicating which cells are affected by the change
  * @param indegree Array to store the indegree values
  */
 void compute_indegree_for_cell(Spreadsheet *sheet, short r, short c, bool *affected, int *indegree) {
     // Get the AVL tree of dependencies for the cell
     AVLNode *node = sheet->grid[r][c].depends_on;
     
     // Compute indegree based on this tree
     compute_indegree_avl(sheet, node, r, c, affected, indegree);
 }
 
 /**
  * @brief Updates the indegree of dependent cells after a cell is recalculated
  *
  * This function traverses the AVL tree of dependent cells and decrements their
  * indegree values. If a cell's indegree becomes zero, it is added to the queue
  * for recalculation.
  *
  * @param sheet Pointer to the spreadsheet
  * @param node Current AVL tree node
  * @param affected Array indicating which cells are affected by the change
  * @param indegree Array storing the indegree values
  * @param queue Queue of cells ready for recalculation
  * @param queue_back Pointer to the back index of the queue
  */
 void update_dependents_avl(Spreadsheet *sheet, AVLNode *node, bool *affected, int *indegree, CellIndex *queue, int *queue_back) {
     if (node == NULL)
         return;
     
     // In-order traversal: left subtree first
     update_dependents_avl(sheet, node->left, affected, indegree, queue, queue_back);
     
     // Process range dependencies
     if (node->is_range) {
         // Ensure range bounds are within the sheet dimensions
         short rStart =node->row;
         short rEnd =node->end_row;
         short cStart =node->col;
         short cEnd = node->end_col;
         
         // Process each cell in the range
         for (short i = rStart; i <= rEnd; i++) {
             for (short j = cStart; j <= cEnd; j++) {
                 int idx = i * sheet->cols + j;
                 // If the cell is affected by the change
                 if (affected[idx]) {
                     // Decrement indegree and add to queue if zero
                     indegree[idx]--;   
                     if (indegree[idx] == 0) {
                         queue[*queue_back].row = i;
                         queue[*queue_back].col = j;
                         (*queue_back)++;
                     }
                 }
             }
         }
     } 
     // Process individual cell dependencies
     else {
         int idx = node->row * sheet->cols + node->col;
         if (affected[idx]) {
             // Decrement indegree and add to queue if zero
             indegree[idx]--;
             if (indegree[idx] == 0) {
                 queue[*queue_back].row = node->row;
                 queue[*queue_back].col = node->col;
                 (*queue_back)++;
             }
         }
     }
     
     // In-order traversal: right subtree last
     update_dependents_avl(sheet, node->right, affected, indegree, queue, queue_back);
 }
 
 /**
  * @brief Updates the indegree of all cells dependent on a specific cell
  *
  * This function decrements the indegree of all cells that depend on
  * the cell at the specified row and column.
  *
  * @param sheet Pointer to the spreadsheet
  * @param r Row of the cell
  * @param c Column of the cell
  * @param affected Array indicating which cells are affected by the change
  * @param indegree Array storing the indegree values
  * @param queue Queue of cells ready for recalculation
  * @param queue_back Pointer to the back index of the queue
  */
 void update_dependents_indegree(Spreadsheet *sheet, short r, short c, bool *affected, int *indegree, CellIndex *queue, int *queue_back) {
     // Get the AVL tree of dependents for the cell
     AVLNode *node = sheet->grid[r][c].dependents;
     
     // Update indegree for all dependents
     update_dependents_avl(sheet, node, affected, indegree, queue, queue_back);
 }
 
 /**
  * @brief Recalculates cells affected by a change using topological sorting
  *
  * This function implements a topological sort algorithm to recalculate cells
  * in the correct dependency order after a cell changes. It ensures that a cell
  * is only recalculated after all its dependencies have been updated.
  *
  * @param sheet Pointer to the spreadsheet
  * @param changed_row Row of the changed cell
  * @param changed_col Column of the changed cell
  */
 void recalc_topological(Spreadsheet *sheet, short changed_row, short changed_col) {
     int total_cells = sheet->rows * sheet->cols;
     
     // Allocate memory for tracking affected cells
     bool *affected = calloc(total_cells, sizeof(bool));
     CellIndex *list = malloc(total_cells * sizeof(CellIndex));
     int list_count = 0;
     
     // Collect all cells affected by the change
     collect_affected(sheet, changed_row, changed_col, affected, list, &list_count);
     
     // Compute indegree (dependency count) for each affected cell
     int *indegree = calloc(total_cells, sizeof(int));
     for (int i = 0; i < list_count; i++) {
         int r = list[i].row;
         int c = list[i].col;
         compute_indegree_for_cell(sheet, r, c, affected, indegree);
     }
     
     // Initialize queue for topological sort
     CellIndex *queue = malloc(total_cells * sizeof(CellIndex));
     int queue_front = 0, queue_back = 0;
     
     // Add cells with zero indegree to the queue
     for (int i = 0; i < list_count; i++) {
         int idx = list[i].row * sheet->cols + list[i].col;
         if (indegree[idx] == 0)
             queue[queue_back++] = list[i];
     }
     
     // Process queue until empty (topological sort)
     while (queue_front < queue_back) {
         // Get next cell to recalculate
         CellIndex current = queue[queue_front++];
         
         // Recalculate the cell
         recalculate_cell(sheet, current.row, current.col);
         
         // Update indegree of dependent cells
         update_dependents_indegree(sheet, current.row, current.col, affected, indegree, queue, &queue_back);
     }
     
     // Free allocated memory
     free(affected);
     free(list);
     free(indegree);
     free(queue);
 }
 
 /**
  * @brief Recalculates a single cell's value
  *
  * This function evaluates the expression in a cell and updates its value.
  * It uses a visited flag to prevent infinite recursion in case of circular references.
  *
  * @param sheet Pointer to the spreadsheet
  * @param row Row of the cell to recalculate
  * @param col Column of the cell to recalculate
  */
 void recalculate_cell(Spreadsheet *sheet, short row, short col) {
     int index = row * sheet->cols + col;
     
     // Prevent infinite recursion
     if (recalcVisited[index])
         return;
     
     // Mark as visited
     recalcVisited[index] = 1;
     
     // Get the cell and reset error flag
     Cell *cell = &sheet->grid[row][col];
     cell->error = false;
     
     // Evaluate the expression and update the cell value
     int new_value = evaluateExpression(sheet, cell->expr, row, col);
     cell->value = new_value;
     
     // Mark as unvisited
     recalcVisited[index] = 0;
 }