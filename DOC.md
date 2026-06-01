# Client Integration Guide: @arnabxd/ntgcalls-napi

This document details the architecture, API reference, and integration best practices for using the `@arnabxd/ntgcalls-napi` package. It provides native, thread-safe, high-performance Node-API (N-API) bindings in Rust for `libntgcalls` (the native WebRTC C-shared library).

---

## 1. Architectural Overview

The library wraps native `libntgcalls` code into a robust, non-blocking asynchronous architecture optimized for Node.js:

```
┌────────────────────────────────────────────────────────┐
│                      Node.js / JS                      │
│   Event Loop / Main Thread (JavaScript / TypeScript)   │
└───────────────────────────┬────────────────────────────┘
     ▲                      │                      │
     │ Thread-safe          │ Async Promises       │ Callback Register
     │ Callbacks            ▼                      ▼
┌────┴───────────────────────────────────────────────────┐
│                    N-API FFI Wrapper                   │
│        (Safe Rust Integration, thread-safe-fns)        │
└───────────────────────────┬────────────────────────────┘
     ▲                      │                      │
     │ Native Worker        │ spawn_blocking       │ C FFI
     │ Signals              ▼                      ▼
┌────┴───────────────────────────────────────────────────┐
│                      libntgcalls                       │
│    Background C++ WebRTC Threads, Key Exchange, etc.    │
└────────────────────────────────────────────────────────┘
```

### Key Architectural Concepts
- **Thread Safety**: WebRTC signaling and media processing run on background native C++ threads. To safely deliver notifications to JavaScript, the wrapper utilizes N-API `ThreadsafeFunction` channels, converting native thread dispatches into event-loop tasks.
- **Non-Blocking Async**: Heavy operations (e.g. SDP generation, key exchange) are delegated to a Tokio threadpool via `spawn_blocking` with safe one-shot synchronization channels.
- **BigInt Representation**: Telegram Chat IDs, User IDs, and WebRTC timestamps are represented using native JavaScript `bigint` types to prevent the 53-bit floating-point precision limit overflow in normal JavaScript `Number`s.
- **Memory Safety**: Wrapper context allocations (e.g. C-string parameters, vector structures) are maintained in native heap context structures (`Keeper` types) and safely freed automatically once native callbacks complete.

---

## 2. Quick Start Example

Here is a full example showing how to initialize `NtgCalls`, configure connection state handlers, start a WebRTC session, change streams, and gracefully clean up.

```typescript
import { NtgCalls, get_version, register_logger } from '@arnabxd/ntgcalls-napi';

// 1. (Optional) Register Global Native Logger
register_logger((log) => {
  console.log(`[Native FFI Log] [Level ${log.level}] ${log.file}:${log.line} - ${log.message}`);
});

console.log('Using ntgcalls FFI version:', get_version());

// 2. Initialize the Calls Client
const ntg = new NtgCalls();

// 3. Register Event Listeners
ntg.on_connection_change((chatId, kind, state) => {
  console.log(`Connection Changed on Chat ${chatId} | Kind: ${kind} | State: ${state}`);
});

ntg.on_stream_end((chatId, streamType, streamDevice) => {
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

---

## 3. Data Structures & Interfaces

Below is the complete set of TypeScript interfaces exposed by the package:

### Media Description Configuration

```typescript
export interface AudioDescription {
  mediaSource: number;  // 1=FILE, 2=SHELL, 4=FFMPEG, 8=DEVICE, 16=DESKTOP, 32=EXTERNAL
  input: string;        // File path, FFmpeg input string, or device identifier
  sampleRate: number;   // e.g. 48000 or 96000
  channelCount: number; // e.g. 1 (mono) or 2 (stereo)
  keepOpen: boolean;    // Keep audio track open after stream exhaustion
}

export interface VideoDescription {
  mediaSource: number;  // Source bitmask
  input: string;        // File path or FFmpeg stream
  width: number;        // e.g. 1280
  height: number;       // e.g. 720
  fps: number;          // e.g. 30
  keepOpen: boolean;    // Keep track open
}

export interface MediaDescription {
  microphone?: AudioDescription;
  speaker?: AudioDescription;
  camera?: VideoDescription;
  screen?: VideoDescription;
}
```

### WebRTC / Media Streaming Structures

```typescript
export interface MediaState {
  muted: boolean;
  videoPaused: boolean;
  videoStopped: boolean;
  presentationPaused: boolean;
}

export interface FrameData {
  absoluteCaptureTimestampMs: bigint;
  width: number;
  height: number;
  rotation: number;
}

export interface Frame {
  ssrc: bigint;
  data: Buffer;
  frameData: FrameData;
}

