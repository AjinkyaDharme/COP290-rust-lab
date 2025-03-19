#ifndef AVL_H
#define AVL_H

#include <stdbool.h>

// Forward declaration of Cell (defined in spreadsheet.h)
struct Cell;

typedef struct AVLNode {
    short row;
    short col;
    bool is_range;
    short end_row;
    short end_col;
    struct AVLNode* left;
    struct AVLNode* right;
    short height;
} AVLNode;

short max_int(short a, short b);
short heightAVL(AVLNode* node);
AVLNode* createAVLNode(short row, short col);
AVLNode* rightRotate(AVLNode* y);
AVLNode* leftRotate(AVLNode* x);
short getBalance(AVLNode* node);
AVLNode* insertAVL(AVLNode* node, short row, short col);
AVLNode* deleteAVL(AVLNode* root, short row, short col);
void freeAVL(AVLNode* node);
AVLNode* copyAVL(AVLNode* node);

int compareAVLNodes(AVLNode* a, AVLNode* b);
AVLNode* createRangeAVLNode(short row, short col, short end_row, short end_col);
AVLNode* insertRangeAVL(AVLNode* node, short row, short col, short end_row, short end_col);
AVLNode* deleteRangeAVL(AVLNode* root, short row, short col, short end_row, short end_col);

void add_range_depends_on(struct Cell *cell, int row, int col, int end_row, int end_col);
void remove_range_depends_on(struct Cell *cell, int row, int col, int end_row, int end_col);
void add_dependent(struct Cell *cell, int row, int col);
void add_depends_on(struct Cell *cell, int row, int col);
void remove_dependent(struct Cell *cell, int row, int col);
void remove_depends_on(struct Cell *cell, int row, int col);
void clear_dependencies(struct Cell *cell);

#endif 
