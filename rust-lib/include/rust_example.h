#ifndef RUST_EXAMPLE_H
#define RUST_EXAMPLE_H

#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/** Add two integers. */
int64_t rust_example_add(int64_t a, int64_t b);

/** Multiply two integers (saturating on overflow). */
int64_t rust_example_multiply(int64_t a, int64_t b);

/** Compute n! (factorial). Returns -1 on overflow or negative input. */
int64_t rust_example_factorial(int64_t n);

/** Compute the nth Fibonacci number. Returns -1 on overflow or negative input. */
int64_t rust_example_fibonacci(int64_t n);

/** Return 1 if n is prime, 0 otherwise. */
int64_t rust_example_is_prime(int64_t n);

/**
 * Greet the given name.
 * Returns a heap-allocated C string that must be freed with rust_example_free_string().
 * Passing NULL uses "World" as the name.
 */
char* rust_example_greet(const char* name);

/** Free a string returned by rust_example_greet(). */
void rust_example_free_string(char* ptr);

/** Return the Rust library version string (static, do not free). */
const char* rust_example_version(void);

#ifdef __cplusplus
}
#endif

#endif /* RUST_EXAMPLE_H */