export interface RemoteSource {
  ssrc: number;
  state: number;        // ntg_stream_status_enum: 0=ACTIVE, 1=PAUSED, 2=IDLING
  device: number;       // ntg_stream_device_enum: 0=MIC, 1=SPEAKER, 2=CAMERA, 3=SCREEN
}

export interface SegmentPartRequest {
  segmentId: bigint;
  partId: number;
  limit: number;
  timestamp: bigint;
  qualityUpdate: boolean;
  channelId: number;
  quality: number;      // ntg_media_segment_quality_enum
}
```

### P2P & Cryptography Exchange

```typescript
export interface DhConfig {
  g: number;
  p: Buffer;
  random: Buffer;
}

export interface AuthParams {
  gAOrB: Buffer;
  keyFingerprint: bigint;
}

export interface RtcServer {
  id: bigint;
  ipv4?: string;
  ipv6?: string;
  username?: string;
  password?: string;
  port: number;
  turn: boolean;
  stun: boolean;
  tcp: boolean;
  peerTag?: Buffer;
}

export interface SsrcGroup {
  semantics: string;
  ssrcs: Array<number>;
}
```

### Protocol & Device Queries

```typescript
export interface Protocol {
  minLayer: number;
  maxLayer: number;
  udpP2P: boolean;
  udpReflector: boolean;
  libraryVersions: Array<string>;
}

export interface DeviceInfo {
  id: string;
  name: string;
}

export interface MediaDevices {
  microphone: Array<DeviceInfo>;
  speaker: Array<DeviceInfo>;
  camera: Array<DeviceInfo>;
  screen: Array<DeviceInfo>;
}

