#include "avl.h"
#include "spreadsheet.h"  
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <limits.h>

/**
 * Returns the maximum of two short integers
 * @param a First integer to compare
 * @param b Second integer to compare
 * @return The larger of the two values
 */
short max_int(short a, short b) {
    return (a > b) ? a : b;
}

/**
 * Gets the height of an AVL tree node
 * @param node The node to get the height of
 * @return The height of the node, or 0 if node is NULL
 */
short heightAVL(AVLNode* node) {
    return (node == NULL) ? 0 : node->height;
}

/**
 * Creates a new AVL node with the given row and column indices
 * @param row The row index of the cell
 * @param col The column index of the cell
 * @return A newly allocated AVL node
 */
AVLNode* createAVLNode(short row, short col) {
    AVLNode* node = (AVLNode*)malloc(sizeof(AVLNode));
    node->row = row;
    node->col = col;
    node->is_range = false;
    node->left = node->right = NULL;
    node->height = 1;
    return node;
}

/**
 * Performs a right rotation on the given AVL tree node
 * @param y The node to rotate
 * @return The new root node after rotation
 */
AVLNode* rightRotate(AVLNode* y) {
    AVLNode* x = y->left;
    AVLNode* T2 = x->right;
    x->right = y;
    y->left = T2;
    y->height = max_int(heightAVL(y->left), heightAVL(y->right)) + 1;
    x->height = max_int(heightAVL(x->left), heightAVL(x->right)) + 1;
    return x;
}

/**
 * Performs a left rotation on the given AVL tree node
 * @param x The node to rotate
 * @return The new root node after rotation
 */
AVLNode* leftRotate(AVLNode* x) {
    AVLNode* y = x->right;
    AVLNode* T2 = y->left;
    y->left = x;
    x->right = T2;
    x->height = max_int(heightAVL(x->left), heightAVL(x->right)) + 1;
    y->height = max_int(heightAVL(y->left), heightAVL(y->right)) + 1;
    return y;
}

/**
 * Calculates the balance factor of an AVL node
 * @param node The node to calculate the balance factor for
 * @return The balance factor (difference between left and right subtree heights)
 */
short getBalance(AVLNode* node) {
    return (node == NULL) ? 0 : heightAVL(node->left) - heightAVL(node->right);
}

/**
 * Inserts a new node with given row and column into the AVL tree
 * @param node The root of the AVL tree
 * @param row The row index to insert
 * @param col The column index to insert
 * @return The new root of the AVL tree after insertion
 */
AVLNode* insertAVL(AVLNode* node, short row, short col) {
    // Perform standard BST insertion
    if (node == NULL)
        return createAVLNode(row, col);
    if (row < node->row || (row == node->row && col < node->col))
        node->left = insertAVL(node->left, row, col);
    else if (row > node->row || (row == node->row && col > node->col))
        node->right = insertAVL(node->right, row, col);
    else
        return node; // Duplicate node, return existing one
    
    // Update height of ancestor node
    node->height = 1 + max_int(heightAVL(node->left), heightAVL(node->right));
    
    // Get the balance factor to check if this node became unbalanced
    int balance = getBalance(node);
    
    // Left Left Case
    if (balance > 1 && (row < node->left->row || (row == node->left->row && col < node->left->col)))
        return rightRotate(node);
    
    // Right Right Case
    if (balance < -1 && (row > node->right->row || (row == node->right->row && col > node->right->col)))
        return leftRotate(node);
    
    // Left Right Case
    if (balance > 1 && (row > node->left->row || (row == node->left->row && col > node->left->col))) {
        node->left = leftRotate(node->left);
        return rightRotate(node);
    }
    
    // Right Left Case
    if (balance < -1 && (row < node->right->row || (row == node->right->row && col < node->right->col))) {
        node->right = rightRotate(node->right);
        return leftRotate(node);
    }
    
    // Return unchanged node
    return node;
}

