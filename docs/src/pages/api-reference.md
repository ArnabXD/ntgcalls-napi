---
title: API Reference
order: 4
group: Reference
eyebrow: Reference
description: Every function, event, and method, grouped by what you use it for.
---

# API Reference

All `chatId` / `userId` parameters accept a `number` or a `bigint`. Methods are `async` and return
Promises. For how these fit together, see the [Quick Start](/quick-start/).

## Constants

These are plain numbers — pass the integer value. Listed here because the methods below reference
them.

| Group | Values |
| --- | --- |
| **Stream mode** (`set_stream_sources`) | `0` capture (what you send) · `1` playback |
| **Media source** (`mediaSource`) | `1` file · `2` shell · `4` ffmpeg · `8` device · `16` desktop · `32` external |
| **Stream device** (`streamDevice` in events) | `0` microphone · `1` speaker · `2` camera · `3` screen |
| **Stream type** (`streamType` in `stream-end`) | `0` audio · `1` video |

## Global functions

| Function | Description |
| --- | --- |
| `register_logger(cb)` | Receive native logs from `libntgcalls` (see [`LogMessage`](/data-structures/#logging)). Useful when debugging signaling or transport. |
| `get_version()` | The linked `libntgcalls` version, as a string. |
| `get_protocol()` | The WebRTC layers and capabilities the native library supports (see [`Protocol`](/data-structures/#protocol-device-queries)). |
| `get_media_devices()` | Enumerate local mics, speakers, cameras, and capture devices. |
| `enable_g_lib_loop(enable)` | Toggle GLib main-loop integration. Only relevant on some Linux desktop hosts; leave off unless you know you need it. |

## Events

`NtgCalls` is an `EventEmitter`. Register listeners with `.on('event', cb)`.

> [!IMPORTANT]
> Register listeners **before** calling `create()` or `connect()`, or you'll miss early events.

| Event | Arguments | When it fires |
| --- | --- | --- |
| `stream-end` | `(chatId, streamType, streamDevice)` | A track finished playing. Use it to queue the next one. `streamType`: `0` audio, `1` video. |
| `connection-change` | `(chatId, kind, state)` | The WebRTC connection state changed (connecting, connected, failed, closed). |
| `upgrade` | `(chatId, state)` | Mute/video flags changed — `state` is a [`MediaState`](/data-structures/#webrtc-media-streaming-structures). |
| `signaling-data` | `(chatId, data)` | P2P only — outgoing signaling bytes to relay to the peer. |
| `frames` | `(chatId, mode, device, frames)` | Decoded incoming media frames (only if you opted into receiving them). |
| `remote-source-change` | `(chatId, source)` | A remote participant changed a track — `source` is a [`RemoteSource`](/data-structures/#webrtc-media-streaming-structures). |
| `request-broadcast-timestamp` | `(chatId)` | Broadcast source: the library wants the current segment timestamp. |
| `request-broadcast-part` | `(chatId, request)` | Broadcast source: the library wants a specific chunk — `request` is a [`SegmentPartRequest`](/data-structures/#webrtc-media-streaming-structures). |

## Joining & connecting

#### `create(chatId): Promise<string>`

Creates the call context and returns a **WebRTC offer (SDP)**. Step 1 of the flow — hand this offer
to your MTProto library to join the call.

#### `connect(chatId, params, isPresentation): Promise<void>`

Finishes the handshake using the **answer** (`params`) you got back from Telegram. Pass
`isPresentation: false` for a normal call; `true` for a screen-share/presentation answer.

#### `init_presentation(chatId): Promise<string>`

Starts a screen-share alongside the active call and returns a second offer to join as a
presentation. Pair the resulting answer with `connect(..., true)`.

#### `stop_presentation(chatId): Promise<void>`

Stops the active presentation track.

## Streaming media

#### `set_stream_sources(chatId, streamMode, desc): Promise<void>`

The general way to set tracks. `desc` is a [`MediaDescription`](/data-structures/#media-description-configuration)
with any of `microphone` / `speaker` / `camera` / `screen`. `streamMode` is `0` (capture) for what
you send. See the [video walkthrough](/quick-start/#video).

#### `set_audio_source(chatId, ffmpegCmd): Promise<void>`

Shortcut for the mic-only case: runs `ffmpegCmd` in SHELL mode and reads `s16le` 48 kHz mono audio
from its stdout. Equivalent to `set_stream_sources` with just a `microphone` description.

#### `pause(chatId)` · `resume(chatId)`

Pause or resume the outgoing stream.

#### `mute(chatId)` · `unmute(chatId)`

Mute or unmute. Prefer muting over stopping when you only want a brief silence — see
[Best Practices](/best-practices/).

#### `stop(chatId): Promise<void>`

Stops streaming and tears down the WebRTC connection for that chat.

#### `time(chatId, mode?): Promise<bigint>`

Elapsed playback time of the current media, in milliseconds.

#### `get_state(chatId): Promise<MediaState>`

Current mute/video flags — a [`MediaState`](/data-structures/#webrtc-media-streaming-structures).

#### `get_connection_mode(chatId): Promise<number>`

The active transport mode the connection negotiated.

## Receiving video

For pulling specific remote video tracks (e.g. forwarding someone's camera).

#### `add_incoming_video(chatId, endpoint, ssrcGroupsList): Promise<number>`

Subscribe to a remote video endpoint. `ssrcGroupsList` is an array of
[`SsrcGroup`](/data-structures/#p2p-cryptography-exchange). Returns the assigned sink id.

#### `remove_incoming_video(chatId, endpoint): Promise<void>`

Unsubscribe from that endpoint.

#### `send_external_frame(chatId, device, data, frameData): Promise<void>`

Push a raw frame you produced yourself into the camera/screen track, instead of letting ffmpeg do
it. `frameData` is a [`FrameData`](/data-structures/#webrtc-media-streaming-structures).

## Peer-to-peer (1:1 calls)

A separate flow from group calls, using Diffie-Hellman key exchange. Only needed for direct
user-to-user calls.

#### `create_p2p(userId): Promise<void>`

Start a direct call context with `userId`.

#### `init_exchange(userId, dhConfig, gAHash): Promise<Buffer>`

Begin the DH key exchange. `dhConfig` is a [`DhConfig`](/data-structures/#p2p-cryptography-exchange).

#### `exchange_keys(userId, gAOrB, fingerprint): Promise<AuthParams>`

Finish the exchange; returns the shared [`AuthParams`](/data-structures/#p2p-cryptography-exchange).

#### `skip_exchange(userId, encryptionKey, isOutgoing): Promise<void>`

Skip DH entirely when you already have a negotiated key.

#### `connect_p2p(userId, rtcServers, versionsList, p2PAllowed): Promise<void>`

Connect the P2P call. `rtcServers` is an array of
[`RtcServer`](/data-structures/#p2p-cryptography-exchange).

#### `send_signaling_data(userId, data): Promise<void>`

Feed signaling bytes received from the peer (the counterpart to the `signaling-data` event).

## Broadcasting (RTMP-style sources)

For acting as a broadcast source that serves media in segments on demand. Drive these from the
`request-broadcast-*` events.

#### `send_broadcast_timestamp(chatId, timestamp): Promise<void>`

Answer a `request-broadcast-timestamp` with the current segment timestamp.

#### `send_broadcast_part(chatId, segmentId, partId, status, qualityUpdate, data): Promise<void>`

Answer a `request-broadcast-part` with the requested chunk bytes.

## Diagnostics

#### `cpu_usage(): Promise<number>`

Native CPU usage as a fraction (`0`–`1`).

#### `calls(): Promise<Record<string, CallInfo>>`

Map of active chat ids to their [`CallInfo`](/data-structures/#protocol-device-queries) (capture /
playback status).
