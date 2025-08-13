#include <stdio.h>

// AetherScript function declarations
extern int add(int x, int y);
extern int subtract(int x, int y);
extern int multiply(int x, int y);
extern int divide(int x, int y);

int main() {
    printf("Testing AetherScript mathematical operations:\n");
    printf("================================\n");
    
    printf("add(10, 5) = %d\n", add(10, 5));
    printf("subtract(10, 5) = %d\n", subtract(10, 5));
    printf("multiply(10, 5) = %d\n", multiply(10, 5));
    printf("divide(10, 5) = %d\n", divide(10, 5));
    
    printf("\nMore complex tests:\n");
    printf("add(100, -50) = %d\n", add(100, -50));
    printf("multiply(7, 6) = %d\n", multiply(7, 6));
    printf("divide(100, 3) = %d (integer division)\n", divide(100, 3));
    
    return 0;
}