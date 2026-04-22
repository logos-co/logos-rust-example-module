# logos-rust-example-module

A Logos module with logic implemented in pure Rust. Demonstrates the `c-ffi` interface: the Rust library compiles to a static C archive, and the Logos Module layer is fully auto-generated from the C header — no hand-written C++.

## Prerequisites

- [Nix](https://nixos.org/download) with flakes enabled
- Git

## Build

```bash
git add -A   # nix requires all files to be tracked
nix build
```

Output: `result/lib/rust_example_module_plugin.dylib` (macOS) or `.so` (Linux).

## Inspect

```bash
lm result/lib/rust_example_module_plugin.dylib
```

## Test with logoscore

Build the install package (creates a `modules/` directory with the manifest logoscore needs):

```bash
nix build .#install
```

Then call methods:

```bash
logoscore -m result/modules \
  -l rust_example_module \
  -c "rust_example_module.add(2,3)" \
  -c "rust_example_module.greet(Logos)" \
  -c "rust_example_module.libVersion()" \
  --quit-on-finish
```

## Build an LGX package

```bash
nix build .#lgx
# result is result/rust_example_module.lgx
```

Install it locally with `lgpm`:

```bash
lgpm --modules-dir ./modules install --file result/rust_example_module.lgx
logoscore -m ./modules -l rust_example_module -c "rust_example_module.add(2,3)" --quit-on-finish
```

## Exposed methods

| Method | Signature | Description |
|--------|-----------|-------------|
| `add` | `(int, int) -> int` | Add two integers |
| `multiply` | `(int, int) -> int` | Multiply two integers |
| `factorial` | `(int) -> int` | Factorial (returns -1 on overflow) |
| `fibonacci` | `(int) -> int` | Nth Fibonacci number |
| `is_prime` | `(int) -> int` | 1 if prime, 0 otherwise |
| `greet` | `(QString) -> QString` | Returns a greeting string |
| `libVersion` | `() -> QString` | Returns the Rust library version |

## How it works

```
rust-lib/src/lib.rs       ← Rust logic
       ↓  cargo build --release --offline
lib/librust_example.a     ← C static archive (staged by preConfigure)
lib/rust_example.h        ← C header (staged by preConfigure)
       ↓  logos-cpp-generator --from-c-header  (run by logos-module-builder)
generated_code/           ← Qt plugin glue (auto-generated, never edited)
       ↓  CMake + Qt MOC
rust_example_module_plugin.dylib  ← Loadable Logos module
```

See [`tutorial.md`](tutorial.md) for a full walkthrough.
