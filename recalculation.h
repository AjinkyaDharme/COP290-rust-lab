#ifndef RECALCULATION_H
#define RECALCULATION_H

#include "spreadsheet.h"

void recalc_topological(Spreadsheet *sheet, short changed_row, short changed_col);
void recalculate_cell(Spreadsheet *sheet, short row, short col);

#endif
