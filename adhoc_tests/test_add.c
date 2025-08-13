#include <stdio.h>

// Declaration of our AetherScript add function
extern int add(int x, int y);

int main() {
    int result = add(5, 7);
    printf("add(5, 7) = %d\n", result);
    
    result = add(10, 32);
    printf("add(10, 32) = %d\n", result);
    
    return 0;
}