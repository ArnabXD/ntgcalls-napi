---
title: API Reference
order: 4
group: Reference
eyebrow: Reference
description: Global functions, the NtgCalls class, events, and methods.
---

# API Reference

## Global Utility Functions

#### `register_logger(cb: (message: LogMessage) => void): void`

Registers a global logger callback to receive native events and logs from `libntgcalls`. Highly
recommended for debugging signaling or WebRTC transport issues.

#### `get_version(): string`

Returns the version of `libntgcalls` currently linked.

#### `get_protocol(): Protocol`

Retrieves WebRTC layers, capabilities, and version signatures supported by the native library.

#### `get_media_devices(): MediaDevices`

Queries the hardware for available input/output devices (microphones, speakers, webcams, capture
devices).

#### `enable_g_lib_loop(enable: boolean): void`

Toggles GLib main loop integration. Use with discretion depending on native hosting targets.

---

## `NtgCalls` Class

### Events

`NtgCalls` extends the standard Node.js `EventEmitter`. Register listeners with
`.on('event-name', callback)`.

> [!IMPORTANT]
> Always register event listeners **before** initiating a call session with `create()` or
> `connect()`.

| Event | Callback signature | Description |
| --- | --- | --- |
| `'stream-end'` | `(chatId: bigint, streamType: number, streamDevice: number) => void` | Triggers when a playback stream is exhausted or terminated. |
| `'upgrade'` | `(chatId: bigint, state: MediaState) => void` | Fires when stream characteristics or muted states are toggled. |
| `'connection-change'` | `(chatId: bigint, kind: number, state: number) => void` | Fires when WebRTC connectivity states shift. |
| `'signaling-data'` | `(chatId: bigint, data: Buffer) => void` | Fires when new signaling parameters are ready to dispatch to remote participants. |
| `'frames'` | `(chatId: bigint, mode: number, device: number, frames: Array<Frame>) => void` | Delivers raw incoming media frames captured by WebRTC components. |
| `'remote-source-change'` | `(chatId: bigint, source: RemoteSource) => void` | Fires when a remote participant alters their media device settings. |
| `'request-broadcast-timestamp'` | `(chatId: bigint) => void` | Requested when synchronizing broadcast components. |
| `'request-broadcast-part'` | `(chatId: bigint, request: SegmentPartRequest) => void` | Triggers when broadcast streaming demands chunk updates. |

---

### Active Session Operations

#### `create(chatId: number | bigint): Promise<string>`

Initializes a calls context for a designated chat and returns an **Offer SDP** string.

#### `connect(chatId: number | bigint, params: string, isPresentation: boolean): Promise<void>`

Completes the WebRTC handshake using the remote **Answer SDP** string (`params`).

#### `init_presentation(chatId: number | bigint): Promise<string>`

Starts presentation/screen sharing within the active session, generating a secondary Offer SDP.

#### `stop_presentation(chatId: number | bigint): Promise<void>`

Terminates the active presentation stream.

---

### Stream Control Operations

#### `set_stream_sources(chatId: number | bigint, streamMode: number, desc: MediaDescription): Promise<void>`

Sets the primary microphone, speaker, camera, and screen streams in `streamMode`
(0 = CAPTURE, 1 = PLAYBACK).

#### `set_audio_source(chatId: number | bigint, ffmpegCmd: string): Promise<void>`

Convenience wrapper that configures raw shell execution to feed audio playback to the session.

#### `pause(chatId: number | bigint): Promise<void>`

Pauses active streaming on the specified chat.

#### `resume(chatId: number | bigint): Promise<void>`

Resumes streaming on the specified chat.

#### `mute(chatId: number | bigint): Promise<void>`

Mutes the client.

#### `unmute(chatId: number | bigint): Promise<void>`

Unmutes the client.

#### `stop(chatId: number | bigint): Promise<void>`

Safely stops the stream and tears down WebRTC allocations for the designated chat.

#### `time(chatId: number | bigint, mode?: number): Promise<bigint>`

Returns the absolute elapsed playback time of active media.

#### `get_state(chatId: number | bigint): Promise<MediaState>`

Returns the current active media state structure (mute and video statuses).

#### `get_connection_mode(chatId: number | bigint): Promise<number>`

Returns the active transport mode (e.g. RTC, RTMP, etc.).

---

### Advanced Peer-to-Peer & Cryptography

#### `create_p2p(userId: number | bigint): Promise<void>`

Initializes a direct P2P media context for a direct call session with `userId`.

#### `init_exchange(userId: number | bigint, dhConfig: DhConfig, gAHash: Buffer): Promise<Buffer>`

Initializes Diffie-Hellman cryptographic exchange with a peer.

#### `exchange_keys(userId: number | bigint, gAOrB: Buffer, fingerprint: number | bigint): Promise<AuthParams>`

Completes the cryptographic handshake, generating the key fingerprint and auth values.

#### `skip_exchange(userId: number | bigint, encryptionKey: Buffer, isOutgoing: boolean): Promise<void>`

Bypasses Diffie-Hellman exchange using a pre-negotiated secure encryption key.

#### `connect_p2p(userId: number | bigint, rtcServers: Array<RtcServer>, versionsList: Array<string>, p2PAllowed: boolean): Promise<void>`

Completes the connection setup for a direct P2P session.

#### `send_signaling_data(userId: number | bigint, data: Buffer): Promise<void>`

Feeds remote signaling credentials into the P2P connection tracker.

#### `add_incoming_video(chatId: number | bigint, endpoint: string, ssrcGroupsList: Array<SsrcGroup>): Promise<number>`

Registers an incoming video channel configuration from a remote participant.

#### `remove_incoming_video(chatId: number | bigint, endpoint: string): Promise<void>`

Deregisters the video channel associated with the endpoint.

#### `send_external_frame(chatId: number | bigint, device: number, data: Buffer, frameData: FrameData): Promise<void>`

Manually feeds a custom video frame payload (`data`) directly into the WebRTC camera or screen
stream channel.

#### `send_broadcast_timestamp(chatId: number | bigint, timestamp: number | bigint): Promise<void>`

Broadcasters use this to push synchronized timestamps.

#### `send_broadcast_part(chatId: number | bigint, segmentId: number | bigint, partId: number, status: number, qualityUpdate: boolean, data: Buffer): Promise<void>`

Feeds video broadcast chunks into active subscriber pipelines.

---

### System Diagnostics

#### `cpu_usage(): Promise<number>`

Returns the relative CPU usage fraction of the native library routines.

#### `calls(): Promise<Record<string, CallInfo>>`

Returns a catalog mapping active Telegram chat IDs to their active capture/playback stream statuses.