/**
 * Finds the node with the minimum value in the AVL tree
 * @param node The root of the tree to search
 * @return The node with the minimum value
 */
AVLNode* minValueNode(AVLNode* node) {
    AVLNode* current = node;
    // Find the leftmost leaf
    while (current->left != NULL)
        current = current->left;
    return current;
}

/**
 * Deletes a node with the given row and column from the AVL tree
 * @param root The root of the AVL tree
 * @param row The row index to delete
 * @param col The column index to delete
 * @return The new root of the AVL tree after deletion
 */
AVLNode* deleteAVL(AVLNode* root, short row, short col) {
    // Standard BST delete operation
    if (root == NULL)
        return root;
    
    // Find the node to be deleted
    if (row < root->row || (row == root->row && col < root->col))
        root->left = deleteAVL(root->left, row, col);
    else if (row > root->row || (row == root->row && col > root->col))
        root->right = deleteAVL(root->right, row, col);
    else {
        // Node with only one child or no child
        if (root->left == NULL || root->right == NULL) {
            AVLNode* temp = root->left ? root->left : root->right;
            
            // No child case
            if (temp == NULL) {
                temp = root;
                root = NULL;
            } else {
                // One child case: copy the contents
                *root = *temp;
            }
            free(temp);
        } else {
            // Node with two children: get the inorder successor
            AVLNode* temp = minValueNode(root->right);
            
            // Copy the data
            root->row = temp->row;
            root->col = temp->col;
            
            // Delete the inorder successor
            root->right = deleteAVL(root->right, temp->row, temp->col);
        }
    }
    
    // If the tree had only one node, return
    if (root == NULL)
        return root;
    
    // Update height
    root->height = 1 + max_int(heightAVL(root->left), heightAVL(root->right));
    
    // Check balance factor
    int balance = getBalance(root);
    
    // Left Left Case
    if (balance > 1 && getBalance(root->left) >= 0)
        return rightRotate(root);
    
    // Left Right Case
    if (balance > 1 && getBalance(root->left) < 0) {
        root->left = leftRotate(root->left);
        return rightRotate(root);
    }
    
    // Right Right Case
    if (balance < -1 && getBalance(root->right) <= 0)
        return leftRotate(root);
    
    // Right Left Case
    if (balance < -1 && getBalance(root->right) > 0) {
        root->right = rightRotate(root->right);
        return leftRotate(root);
    }
    
    return root;
}

/**
 * Recursively frees all nodes in an AVL tree
 * @param node The root of the AVL tree to free
 */
void freeAVL(AVLNode* node) {
    if (node == NULL) return;
    freeAVL(node->left);
    freeAVL(node->right);
    free(node);
}

/**
 * Creates a deep copy of an AVL tree
 * @param node The root of the AVL tree to copy
 * @return The root of the new copy
 */
AVLNode* copyAVL(AVLNode* node) {
    if (node == NULL)
        return NULL;
    AVLNode* newNode = createAVLNode(node->row, node->col);
    newNode->left = copyAVL(node->left);
    newNode->right = copyAVL(node->right);
    newNode->height = node->height;
    newNode->is_range = node->is_range;
    newNode->end_row = node->end_row;
    newNode->end_col = node->end_col;
    return newNode;
}

/**
 * Compares two AVL nodes for ordering
 * @param a The first node to compare
 * @param b The second node to compare
 * @return -1 if a < b, 0 if a == b, 1 if a > b
 */
int compareAVLNodes(AVLNode* a, AVLNode* b) {
    short a_end = a->is_range ? a->end_row : a->row;
    short b_end = b->is_range ? b->end_row : b->row;
    if (a->row != b->row)
         return (a->row < b->row) ? -1 : 1;
    if (a->col != b->col)
         return (a->col < b->col) ? -1 : 1;
    if (a_end != b_end)
         return (a_end < b_end) ? -1 : 1;
    return 0;
}

