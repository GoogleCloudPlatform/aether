#include <stdio.h>

// AetherScript function declarations
extern int square(int x);
extern int sum_of_squares(int a, int b);
extern int factorial(int n);

int main() {
    printf("Testing AetherScript function calls:\n");
    printf("===================================\n");
    
    // Test square function
    printf("square(5) = %d (expected: 25)\n", square(5));
    printf("square(7) = %d (expected: 49)\n", square(7));
    printf("square(-3) = %d (expected: 9)\n", square(-3));
    
    printf("\n");
    
    // Test sum_of_squares function (calls square internally)
    printf("sum_of_squares(3, 4) = %d (expected: 25)\n", sum_of_squares(3, 4));
    printf("sum_of_squares(5, 12) = %d (expected: 169)\n", sum_of_squares(5, 12));
    
    printf("\n");
    
    // Test factorial function (recursive calls)
    printf("factorial(1) = %d (expected: 1)\n", factorial(1));
    printf("factorial(3) = %d (expected: 6)\n", factorial(3));
    printf("factorial(5) = %d (expected: 120)\n", factorial(5));
    
    return 0;
}