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