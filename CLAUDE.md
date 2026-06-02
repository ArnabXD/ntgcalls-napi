# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust N-API native addon that wraps `libntgcalls` (a C WebRTC shared library) for use in Node.js and Bun. The compiled output is a single `.node` file consumed as an ES module.

## Build

`libntgcalls.so` (or `.dylib`/`.dll`) must be present in `./lib/` before building. Download it from the [pytgcalls/ntgcalls releases](https://github.com/pytgcalls/ntgcalls/releases/latest) and unzip the platform-appropriate shared-libs archive into `./lib/`.

To build the native addon and auto-generate TypeScript type definitions (`index.d.ts`), run:

```bash
npm run build
```

This uses `@napi-rs/cli` (`napi build --release`) which compiles the Rust crate and outputs `ntgcalls.node` in the package root.

`build.rs` sets linker search path to `./lib/` and embeds `$ORIGIN/lib` (Linux) / `@loader_path/lib` (macOS) RPATH so the `.node` file resolves `libntgcalls` at runtime without `LD_LIBRARY_PATH`.

## Lint & format

```bash
cargo fmt --all              # format Rust
cargo fmt --all -- --check   # check only (used in CI)
cargo clippy --all-targets -- -D warnings   # lint Rust (zero warnings)
cargo test --verbose         # run Rust test suite

npm install                  # install JS dev tools (biome, lefthook)
npx biome check .            # lint/format JS files
npx biome check --write .    # auto-fix JS files
```

## Architecture

The crate has been modularized from a massive monolith into several logical submodules to improve readability and maintainability:

- `src/ffi.rs`: Raw `extern "C"` declarations of the `libntgcalls` C API and the `#[repr(C)]` mirroring C structs (e.g. `ntg_async_struct`, `ntg_audio_description_struct`).
- `src/types.rs`: High-level Rust structs and types that map to N-API/JavaScript objects (e.g. `MediaState`, `Frame`, `DeviceInfo`).
- `src/callbacks.rs`: Heap-allocated `AsyncContext` handlers and raw FFI callbacks (e.g. `rust_async_callback`, `raw_stream_end_callback`) that safely bridge background threads back to the Node.js event loop using N-API `ThreadsafeFunction` channels.
- `src/utils.rs`: Synchronous library-level utilities (e.g. `get_version`, `get_protocol`, `register_logger`).
- `src/events.rs`: Stateful event listener registration methods on the `NtgCalls` instance (e.g. `on_stream_end`, `on_connection_change`).
- `src/session.rs`: Call session initialization and control methods (e.g. `create`, `connect`, `stop`).
- `src/video.rs`: Multi-stream incoming video channel management and custom frame injection.
- `src/p2p.rs`: Direct Peer-to-Peer connection management and Diffie-Hellman cryptographic exchanges.
- `src/broadcast.rs`: Broadcast streaming source controls and segment chunk delivery.
- `src/lib.rs`: Exposes the submodule tree, the central `NtgCalls` struct, its N-API constructor, its `Drop` implementation, and the internal async execution helper.

### Core Implementation Layers
1. **C FFI layer** (`src/ffi.rs`) — Low-level interface matching the `libntgcalls` C API.
2. **Async bridge** (`src/callbacks.rs`) — Allocates an `AsyncContext` on the heap, passes a raw pointer as `user_data`, and pairs it with a `oneshot` channel. The C library fires `rust_async_callback` when done, which resolves the awaiting Rust future.
3. **N-API / JS layer** (spread across module files) — Exposes class methods and structs using `#[napi]` annotations. Events use `ThreadsafeFunction` to prevent cross-thread event-loop starvation.

## Key invariants

- `NtgAsyncStruct.error_code` and `NtgAsyncStruct.error_message` are raw pointers into the `AsyncContext` box that is still heap-alive at the time of the C call. They must not be moved or dropped until `rust_async_callback` fires — this is why `context_addr` (a `usize`) is captured rather than the raw pointer directly.

- `on_stream_end` / `on_connection_change` register their C callback exactly once per `NtgCalls` instance (guarded by `compare_exchange` on an `AtomicPtr`). The `Arc<Mutex<Option<ThreadsafeFunction<…>>>>` holding the JS callback is leaked into a raw pointer and given to the C library; subsequent calls to the same `on_*` method just swap the `ThreadsafeFunction` inside the mutex. Both leaked `Arc` clones are reclaimed in `Drop` after `ntg_destroy` guarantees no further callbacks.

- `Drop` spin-waits on `pending_ops` (an `AtomicUsize`) to reach zero before calling `ntg_destroy`, ensuring all in-flight `spawn_blocking` tasks have exited the C library.

- `AsyncContext._keep_alive` stores `CString` and heap-allocated C structs that must remain valid until `rust_async_callback` fires. Do not reference them through the original stack-bound variable after passing to `spawn_blocking`.

## JS interface

`index.js` is an ESM wrapper that `require()`s the `.node` file, wraps `NativeNtgCalls` in an EventEmitter subclass, and re-exports everything. The build command (`napi build --release --dts binding.d.ts`) auto-generates `binding.d.ts` — do NOT edit it manually. All type adjustments (mapping native `i64` to TypeScript `bigint` etc.) must be done in Rust source files using N-API macros (`#[napi(ts_type = "bigint")]`, `#[napi(ts_arg_type = "bigint")]`, `#[napi(ts_return_type = "Promise<bigint>")]`).

`index.d.ts` is hand-maintained. It imports types from `binding.d.ts` and re-exports only the public-facing interfaces and standalone functions — the raw `NtgCalls` native class and its `on_*` callback methods are intentionally not re-exported. The `NtgCalls` EventEmitter class and `register_logger` (wrapper signature) are declared directly here. When adding new Rust-exposed types or functions, update `index.d.ts` accordingly.

The `set_audio_source` method always uses `media_source: 2` (SHELL) at 48 kHz mono; it is the only stream-source helper currently exposed to JS.

## Release

CI builds platform-specific `.node` files named `ntgcalls.<platform>.node` (e.g. `ntgcalls.linux-x64.node`) and attaches them to GitHub releases when a `v*` tag is pushed. The `libntgcalls` shared library is bundled at runtime in a `lib/` subdirectory beside the `.node` file, resolved via RPATH — consumers do not need to set `LD_LIBRARY_PATH`.