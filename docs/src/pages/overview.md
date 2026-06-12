---
title: Overview
order: 1
group: Guide
eyebrow: Start here
description: What ntgcalls-napi is, what you need before using it, and how the pieces fit.
---

# Overview

`@arnabxd/ntgcalls-napi` lets your Node.js or Bun app join Telegram group voice/video chats and
stream media into them. It's a thin, thread-safe binding over
[`libntgcalls`](https://github.com/pytgcalls/ntgcalls) — the native WebRTC engine that does the
actual audio/video work.

It handles **media**: encoding your audio/video, the WebRTC transport, and the call lifecycle. It
does **not** talk to Telegram's API. You bring that part yourself.

## What you need first

> [!IMPORTANT]
> You need an **MTProto library logged in as a regular user account** — not a bot. Bots cannot join
> voice chats. Use [MTKruto](https://mtkru.to), [GramJS](https://gram.js.org), or any MTProto client.

The two halves of a call:

| Piece | Who does it | Examples |
| --- | --- | --- |
| **Signaling** — join the call, exchange the SDP offer/answer with Telegram | Your MTProto library | MTKruto, GramJS |
| **Media** — produce the audio/video, run the WebRTC connection | This package | `NtgCalls` |

The flow is always: **this package makes an offer → your MTProto lib joins the call with it and
returns Telegram's answer → this package connects with that answer → you push media.** The
[Quick Start](/quick-start/) walks through it end to end.

## Installation

{% tabs "pm" %}
{% tab "npm" %}
```bash
npm install @arnabxd/ntgcalls-napi
```
{% endtab %}
{% tab "pnpm" %}
```bash
pnpm add @arnabxd/ntgcalls-napi
```
{% endtab %}
{% tab "yarn" %}
```bash
yarn add @arnabxd/ntgcalls-napi
```
{% endtab %}
{% tab "bun" %}
```bash
bun add @arnabxd/ntgcalls-napi
```
{% endtab %}
{% endtabs %}

The right prebuilt binary (Linux, macOS, Windows) is downloaded automatically on install — no
compiler or `LD_LIBRARY_PATH` setup needed.

You'll also need **ffmpeg** on the host, since the common way to feed media is to let ntgcalls run an
ffmpeg command and read raw frames from its output (see [Quick Start](/quick-start/)).

## Good to know

- **IDs accept `number` or `bigint`.** Pass `chatId` / `userId` either way — `-1001234567890n` or
  `-1001234567890` both work. Values the library returns (like `time()`) come back as `bigint`.
- **It's an EventEmitter.** `NtgCalls` extends Node's `EventEmitter`, so call state, incoming
  frames, and stream-end notifications arrive as ordinary events. Register listeners *before* you
  start a call.
- **Everything async is a Promise.** `create`, `connect`, `set_stream_sources`, etc. return
  promises; heavy native work runs off the main thread so your event loop stays responsive.

## Building from source

You only need this if you're hacking on the binding itself:

1. Install **Rust & Cargo**.
2. Download the matching `libntgcalls` shared library
   (`libntgcalls.so` / `.dylib` / `ntgcalls.dll`) from the
   [ntgcalls releases](https://github.com/pytgcalls/ntgcalls/releases/latest) into `./lib/`.
3. Run `npm run build` to compile the crate into `ntgcalls.node`.