/**
 * Creates a new AVL node representing a range of cells
 * @param row The starting row of the range
 * @param col The starting column of the range
 * @param end_row The ending row of the range
 * @param end_col The ending column of the range
 * @return A newly allocated AVL node
 */
 //These are used for the depends_on AVL tree for high optimization
AVLNode* createRangeAVLNode(short row, short col, short end_row, short end_col) {
    AVLNode* node = (AVLNode*)malloc(sizeof(AVLNode));
    node->row = row;
    node->col = col;
    node->is_range = true;
    node->end_row = end_row;
    node->end_col = end_col;
    node->left = node->right = NULL;
    node->height = 1;
    return node;
}

/**
 * Inserts a new range node into the AVL tree
 * @param node The root of the AVL tree
 * @param row The starting row of the range
 * @param col The starting column of the range
 * @param end_row The ending row of the range
 * @param end_col The ending column of the range
 * @return The new root of the AVL tree after insertion
 */
AVLNode* insertRangeAVL(AVLNode* node, short row, short col, short end_row, short end_col) {
    // Create a temporary node for comparison
    AVLNode temp;
    temp.row = row;
    temp.col = col;
    temp.is_range = true;
    temp.end_row = end_row;
    temp.end_col = end_col;
    
    // Perform standard BST insertion using node comparison
    if (node == NULL)
        return createRangeAVLNode(row, col, end_row, end_col);
    
    if (compareAVLNodes(&temp, node) < 0)
        node->left = insertRangeAVL(node->left, row, col, end_row, end_col);
    else if (compareAVLNodes(&temp, node) > 0)
        node->right = insertRangeAVL(node->right, row, col, end_row, end_col);
    else
        return node; // Duplicate node, return existing one
    
    // Update height of ancestor node
    node->height = 1 + max_int(heightAVL(node->left), heightAVL(node->right));
    
    // Get the balance factor to check if this node became unbalanced
    int balance = getBalance(node);
    
    // Left Left Case
    if (balance > 1 && compareAVLNodes(&temp, node->left) < 0)
        return rightRotate(node);
    
    // Right Right Case
    if (balance < -1 && compareAVLNodes(&temp, node->right) > 0)
        return leftRotate(node);
    
    // Left Right Case
    if (balance > 1 && compareAVLNodes(&temp, node->left) > 0) {
        node->left = leftRotate(node->left);
        return rightRotate(node);
    }
    
    // Right Left Case
    if (balance < -1 && compareAVLNodes(&temp, node->right) < 0) {
        node->right = rightRotate(node->right);
        return leftRotate(node);
    }
    
    return node;
}

/**
 * Deletes a range node from the AVL tree
 * @param root The root of the AVL tree
 * @param row The starting row of the range to delete
 * @param col The starting column of the range to delete
 * @param end_row The ending row of the range to delete
 * @param end_col The ending column of the range to delete
 * @return The new root of the AVL tree after deletion
 */
