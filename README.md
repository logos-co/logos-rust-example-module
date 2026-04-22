# logos-rust-example-module

Two Logos modules demonstrating **inter-module IPC from Rust** using the `c-ffi` interface and [`logos-rust-sdk`](../logos-rust-sdk).

## Modules

### `rust_provider_module` (callee)

A pure Rust math library. Exposes `add`, `multiply`, `greet`, etc. as Logos IPC methods. No hand-written C++ — the Qt plugin glue is auto-generated from a C header.

### `rust_caller_module` (caller)

A Rust module that calls `rust_provider_module` **through the Logos IPC stack** using [`logos-rust-sdk`](../logos-rust-sdk). The SDK wraps `logos-module-client`'s C API, handling parameter serialization, memory management, and error handling.

```rust
use logos_rust_sdk::LogosModuleSDK;

let sdk = LogosModuleSDK::new();
let provider = sdk.plugin("rust_provider_module");
let result = provider.call_sync("add", &[5i64, 3i64])?;
```

## IPC Architecture

```
logoscore
  └─► rust_caller_module (Qt plugin)
        └─► rust_caller_call_add(5, 3)   [C function in librust_caller.a]
              └─► LogosModuleSDK::call_sync("add", &[5, 3])   [logos-rust-sdk]
                    └─► logos_sdk_call_method_sync(...)         [logos-module-client C API]
                          └─► LogosCoreClient::callMethodSync (C++)
                                └─► LogosAPIClient::invokeRemoteMethod (Qt Remote Objects IPC)
                                      └─► rust_provider_module (Qt plugin)
                                            └─► rust_provider_add(5, 3)  →  8
```

## Prerequisites

- [Nix](https://nixos.org/download) with flakes enabled
- Git

## Build

```bash
git add -A   # Nix requires all files to be tracked

# Build just the provider module
nix build .#rust_provider_module

# Build just the caller module
nix build .#rust_caller_module

# Build both (combined output)
nix build
```

## Test IPC

```bash
# Build the combined modules directory first
nix build

# Run the caller module; it calls the provider through Logos IPC
logoscore \
  -m result \
  -l rust_caller_module \
  -c "rust_caller_module.call_add(5, 3)" \
  -c "rust_caller_module.call_multiply(4, 6)" \
  -c "rust_caller_module.call_greet(World)"
```

Or run the automated check:

```bash
nix flake check
```

## Module APIs

### rust_provider_module

| Method | Signature | Description |
|--------|-----------|-------------|
| `add` | `(int64, int64) -> int64` | Add two integers |
| `multiply` | `(int64, int64) -> int64` | Multiply (saturating) |
| `factorial` | `(int64) -> int64` | Factorial (-1 on overflow) |
| `fibonacci` | `(int64) -> int64` | Nth Fibonacci (-1 on overflow) |
| `isPrime` | `(int64) -> int64` | 1 if prime, 0 otherwise |
| `greet` | `(string) -> string` | "Hello, {name}! (from Rust provider)" |
| `version` | `() -> string` | Library version |

### rust_caller_module

| Method | Signature | Description |
|--------|-----------|-------------|
| `call_add` | `(int64, int64) -> int64` | Calls provider.add via IPC |
| `call_multiply` | `(int64, int64) -> int64` | Calls provider.multiply via IPC |
| `call_greet` | `(string) -> string` | Calls provider.greet via IPC |
| `provider_name` | `() -> string` | Returns the provider module name |

## How it works

### Provider (pure Rust -> C FFI -> Qt plugin)

```
rust-provider-module/rust-lib/src/lib.rs   <- Rust functions (no Qt)
          |  cargo build --release
lib/librust_provider.a                     <- Static archive (staged by preConfigure)
lib/rust_provider.h                        <- C header (staged by preConfigure)
          |  logos-cpp-generator --from-c-header
generated_code/                            <- Qt plugin glue (auto-generated)
          |  CMake + MOC
rust_provider_module_plugin.dylib          <- Loadable Logos module
```

### Caller (Rust + logos-rust-sdk -> Qt plugin)

```
rust-caller-module/rust-lib/src/lib.rs     <- Uses LogosModuleSDK to call provider
          |  cargo build --release (with vendored deps + SDK source)
lib/librust_caller.a                       <- Static archive (staged by preConfigure)
lib/rust_caller.h                          <- C header (staged by preConfigure)
          |  logos-cpp-generator --from-c-header
generated_code/                            <- Qt plugin glue (auto-generated)
          |  CMake + MOC + link liblogos_module_client
rust_caller_module_plugin.dylib            <- Loadable Logos module
```

The caller's Rust code uses `logos-rust-sdk` which declares `extern "C"` bindings to `logos_sdk_*` functions.
These symbols are resolved at link time when CMake links the plugin `.dylib` against `liblogos_module_client`.

### Nix build: how logos-rust-sdk is available offline

The caller's `rust-lib/` has:
- `Cargo.toml` with `logos-rust-sdk = { path = "../logos-rust-sdk-src" }`
- `vendor/` directory containing all transitive Cargo dependencies (serde, serde_json, etc.)
- `.cargo/config.toml` pointing Cargo at the vendor directory

During the Nix build, `preConfigure` copies the `logos-rust-sdk` flake input source into `logos-rust-sdk-src/` so the path dependency resolves inside the sandbox. `cargo build --release --offline` then compiles everything from vendored sources.

## logos-rust-sdk API

The caller module uses the [logos-rust-sdk](../logos-rust-sdk) API:

```rust
use logos_rust_sdk::LogosModuleSDK;

let sdk = LogosModuleSDK::new();
let provider = sdk.plugin("rust_provider_module");

// Synchronous call (for use in Q_INVOKABLE-generated functions)
let result = provider.call_sync("add", &[5i64, 3i64])?;
println!("5 + 3 = {}", result.message);  // "8"

// Asynchronous call with channel-based result
let rx = provider.call("greet", &["World"])?;
if let Ok(result) = rx.try_recv() {
    println!("{}", result.message);
}

// Event subscription
let mut chat = sdk.plugin("chat_module");
let events = chat.on("newMessage")?;
while let Ok(event) = events.try_recv() {
    println!("Event: {} - {:?}", event.event, event.data);
}
```
