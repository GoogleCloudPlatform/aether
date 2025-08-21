/*
 * Copyright 2025 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

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