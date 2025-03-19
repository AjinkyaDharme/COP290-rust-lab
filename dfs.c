#include "dfs.h"
#include <stdlib.h>

/**
 * Helper function for cycle detection using DFS algorithm
 * 
 * @param sheet Pointer to the spreadsheet structure
 * @param cur_row Current row being processed
 * @param cur_col Current column being processed
 * @param visited Array tracking visited cells
 * @param recStack Array tracking cells in current recursion stack
 * @return true if cycle is detected, false otherwise
 */
bool check_dfs_cycle_util(Spreadsheet *sheet, short cur_row, short cur_col, bool *visited, bool *recStack) {
    // Calculate linear index for the current cell
    int index = cur_row * sheet->cols + cur_col;
    
    // If cell is already in recursion stack, we found a cycle
    if (recStack[index])
        return true;
    
    // If cell was already visited but not in recursion stack, no cycle here
    if (visited[index])
        return false;
    
    // Mark current cell as visited and add to recursion stack
    visited[index] = true;
    recStack[index] = true;
    
    // Get the current cell and check all its dependencies for cycles
    struct Cell *curCell = &sheet->grid[cur_row][cur_col];
    if (check_cycle_in_avl(sheet, curCell->depends_on, visited, recStack))
        return true;
    
    // Remove current cell from recursion stack as we're done with it
    recStack[index] = false;
    return false;
}

/**
 * Recursively checks for cycles in an AVL tree of dependencies
 * 
 * @param sheet Pointer to the spreadsheet structure
 * @param node Current AVL node being processed
 * @param visited Array tracking visited cells
 * @param recStack Array tracking cells in current recursion stack
 * @return true if cycle is detected, false otherwise
 */
bool check_cycle_in_avl(Spreadsheet *sheet, AVLNode* node, bool *visited, bool *recStack) {
    // Base case: empty tree has no cycles
    if (node == NULL)
        return false;
    
    // Check for cycles in left subtree
    if (check_cycle_in_avl(sheet, node->left, visited, recStack))
        return true;
    
    // Process current node
    if (node->is_range) {
        // Handle cell range: adjust boundaries to be within sheet dimensions
        short rStart = node->row;
        short rEnd =  node->end_row;
        short cStart = node->col;
        short cEnd = node->end_col;
        
        // First check if any cell in the range is already in the recursion stack (immediate cycle)
        for (short r = rStart; r <= rEnd; r++) {
            for (short c = cStart; c <= cEnd; c++) {
                int idx = r * sheet->cols + c;
                if (recStack[idx])
                    return true;
            }
        }
        
        // Then recursively check unvisited cells in the range for cycles
        for (short r = rStart; r <= rEnd; r++) {
            for (short c = cStart; c <= cEnd; c++) {
                int idx = r * sheet->cols + c;
                if (!visited[idx]) {
                    if (check_dfs_cycle_util(sheet, r, c, visited, recStack))
                        return true;
                }
            }
        }
    } else {
        // Handle single cell dependency
        if (check_dfs_cycle_util(sheet, node->row, node->col, visited, recStack))
            return true;
    }
    
    // Check for cycles in right subtree
    if (check_cycle_in_avl(sheet, node->right, visited, recStack))
        return true;
    
    return false;
}

/**
 * Main function to check for dependency cycles starting from a specific cell
 * 
 * @param sheet Pointer to the spreadsheet structure
 * @param start_row Starting row index
 * @param start_col Starting column index
 * @return true if a cycle is detected, false otherwise
 */
bool check_dfs_cycle(Spreadsheet *sheet, short start_row, short start_col) {
    // Calculate total number of cells in the spreadsheet
    int total = sheet->rows * sheet->cols;
    
    // Allocate and initialize tracking arrays
    bool *visited = calloc(total, sizeof(bool));    // Tracks visited cells
    bool *recStack = calloc(total, sizeof(bool));   // Tracks cells in current recursion path
    
    // Start DFS from the specified cell
    bool cycleFound = check_dfs_cycle_util(sheet, start_row, start_col, visited, recStack);
    
    // Clean up memory
    free(visited);
    free(recStack);
    
    return cycleFound;
}