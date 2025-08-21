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