export interface CallInfo {
  capture: number;
  playback: number;
}
```

### Logging Configuration

```typescript
export interface LogMessage {
  level: number;        // ntg_log_level_enum: 1=DEBUG, 2=INFO, 4=WARNING, 8=ERROR
  source: number;       // ntg_log_source_enum: 1=WEBRTC, 2=SELF
  file: string;         // Native source code file
  line: number;         // Native line number
  message: string;      // Log content
}
```

---

## 4. API Reference

### Global Utility Functions

#### `register_logger(cb: (message: LogMessage) => void): void`
Registers a global logger callback to receive native events and logs from `libntgcalls`. Highly recommended for debugging signaling or WebRTC transport issues.

#### `get_version(): string`
Returns the version of `libntgcalls` currently linked.

#### `get_protocol(): Protocol`
Retrieves WebRTC layers, capabilities, and version signatures supported by the native library.

#### `get_media_devices(): MediaDevices`
Queries the hardware for available input/output devices (microphones, speakers, webcams, capture devices).

#### `enable_g_lib_loop(enable: boolean): void`
Toggles GLib main loop integration. Use with discretion depending on native hosting targets.

---

### `NtgCalls` Class Methods

#### Callbacks

> [!IMPORTANT]
> Always register event listeners **before** initiating a call session with `create()` or `connect()`.

##### `on_stream_end(cb: (chatId: bigint, streamType: number, streamDevice: number) => void): void`
Triggers when a playback stream is exhausted or terminated.

##### `on_upgrade(cb: (chatId: bigint, state: MediaState) => void): void`
Fires when stream characteristics or muted states are toggled.

##### `on_connection_change(cb: (chatId: bigint, kind: number, state: number) => void): void`
Fires when WebRTC connectivity states shift.

##### `on_signaling_data(cb: (chatId: bigint, data: Buffer) => void): void`
Fires when new signaling parameters are ready to be dispatched to remote participants.

##### `on_frames(cb: (chatId: bigint, mode: number, device: number, frames: Array<Frame>) => void): void`
Delivers raw incoming media frames captured by WebRTC components.

##### `on_remote_source_change(cb: (chatId: bigint, source: RemoteSource) => void): void`
Fires when a remote WebRTC participant alters their media device settings.

##### `on_request_broadcast_timestamp(cb: (chatId: bigint) => void): void`
Requested when synchronizing broadcast components.

##### `on_request_broadcast_part(cb: (chatId: bigint, request: SegmentPartRequest) => void): void`
Triggers when broadcast streaming demands chunk updates.

---

#### Active Session Operations

##### `create(chatId: bigint): Promise<string>`
Initializes a calls context for a designated chat and returns an **Offer SDP** string.

##### `connect(chatId: bigint, params: string, isPresentation: boolean): Promise<void>`
Completes the WebRTC handshake using the remote **Answer SDP** string (`params`).

##### `init_presentation(chatId: bigint): Promise<string>`
Starts presentation/screen sharing within the active session, generating a secondary Offer SDP.

##### `stop_presentation(chatId: bigint): Promise<void>`
Terminates the active presentation stream.

---

#### Stream Control Operations

##### `set_stream_sources(chatId: bigint, streamMode: number, desc: MediaDescription): Promise<void>`
Sets the primary microphone, speaker, camera, and screen streams in `streamMode` (0 = CAPTURE, 1 = PLAYBACK).

##### `set_audio_source(chatId: bigint, ffmpegCmd: string): Promise<void>`
Convenience wrapper that configures raw shell execution to feed audio playback to the session.

##### `pause(chatId: bigint): Promise<void>`
Pauses active streaming on the specified chat.

##### `resume(chatId: bigint): Promise<void>`
Resumes streaming on the specified chat.

##### `mute(chatId: bigint): Promise<void>`
Mutes the client.

##### `unmute(chatId: bigint): Promise<void>`
Unmutes the client.

##### `stop(chatId: bigint): Promise<void>`
Safely stops the stream and tears down WebRTC allocations for the designated chat.

##### `time(chatId: bigint, mode?: number): Promise<bigint>`
Returns the absolute elapsed playback time of active media.

##### `get_state(chatId: bigint): Promise<MediaState>`
Returns the current active media state structure (mute and video statuses).

##### `get_connection_mode(chatId: bigint): Promise<number>`
Returns the active transport mode (e.g. RTC, RTMP, etc.).

---

#### Advanced Peer-to-Peer & Cryptography

##### `create_p2p(userId: bigint): Promise<void>`
Initializes a direct P2P media context for a direct call session with `userId`.

##### `init_exchange(userId: bigint, dhConfig: DhConfig, gAHash: Buffer): Promise<Buffer>`
Initializes Diffie-Hellman cryptographic exchange with a peer.

##### `exchange_keys(userId: bigint, gAOrB: Buffer, fingerprint: bigint): Promise<AuthParams>`
Completes the cryptographic handshake, generating the key fingerprint and auth values.

##### `skip_exchange(userId: bigint, encryptionKey: Buffer, isOutgoing: boolean): Promise<void>`
Bypasses Diffie-Hellman exchange using a pre-negotiated secure encryption key.

##### `connect_p2p(userId: bigint, rtcServers: Array<RtcServer>, versionsList: Array<string>, p2PAllowed: boolean): Promise<void>`
Completes the connection setup for a direct P2P session.

##### `send_signaling_data(userId: bigint, data: Buffer): Promise<void>`
Feeds remote signaling credentials into the P2P connection tracker.

##### `add_incoming_video(chatId: bigint, endpoint: string, ssrcGroupsList: Array<SsrcGroup>): Promise<number>`
Registers a incoming video channel configuration from a remote participant.

##### `remove_incoming_video(chatId: bigint, endpoint: string): Promise<void>`
Deregisters the video channel associated with the endpoint.

##### `send_external_frame(chatId: bigint, device: number, data: Buffer, frameData: FrameData): Promise<void>`
Manually feeds a custom video frame payload (`data`) directly into the WebRTC camera or screen stream channel.

##### `send_broadcast_timestamp(chatId: bigint, timestamp: bigint): Promise<void>`
Broadcasters use this to push synchronized timestamps.

##### `send_broadcast_part(chatId: bigint, segmentId: bigint, partId: number, status: number, qualityUpdate: boolean, data: Buffer): Promise<void>`
Feeds video broadcast chunks into active subscriber pipelines.

---

#### System Diagnostics

##### `cpu_usage(): Promise<number>`
Returns the relative CPU usage fraction of the native library routines.

##### `calls(): Promise<Record<string, CallInfo>>`
Returns a catalog mapping active Telegram chat IDs to their active capture/playback stream statuses.

---

## 5. Integration Best Practices

### A. JavaScript/TypeScript Event Loop Preservation
Native callbacks invoke JavaScript asynchronously. Keep your callbacks **lightweight and fast**. If you need to perform intensive disk I/O or heavy networking on event notifications, delegate them to standard microtasks (using `setImmediate`, `setTimeout`, or secondary worker threads) to prevent stalling the main event loop.

### B. Muting/Pausing Graceful States
When a track finishes, the `on_stream_end` callback fires. Clients must capture this state to queue the next audio track in their music-bot playlist.
- Ensure that you mute or stop playback before issuing a secondary `set_stream_sources` or `set_audio_source` payload to prevent audio buffer overlapping or clicking sounds in the voice chat channel.

### C. BigInt Precision
Never attempt to convert `chatId` or `userId` parameter parameters into JavaScript floating-point `Number`s. Telegram IDs frequently exceed the maximum safe integer limit (`Number.MAX_SAFE_INTEGER`). Always use `BigInt` suffixes (e.g. `1234567890n`) or create them using `BigInt("chat_id_string")`.
