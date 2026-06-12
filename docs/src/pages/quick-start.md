---
title: Quick Start
order: 2
group: Guide
eyebrow: Walkthrough
description: A complete example from session creation to graceful teardown.
---

# Quick Start

Here is a full example showing how to initialize `NtgCalls`, configure connection state handlers,
start a WebRTC session, change streams, and gracefully clean up.

```typescript
import { NtgCalls, get_version, register_logger } from '@arnabxd/ntgcalls-napi';

// 1. (Optional) Register Global Native Logger
register_logger((log) => {
  console.log(`[Native FFI Log] [Level ${log.level}] ${log.file}:${log.line} - ${log.message}`);
});

console.log('Using ntgcalls FFI version:', get_version());

// 2. Initialize the Calls Client
const ntg = new NtgCalls();

// 3. Register Event Listeners (EventEmitter Style)
ntg.on('connection-change', (chatId, kind, state) => {
  console.log(`Connection Changed on Chat ${chatId} | Kind: ${kind} | State: ${state}`);
});

ntg.on('stream-end', (chatId, streamType, streamDevice) => {
  console.log(`Playback finished on Chat ${chatId} | Type: ${streamType} | Device: ${streamDevice}`);
});

// 4. Create a WebRTC Session
const chatId = 1001185324811n; // Native BigInt representation
const offerSdp = await ntg.create(chatId);
console.log('WebRTC Offer SDP generated:\n', offerSdp);

// 5. Connect Session using Answer SDP
const answerSdp = 'v=0...'; // The answer SDP retrieved from Telegram
await ntg.connect(chatId, answerSdp, false);

// 6. Play Audio Stream using shell-based FFmpeg input
const ffmpegCommand = 'ffmpeg -i input.mp3 -f s16le -ac 1 -ar 48000 pipe:1';
await ntg.set_audio_source(chatId, ffmpegCommand);

// 7. Mute / Pause Controls
await ntg.pause(chatId);
await ntg.resume(chatId);
await ntg.mute(chatId);
await ntg.unmute(chatId);

// 8. Graceful Stop
await ntg.stop(chatId);
```

> [!IMPORTANT]
> Always register event listeners **before** initiating a call session with `create()` or
> `connect()`.

Next, browse the [Data Structures](/data-structures/) reference for every interface used above, or
jump to the full [API Reference](/api-reference/).