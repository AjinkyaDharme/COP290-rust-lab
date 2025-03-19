#ifndef DFS_H
#define DFS_H

#include <stdbool.h>
#include "spreadsheet.h"
#include "avl.h"

bool check_dfs_cycle_util(Spreadsheet *sheet, short cur_row, short cur_col, bool *visited, bool *recStack);
bool check_cycle_in_avl(Spreadsheet *sheet, AVLNode* node, bool *visited, bool *recStack);
bool check_dfs_cycle(Spreadsheet *sheet, short start_row, short start_col);

#endif 
