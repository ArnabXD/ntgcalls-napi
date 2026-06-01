/// <reference types="node" />

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
  state: number;
  device: number;
}

export interface SegmentPartRequest {
  segmentId: bigint;
  partId: number;
  limit: number;
  timestamp: bigint;
  qualityUpdate: boolean;
  channelId: number;
  quality: number;
}

export interface AudioDescription {
  mediaSource: number;
  input: string;
  sampleRate: number;
  channelCount: number;
  keepOpen: boolean;
}

export interface VideoDescription {
  mediaSource: number;
  input: string;
  width: number;
  height: number;
  fps: number;
  keepOpen: boolean;
}

export interface MediaDescription {
  microphone?: AudioDescription;
  speaker?: AudioDescription;
  camera?: VideoDescription;
  screen?: VideoDescription;
}

export interface SsrcGroup {
  semantics: string;
  ssrcs: Array<number>;
}

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

export class NtgCalls {
  constructor();

  on_stream_end(
    cb: (chatId: bigint, streamType: number, streamDevice: number) => void,
  ): void;

  on_upgrade(cb: (chatId: bigint, state: MediaState) => void): void;

  on_connection_change(
    cb: (chatId: bigint, kind: number, state: number) => void,
  ): void;

  on_signaling_data(cb: (chatId: bigint, data: Buffer) => void): void;

  on_frames(
    cb: (
      chatId: bigint,
      mode: number,
      device: number,
      frames: Array<Frame>,
    ) => void,
  ): void;

  on_remote_source_change(
    cb: (chatId: bigint, source: RemoteSource) => void,
  ): void;

  on_request_broadcast_timestamp(cb: (chatId: bigint) => void): void;

  on_request_broadcast_part(
    cb: (chatId: bigint, request: SegmentPartRequest) => void,
  ): void;

  create(chatId: bigint): Promise<string>;

  connect(
    chatId: bigint,
    params: string,
    isPresentation: boolean,
  ): Promise<void>;

  init_presentation(chatId: bigint): Promise<string>;

  stop_presentation(chatId: bigint): Promise<void>;

  set_stream_sources(
    chatId: bigint,
    streamMode: number,
    desc: MediaDescription,
  ): Promise<void>;

  set_audio_source(chatId: bigint, ffmpegCmd: string): Promise<void>;

  pause(chatId: bigint): Promise<void>;

  resume(chatId: bigint): Promise<void>;

  mute(chatId: bigint): Promise<void>;

  unmute(chatId: bigint): Promise<void>;

  stop(chatId: bigint): Promise<void>;

  time(chatId: bigint, mode?: number): Promise<bigint>;

  get_state(chatId: bigint): Promise<MediaState>;

  get_connection_mode(chatId: bigint): Promise<number>;

  add_incoming_video(
    chatId: bigint,
    endpoint: string,
    ssrcGroupsList: Array<SsrcGroup>,
  ): Promise<number>;

  remove_incoming_video(chatId: bigint, endpoint: string): Promise<void>;

  create_p2p(userId: bigint): Promise<void>;

  init_exchange(
    userId: bigint,
    dhConfig: DhConfig,
    gAHash: Buffer,
  ): Promise<Buffer>;

  exchange_keys(
    userId: bigint,
    gAOrB: Buffer,
    fingerprint: bigint,
  ): Promise<AuthParams>;

  skip_exchange(
    userId: bigint,
    encryptionKey: Buffer,
    isOutgoing: boolean,
  ): Promise<void>;

  connect_p2p(
    userId: bigint,
    rtcServers: Array<RtcServer>,
    versionsList: Array<string>,
    p2pAllowed: boolean,
  ): Promise<void>;

  send_signaling_data(userId: bigint, data: Buffer): Promise<void>;

  send_external_frame(
    chatId: bigint,
    device: number,
    data: Buffer,
    frameData: FrameData,
  ): Promise<void>;

  send_broadcast_timestamp(chatId: bigint, timestamp: bigint): Promise<void>;

  send_broadcast_part(
    chatId: bigint,
    segmentId: bigint,
    partId: number,
    status: number,
    qualityUpdate: boolean,
    data: Buffer,
  ): Promise<void>;

  cpu_usage(): Promise<number>;

  calls(): Promise<Record<string, CallInfo>>;
}

export function get_version(): string;
export function get_protocol(): Protocol;
export function enable_g_lib_loop(enable: boolean): void;
export function get_media_devices(): MediaDevices;
export interface LogMessage {
  level: number;
  source: number;
  file: string;
  line: number;
  message: string;
}
export function register_logger(cb: (message: LogMessage) => void): void;
