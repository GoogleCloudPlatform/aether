#include <stdio.h>

// AetherScript function declarations
extern int is_equal(int x, int y);
extern int is_greater(int x, int y);

int main() {
    printf("Testing AetherScript comparison operations:\n");
    printf("========================================\n");
    
    printf("is_equal(5, 5) = %d (expected: 1)\n", is_equal(5, 5));
    printf("is_equal(5, 7) = %d (expected: 0)\n", is_equal(5, 7));
    
    printf("is_greater(10, 5) = %d (expected: 1)\n", is_greater(10, 5));
    printf("is_greater(5, 10) = %d (expected: 0)\n", is_greater(5, 10));
    printf("is_greater(5, 5) = %d (expected: 0)\n", is_greater(5, 5));
    
    return 0;
}