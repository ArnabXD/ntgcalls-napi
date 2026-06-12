---
title: Overview
order: 1
group: Guide
eyebrow: Architecture
description: Architecture and core concepts of the ntgcalls-napi bindings.
---

# Architectural Overview

This package provides native, thread-safe, high-performance Node-API (N-API) bindings in Rust
for `libntgcalls` (the native WebRTC C-shared library). It wraps the native code into a robust,
non-blocking asynchronous architecture optimized for Node.js:

```text
┌────────────────────────────────────────────────────────┐
│                      Node.js / JS                      │
│   Event Loop / Main Thread (JavaScript / TypeScript)   │
└───────────────────────────┬────────────────────────────┘
     ▲                       │                       │
     │ Thread-safe           │ Async Promises        │ Callback Register
     │ Callbacks             ▼                       ▼
┌────┴───────────────────────────────────────────────────┐
│                    N-API FFI Wrapper                   │
│        (Safe Rust Integration, thread-safe-fns)        │
└───────────────────────────┬────────────────────────────┘
     ▲                       │                       │
     │ Native Worker         │ spawn_blocking        │ C FFI
     │ Signals               ▼                       ▼
┌────┴───────────────────────────────────────────────────┐
│                      libntgcalls                       │
│    Background C++ WebRTC Threads, Key Exchange, etc.    │
└────────────────────────────────────────────────────────┘
```

## Key Architectural Concepts

- **Thread Safety** — WebRTC signaling and media processing run on background native C++ threads.
  To safely deliver notifications to JavaScript, the wrapper uses N-API `ThreadsafeFunction`
  channels, converting native thread dispatches into event-loop tasks.

- **Non-Blocking Async** — Heavy operations (e.g. SDP generation, key exchange) are delegated to a
  Tokio threadpool via `spawn_blocking` with safe one-shot synchronization channels.

- **BigInt Representation** — Telegram Chat IDs, User IDs, and WebRTC timestamps may be supplied as
  either a JavaScript `number` or a `bigint` (`number | bigint`). The JS wrapper coerces `bigint`
  arguments to `Number` at the N-API boundary, since napi-rs v3 expects a JS `Number` at runtime for
  `i64` parameters — this is safe because Telegram IDs fit within `Number.MAX_SAFE_INTEGER`
  (2^53 − 1). Values returned by the library (e.g. `time()`) are still `bigint`.

- **Memory Safety** — Wrapper context allocations (e.g. C-string parameters, vector structures) are
  maintained in native heap context structures (`Keeper` types) and safely freed automatically once
  native callbacks complete.

## Installation

```bash
npm install @arnabxd/ntgcalls-napi
```

The platform-specific binary is automatically downloaded at postinstall time, so compilation is not
required under normal use. Prebuilt binaries are published for Linux, macOS, and Windows.

### Building from source

If you need to compile the native addon yourself:

1. Install **Rust & Cargo**.
2. Download the platform-appropriate `libntgcalls` shared library
   (`libntgcalls.so` / `libntgcalls.dylib` / `ntgcalls.dll`) from the
   [ntgcalls releases](https://github.com/pytgcalls/ntgcalls/releases/latest) and place it in
   `./lib/`.
3. Run `npm run build`, which invokes `@napi-rs/cli` to compile the Rust crate and output
   `ntgcalls.node` in the package root.