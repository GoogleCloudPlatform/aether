#include <stdio.h>

// AetherScript function declarations
extern int max(int x, int y);
extern int abs(int x);

int main() {
    printf("Testing AetherScript control flow operations:\n");
    printf("=============================================\n");
    
    // Test max function
    printf("max(10, 5) = %d (expected: 10)\n", max(10, 5));
    printf("max(3, 8) = %d (expected: 8)\n", max(3, 8));
    printf("max(7, 7) = %d (expected: 7)\n", max(7, 7));
    printf("max(-2, 5) = %d (expected: 5)\n", max(-2, 5));
    
    printf("\n");
    
    // Test abs function
    printf("abs(10) = %d (expected: 10)\n", abs(10));
    printf("abs(-5) = %d (expected: 5)\n", abs(-5));
    printf("abs(0) = %d (expected: 0)\n", abs(0));
    printf("abs(-42) = %d (expected: 42)\n", abs(-42));
    
    return 0;
}