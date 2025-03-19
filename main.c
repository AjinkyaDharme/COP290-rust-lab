#include "spreadsheet.h"
#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <time.h>
#include <string.h>


double get_elapsed_time(struct timespec start, struct timespec end) {
    return (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1000000000.0;
}

int main(int argc, char *argv[]) {
    struct timespec program_start, program_end;
    clock_gettime(CLOCK_MONOTONIC, &program_start);
    if (argc != 3) {
        printf("(Invalid Command) ");
        return 1;
    }
    short rows = atoi(argv[1]);
    short cols = atoi(argv[2]);
    gl_rows = rows;
    gl_cols = cols;
    recalcVisited = calloc(rows * cols, sizeof(bool));
    if (rows < 1 || rows > 999 || cols < 1 || cols > 18278) {
        printf("(Invalid command) ");
        return 1;
    }
    Spreadsheet *sheet = initSpreadsheet(rows, cols);
    displaySpreadsheet(sheet);
    clock_gettime(CLOCK_MONOTONIC, &program_end);
    double elapsed_time = get_elapsed_time(program_start, program_end);
    printf("[%.1f] (ok) > ", elapsed_time);
    bool output_enabled = true;
    // printf("%zu",sizeof(Expression));

    while (1) {
        char user_input[35];
        if (!fgets(user_input, sizeof(user_input), stdin))
            break;
        clock_gettime(CLOCK_MONOTONIC, &program_start);
        user_input[strcspn(user_input, "\n")] = '\0';
        commandError = false;
        if (strcmp(user_input, "q") == 0)
            break;
        if (strcmp(user_input, "disable_output") == 0) {
            output_enabled = false;
        } else if (strcmp(user_input, "enable_output") == 0) {
            output_enabled = true;
        } else if (strncmp(user_input, "scroll_to", 9) == 0) {
            char cellStr[15];
            if (sscanf(user_input, "scroll_to %s", cellStr) == 1)
                scrollTo(sheet, cellStr);
            else
                commandError = true;  
        } else if ((user_input[0] == 'w' || user_input[0] == 's' ||
                    user_input[0] == 'a' || user_input[0] == 'd') && strlen(user_input) == 1) {
            scrollSpreadsheet(sheet, user_input[0]);
        } else {
            setCell(sheet, user_input);
        }
        if (output_enabled)
            displaySpreadsheet(sheet);
        clock_gettime(CLOCK_MONOTONIC, &program_end);
        double total_time = get_elapsed_time(program_start, program_end);
        if (commandError)
            printf("[%.1f] (Invalid command) > ", total_time);
        else
            printf("[%.1f] (ok) > ", total_time);
    }
    freeSpreadsheet(sheet);
    return 0;
}