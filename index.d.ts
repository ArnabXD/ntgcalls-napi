import { EventEmitter } from "node:events";

export interface AudioDescription {
  mediaSource: number;
  input: string;
  sampleRate: number;
  channelCount: number;
  keepOpen: boolean;
}

export interface AuthParams {
  gAOrB: Buffer;
  keyFingerprint: bigint;
}

export interface CallInfo {
  capture: number;
  playback: number;
}

export interface DeviceInfo {
  id: string;
  name: string;
}

export interface DhConfig {
  g: number;
  p: Buffer;
  random: Buffer;
}

export declare function enable_g_lib_loop(enable: boolean): void;

export interface Frame {
  ssrc: bigint;
  data: Buffer;
  frameData: FrameData;
}

export interface FrameData {
  absoluteCaptureTimestampMs: bigint | number;
  width: number;
  height: number;
  rotation: number;
}

export declare function get_media_devices(): MediaDevices;

export declare function get_protocol(): Protocol;

export declare function get_version(): string;

export interface LogMessage {
  level: number;
  source: number;
  file: string;
  line: number;
  message: string;
}

export interface MediaDescription {
  microphone?: AudioDescription;
  speaker?: AudioDescription;
  camera?: VideoDescription;
  screen?: VideoDescription;
}

export interface MediaDevices {
  microphone: Array<DeviceInfo>;
  speaker: Array<DeviceInfo>;
  camera: Array<DeviceInfo>;
  screen: Array<DeviceInfo>;
}

export interface MediaState {
  muted: boolean;
  videoPaused: boolean;
  videoStopped: boolean;
  presentationPaused: boolean;
}

export interface Protocol {
  minLayer: number;
  maxLayer: number;
  udpP2P: boolean;
  udpReflector: boolean;
  libraryVersions: Array<string>;
}

export declare function register_logger(
  cb: (message: LogMessage) => void,
): void;

export interface RemoteSource {
  ssrc: number;
  state: number;
  device: number;
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

export interface SegmentPartRequest {
  segmentId: bigint;
  partId: number;
  limit: number;
  timestamp: bigint;
  qualityUpdate: boolean;
  channelId: number;
  quality: number;
}

export interface SsrcGroup {
  semantics: string;
  ssrcs: Array<number>;
}

export interface VideoDescription {
  mediaSource: number;
  input: string;
  width: number;
  height: number;
  fps: number;
  keepOpen: boolean;
}

export interface NtgCallsEvents {
  "stream-end": (
    chatId: bigint,
    streamType: number,
    streamDevice: number,
  ) => void;
  upgrade: (chatId: bigint, state: MediaState) => void;
  "connection-change": (chatId: bigint, kind: number, state: number) => void;
  "signaling-data": (chatId: bigint, data: Buffer) => void;
  frames: (
    chatId: bigint,
    mode: number,
    device: number,
    frames: Frame[],
  ) => void;
  "remote-source-change": (chatId: bigint, source: RemoteSource) => void;
  "request-broadcast-timestamp": (chatId: bigint) => void;
  "request-broadcast-part": (
    chatId: bigint,
    request: SegmentPartRequest,
  ) => void;
}

export declare class NtgCalls extends EventEmitter {
  constructor();

  on<E extends keyof NtgCallsEvents>(
    event: E,
    listener: NtgCallsEvents[E],
  ): this;
  once<E extends keyof NtgCallsEvents>(
    event: E,
    listener: NtgCallsEvents[E],
  ): this;
  off<E extends keyof NtgCallsEvents>(
    event: E,
    listener: NtgCallsEvents[E],
  ): this;
  emit<E extends keyof NtgCallsEvents>(
    event: E,
    ...args: Parameters<NtgCallsEvents[E]>
  ): boolean;

  send_broadcast_timestamp(
    chatId: number | bigint,
    timestamp: number | bigint,
  ): Promise<void>;
  send_broadcast_part(
    chatId: number | bigint,
    segmentId: number | bigint,
    partId: number,
    status: number,
    qualityUpdate: boolean,
    data: Buffer,
  ): Promise<void>;
  create_p2p(userId: number | bigint): Promise<void>;
  init_exchange(
    userId: number | bigint,
    dhConfig: DhConfig,
    gAHash: Buffer,
  ): Promise<Buffer>;
  exchange_keys(
    userId: number | bigint,
    gAOrB: Buffer,
    fingerprint: number | bigint,
  ): Promise<AuthParams>;
  skip_exchange(
    userId: number | bigint,
    encryptionKey: Buffer,
    isOutgoing: boolean,
  ): Promise<void>;
  connect_p2p(
    userId: number | bigint,
    rtcServers: Array<RtcServer>,
    versionsList: Array<string>,
    p2PAllowed: boolean,
  ): Promise<void>;
  send_signaling_data(userId: number | bigint, data: Buffer): Promise<void>;
  create(chatId: number | bigint): Promise<string>;
  connect(
    chatId: number | bigint,
    params: string,
    isPresentation: boolean,
  ): Promise<void>;
  init_presentation(chatId: number | bigint): Promise<string>;
  stop_presentation(chatId: number | bigint): Promise<void>;
  set_stream_sources(
    chatId: number | bigint,
    streamMode: number,
    desc: MediaDescription,
  ): Promise<void>;
  set_audio_source(chatId: number | bigint, ffmpegCmd: string): Promise<void>;
  pause(chatId: number | bigint): Promise<void>;
  resume(chatId: number | bigint): Promise<void>;
  mute(chatId: number | bigint): Promise<void>;
  unmute(chatId: number | bigint): Promise<void>;
  stop(chatId: number | bigint): Promise<void>;
  time(
    chatId: number | bigint,
    mode?: number | undefined | null,
  ): Promise<bigint>;
  get_state(chatId: number | bigint): Promise<MediaState>;
  get_connection_mode(chatId: number | bigint): Promise<number>;
  cpu_usage(): Promise<number>;
  calls(): Promise<Record<string, CallInfo>>;
  add_incoming_video(
    chatId: number | bigint,
    endpoint: string,
    ssrcGroupsList: Array<SsrcGroup>,
  ): Promise<number>;
  remove_incoming_video(
    chatId: number | bigint,
    endpoint: string,
  ): Promise<void>;
  send_external_frame(
    chatId: number | bigint,
    device: number,
    data: Buffer,
    frameData: FrameData,
  ): Promise<void>;
}
