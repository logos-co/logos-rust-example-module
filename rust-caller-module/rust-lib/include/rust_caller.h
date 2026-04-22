#ifndef RUST_CALLER_H
#define RUST_CALLER_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Call rust_provider_module.add(a, b) via Logos IPC.
 * Returns the sum, or -1 on failure.
 */
int64_t rust_caller_call_add(int64_t a, int64_t b);

/**
 * Call rust_provider_module.multiply(a, b) via Logos IPC.
 * Returns the product, or -1 on failure.
 */
int64_t rust_caller_call_multiply(int64_t a, int64_t b);

/**
 * Call rust_provider_module.greet(name) via Logos IPC.
 * Returns a heap-allocated C string that must be freed with rust_caller_free_string().
 * Passing NULL uses "World" as the name.
 */
char* rust_caller_call_greet(const char* name);

/** Free a string returned by rust_caller_call_greet(). */
void rust_caller_free_string(char* ptr);

/**
 * Return the name of the provider module this caller talks to.
 * Static string — do NOT free.
 */
const char* rust_caller_provider_name(void);

#ifdef __cplusplus
}
#endif

#endif /* RUST_CALLER_H */
