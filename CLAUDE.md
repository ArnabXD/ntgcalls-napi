# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What this is

A Rust N-API native addon that wraps `libntgcalls` (a C WebRTC shared library) for use in Node.js and Bun. The compiled output is a single `.node` file consumed as an ES module.

## Build

`libntgcalls.so` (or `.dylib`/`.dll`) must be present in `./lib/` before building.

```bash
cargo build --release

# Copy the compiled output to the loadable .node file
cp target/release/libntgcalls.so ./ntgcalls.node   # Linux
cp target/release/libntgcalls.dylib ./ntgcalls.node # macOS
```

`build.rs` sets linker search path to `./lib/` and embeds `$ORIGIN/lib` RPATH so the `.node` file resolves `libntgcalls` at runtime without `LD_LIBRARY_PATH`.

## Architecture

The crate (`src/lib.rs`) has three layers:

1. **C FFI layer** — `extern "C"` declarations mirroring the `libntgcalls` C API (`ntg_init`, `ntg_destroy`, `ntg_create`, `ntg_connect`, `ntg_set_stream_sources`, `ntg_pause`, `ntg_resume`, `ntg_mute`, `ntg_unmute`, `ntg_stop`, plus callback registration via `ntg_on_stream_end` / `ntg_on_connection_change`). The C structs (`NtgAsyncStruct`, `NtgAudioDescriptionStruct`, etc.) are `#[repr(C)]` mirrors of the C headers.

2. **Async bridge** — `libntgcalls` is callback-based. Each async method allocates an `AsyncContext` on the heap, passes a raw pointer into the C library as `user_data`, and pairs it with a `tokio::sync::oneshot` channel. The C library calls `rust_async_callback` when done, which reconstructs the box, sends the result on the channel, and the awaiting Rust future picks it up. The blocking C call itself runs inside `tokio::task::spawn_blocking` to avoid stalling the async runtime.

3. **N-API / JS layer** — `#[napi]` + `#[napi(constructor)]` macros (via `napi-derive`) expose the `NtgCalls` struct as a JS class. Event callbacks (`on_stream_end`, `on_connection_change`) use `ThreadsafeFunction` so that background C++ WebRTC threads can safely fire into the Node.js event loop.

## Key invariant

`NtgAsyncStruct.error_code` and `NtgAsyncStruct.error_message` are raw pointers into the `AsyncContext` box that is still heap-alive at the time of the C call. They must not be moved or dropped until `rust_async_callback` fires — this is why `context_addr` (a `usize`) is captured rather than the raw pointer directly.

## JS interface

`index.js` is an ESM wrapper that `require()`s the `.node` file and re-exports `NtgCalls`. `index.d.ts` is the hand-written TypeScript declaration file — update it manually when the Rust API changes, since `napi-derive` does not auto-generate it here.