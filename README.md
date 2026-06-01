# @arnabxd/ntgcalls-napi

[![npm version](https://img.shields.io/npm/v/@arnabxd/ntgcalls-napi?color=blue&style=flat-square)](https://www.npmjs.com/package/@arnabxd/ntgcalls-napi)
[![license](https://img.shields.io/github/license/ArnabXD/ntgcalls-napi?style=flat-square)](LICENSE)
[![GitHub Actions CI](https://img.shields.io/github/actions/workflow/status/ArnabXD/ntgcalls-napi/ci.yml?branch=main&style=flat-square)](https://github.com/ArnabXD/ntgcalls-napi/actions)
[![Platform Support](https://img.shields.io/badge/platforms-linux%20%7C%20macos%20%7C%20windows-orange?style=flat-square)](https://github.com/ArnabXD/ntgcalls-napi/releases)

Thread-safe Node-API (N-API) native bindings in Rust for `libntgcalls` (C-shared WebRTC library). Fully compatible with Node.js and Bun.

---

## 🚀 Key Features

* **High-Level JS Wrapper**: Native bindings wrapped in a clean, standard `EventEmitter` class.
* **Smart Parameter Mapping**: Automatically accepts either a JavaScript `number` or a `bigint` for Telegram Chat/User IDs, timestamps, and fingerprints, eliminating manual BigInt conversions.
* **100% Thread-Safe Callbacks**: Translates background native C++ WebRTC thread events into Node.js event-loop tasks crash-free using N-API `ThreadsafeFunction`.
* **Async Promise-Based API**: Non-blocking async operations (`create`, `connect`, etc.) delegated to a Tokio threadpool via safe one-shot synchronization channels.
* **RPATH Isolation**: Automatically resolves the dynamic dependency `libntgcalls` relative to the native addon directory using `$ORIGIN/lib` RPATH (no need for `LD_LIBRARY_PATH`!).

---

## 📦 Installation

```bash
npm install @arnabxd/ntgcalls-napi
```

> [!NOTE]
> The platform-specific binary is automatically downloaded at postinstall time, so compilation is not required under normal use.

---

## 🛠️ Prerequisites (For Compiling From Source)

1. **Rust & Cargo**:
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```
2. **libntgcalls**:
   Download the platform-appropriate `libntgcalls` shared library (`libntgcalls.so` / `libntgcalls.dylib` / `ntgcalls.dll`) from the [ntgcalls releases](https://github.com/pytgcalls/ntgcalls/releases/latest) and place it into the `./lib/` folder prior to compilation.

---

## 🏗️ Build Instructions

To compile the native binary from source:
```bash
npm run build
```
This runs `@napi-rs/cli` (`napi build --release --dts binding.d.ts`) which compiles the Rust crate and outputs `ntgcalls.node` in the package root.

---

## 📚 Quick Start Usage Example

```typescript
import { NtgCalls } from '@arnabxd/ntgcalls-napi';

const ntg = new NtgCalls();

// Register WebRTC event listeners (Fully typed!)
ntg.on('connection-change', (chatId, kind, state) => {
  console.log(`Connection changed: chat=${chatId}, kind=${kind}, state=${state}`);
});

ntg.on('stream-end', (chatId, streamType, streamDevice) => {
  console.log(`Stream ended: chat=${chatId}, type=${streamType}, device=${streamDevice}`);
});

// Start a WebRTC session (supports either number or bigint!)
const chatId = -1001185324811n; 
const offerSdp = await ntg.create(chatId);

console.log('Generated WebRTC Offer SDP:', offerSdp);
```

For comprehensive details on all available APIs, data structures, and best practices, check the **[Client Integration Guide (DOC.md)](DOC.md)**.