AVLNode* deleteRangeAVL(AVLNode* root, short row, short col, short end_row, short end_col) {
    if (root == NULL)
        return root;
    
    // Create a temporary node for comparison
    AVLNode temp;
    temp.row = row;
    temp.col = col;
    temp.is_range = true;
    temp.end_row = end_row;
    temp.end_col = end_col;
    
    // Find the node to be deleted using node comparison
    if (compareAVLNodes(&temp, root) < 0)
        root->left = deleteRangeAVL(root->left, row, col, end_row, end_col);
    else if (compareAVLNodes(&temp, root) > 0)
        root->right = deleteRangeAVL(root->right, row, col, end_row, end_col);
    else {
        // Node with only one child or no child
        if (root->left == NULL || root->right == NULL) {
            AVLNode* tempNode = root->left ? root->left : root->right;
            
            // No child case
            if (tempNode == NULL) {
                tempNode = root;
                root = NULL;
            } else {
                // One child case: copy the contents
                *root = *tempNode;
            }
            free(tempNode);
        } else {
            // Node with two children: get the inorder successor
            AVLNode* tempNode = minValueNode(root->right);
            
            // Copy the data including range information
            root->row = tempNode->row;
            root->col = tempNode->col;
            root->is_range = tempNode->is_range;
            root->end_row = tempNode->end_row;
            root->end_col = tempNode->end_col;
            
            // Delete the inorder successor
            root->right = deleteRangeAVL(root->right, tempNode->row, tempNode->col, tempNode->end_row, tempNode->end_col);
        }
    }
    
    // If the tree had only one node, return
    if (root == NULL)
        return root;
    
    // Update height
    root->height = 1 + max_int(heightAVL(root->left), heightAVL(root->right));
    
    // Check balance factor
    int balance = getBalance(root);
    
    // Left Left Case
    if (balance > 1 && getBalance(root->left) >= 0)
        return rightRotate(root);
    
    // Left Right Case
    if (balance > 1 && getBalance(root->left) < 0) {
        root->left = leftRotate(root->left);
        return rightRotate(root);
    }
    
    // Right Right Case
    if (balance < -1 && getBalance(root->right) <= 0)
        return leftRotate(root);
    
    // Right Left Case
    if (balance < -1 && getBalance(root->right) > 0) {
        root->right = rightRotate(root->right);
        return leftRotate(root);
    }
    
    return root;
}

/**
 * Adds a range dependency to a cell
 * @param cell The cell to add the dependency to
 * @param row The starting row of the range
 * @param col The starting column of the range
 * @param end_row The ending row of the range
 * @param end_col The ending column of the range
 */
void add_range_depends_on(struct Cell *cell, int row, int col, int end_row, int end_col) {
    cell->depends_on = insertRangeAVL(cell->depends_on, row, col, end_row, end_col);
}

/**
 * Removes a range dependency from a cell
 * @param cell The cell to remove the dependency from
 * @param row The starting row of the range
 * @param col The starting column of the range
 * @param end_row The ending row of the range
 * @param end_col The ending column of the range
 */
void remove_range_depends_on(struct Cell *cell, int row, int col, int end_row, int end_col) {
    cell->depends_on = deleteRangeAVL(cell->depends_on, row, col, end_row, end_col);
}

/**
 * Adds a dependent cell to the current cell's dependents list
 * @param cell The cell to update
 * @param row The row of the dependent cell
 * @param col The column of the dependent cell
 */
void add_dependent(struct Cell *cell, int row, int col) {
    cell->dependents = insertAVL(cell->dependents, row, col);
}

/**
 * Adds a dependency to a cell
 * @param cell The cell to update
 * @param row The row of the cell that this cell depends on
 * @param col The column of the cell that this cell depends on
 */
void add_depends_on(struct Cell *cell, int row, int col) {
    cell->depends_on = insertAVL(cell->depends_on, row, col);
}

/**
 * Removes a dependent from a cell's dependents list
 * @param cell The cell to update
 * @param row The row of the dependent cell to remove
 * @param col The column of the dependent cell to remove
 */
void remove_dependent(struct Cell *cell, int row, int col) {
    cell->dependents = deleteAVL(cell->dependents, row, col);
}

/**
 * Removes a dependency from a cell
 * @param cell The cell to update
 * @param row The row of the dependency to remove
 * @param col The column of the dependency to remove
 */
void remove_depends_on(struct Cell *cell, int row, int col) {
    cell->depends_on = deleteAVL(cell->depends_on, row, col);
}

/**
 * Clears all dependencies from a cell
 * @param cell The cell to clear dependencies from
 */
void clear_dependencies(struct Cell *cell) {
    if (cell->depends_on != NULL) {
        freeAVL(cell->depends_on);
        cell->depends_on = NULL;
    }
}