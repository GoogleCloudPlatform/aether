// Test C library for callback FFI testing
#include <stdio.h>

// Simple callback that takes an int and returns an int
typedef int (*int_callback)(int);

// Function that calls a callback with a value
int call_with_value(int_callback cb, int value) {
    printf("C: Calling callback with value %d\n", value);
    int result = cb(value);
    printf("C: Callback returned %d\n", result);
    return result;
}

// Callback that takes two ints and returns an int
typedef int (*binary_callback)(int, int);

// Function that applies a binary operation
int apply_binary_op(binary_callback cb, int a, int b) {
    printf("C: Applying binary operation to %d and %d\n", a, b);
    int result = cb(a, b);
    printf("C: Binary operation returned %d\n", result);
    return result;
}

// Callback that takes no parameters and returns void
typedef void (*void_callback)(void);

// Function that calls a void callback multiple times
void call_repeatedly(void_callback cb, int times) {
    printf("C: Calling void callback %d times\n", times);
    for (int i = 0; i < times; i++) {
        cb();
    }
    printf("C: Finished calling callback\n");
}

// Callback for array processing
typedef void (*array_callback)(int*, int);

// Function that processes an array with a callback
void process_array(int* array, int length, array_callback cb) {
    printf("C: Processing array of length %d\n", length);
    cb(array, length);
    printf("C: Array processing complete\n");
}

// Callback that returns a float
typedef float (*float_callback)(float);

// Function that transforms a float value
float transform_float(float_callback cb, float value) {
    printf("C: Transforming float value %f\n", value);
    float result = cb(value);
    printf("C: Transform returned %f\n", result);
    return result;
}