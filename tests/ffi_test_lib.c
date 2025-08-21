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

// Simple C library for testing FFI
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

// Simple function to add two numbers
int add_numbers(int a, int b) {
    return a + b;
}

// Function that modifies an integer through a pointer
void modify_int(int* ptr) {
    if (ptr != NULL) {
        *ptr = *ptr * 2;
    }
}

// Function that returns string length
size_t get_string_length(const char* str) {
    if (str == NULL) {
        return 0;
    }
    return strlen(str);
}

// Struct for testing
typedef struct {
    int x;
    int y;
} Point;

// Function that sums point coordinates
int sum_point_coords(const Point* point) {
    if (point == NULL) {
        return 0;
    }
    return point->x + point->y;
}

// Function that allocates a buffer
void* allocate_buffer(size_t size) {
    return malloc(size);
}

// Function that deallocates a buffer
void deallocate_buffer(void* buffer) {
    free(buffer);
}

// Progress callback type
typedef void (*progress_callback_t)(int current, int total);

// Function that processes data with a callback
int process_data(void* data, size_t size, progress_callback_t callback) {
    if (data == NULL || callback == NULL) {
        return -1;
    }
    
    // Simulate processing
    for (int i = 0; i <= 10; i++) {
        callback(i, 10);
    }
    
    return 0;
}

// Function for geometry calculations
double calculate_distance(const Point* p1, const Point* p2) {
    if (p1 == NULL || p2 == NULL) {
        return 0.0;
    }
    
    int dx = p2->x - p1->x;
    int dy = p2->y - p1->y;
    
    // Simple distance calculation (not using sqrt for simplicity)
    return (double)(dx * dx + dy * dy);
}

typedef struct {
    Point top_left;
    double width;
    double height;
} Rectangle;

// Calculate rectangle area
double calculate_area(const Rectangle* rect) {
    if (rect == NULL) {
        return 0.0;
    }
    return rect->width * rect->height;
}