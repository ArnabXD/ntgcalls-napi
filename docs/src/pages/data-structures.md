---
title: Data Structures
order: 3
group: Reference
eyebrow: Interfaces
description: Every TypeScript interface exposed by the package.
---

# Data Structures

Every TypeScript interface the package exports. They're fully typed, so your editor will surface
these inline — this page is the at-a-glance reference.

## Media Description Configuration

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

## WebRTC / Media Streaming Structures

```typescript
export interface MediaState {
  muted: boolean;
  videoPaused: boolean;
  videoStopped: boolean;
  presentationPaused: boolean;
}

export interface FrameData {
  absoluteCaptureTimestampMs: bigint | number;
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

## P2P & Cryptography Exchange

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
  id: bigint | number;
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

## Protocol & Device Queries

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

## Logging

```typescript
export interface LogMessage {
  level: number;        // ntg_log_level_enum: 1=DEBUG, 2=INFO, 4=WARNING, 8=ERROR
  source: number;       // ntg_log_source_enum: 1=WEBRTC, 2=SELF
  file: string;         // Native source code file
  line: number;         // Native line number
  message: string;      // Log content
}
```