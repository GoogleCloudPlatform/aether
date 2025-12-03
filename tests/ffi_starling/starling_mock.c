#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>

// --- Starling Mock C Library ---

// 1. Opaque Handle Pattern
typedef struct {
    char* model_path;
    int loaded;
    int context_size;
} ModelContext;

// Initialize model (returns opaque handle)
void* starling_init(const char* model_path) {
    printf("[C] starling_init called with path: %s\n", model_path);
    fflush(stdout);
    ModelContext* ctx = (ModelContext*)malloc(sizeof(ModelContext));
    ctx->model_path = strdup(model_path);
    ctx->loaded = 1;
    ctx->context_size = 4096;
    return (void*)ctx;
}

// Check if model is loaded
int starling_is_loaded(void* handle) {
    printf("[C] starling_is_loaded called with %p\n", handle);
    fflush(stdout);
    ModelContext* ctx = (ModelContext*)handle;
    return ctx ? ctx->loaded : 0;
}

// Free model
void starling_free(void* handle) {
    printf("[C] starling_free called\n");
    fflush(stdout);
    if (handle) {
        ModelContext* ctx = (ModelContext*)handle;
        free(ctx->model_path);
        free(ctx);
    }
}

// 2. Array Passing Pattern
// Tokenize: takes string, returns array of ints (simulated)
// Returns number of tokens written
int starling_tokenize(void* handle, const char* text, int64_t* out_tokens, int max_tokens) {
    printf("[C] starling_tokenize called with text: %s\n", text);
    fflush(stdout);
    // Simulate tokenization: simple ASCII values offset by 1000
    int len = strlen(text);
    int count = 0;
    for (int i = 0; i < len && i < max_tokens; i++) {
        out_tokens[i] = (int64_t)text[i] + 1000;
        count++;
    }
    return count;
}

// 3. Function Pointer Pattern (Callback)
// Callback signature: void callback(int token_id, int pos)
typedef void (*token_callback_t)(int64_t token, int64_t pos); // Updated to int64_t to match Aether Int

// Generate: takes input tokens, generates output, calls callback for each token
void starling_generate(void* handle, int64_t* input_ids, int input_len, token_callback_t on_token) {
    printf("[C] starling_generate called with %d input tokens\n", input_len);
    fflush(stdout);
    
    // Simulate generation (just returning 3 tokens: 100, 101, 102)
    for (int i = 0; i < 3; i++) {
        int64_t new_token = 100 + i;
        printf("[C] Generating token %lld at pos %d\n", new_token, i);
        fflush(stdout);
        on_token(new_token, (int64_t)i);
    }
}

// 4. Helper for Aether testing (pointer casting workaround)
int64_t debug_ptr_to_int(void* ptr) {
    printf("[C] debug_ptr_to_int called with %p\n", ptr);
    fflush(stdout);
    return (int64_t)ptr;
}

void* debug_int_to_ptr(int64_t val) {
    printf("[C] debug_int_to_ptr called with %lld\n", val);
    fflush(stdout);
    return (void*)val;
}

