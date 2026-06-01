use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi(object)]
pub struct MediaState {
    pub muted: bool,
    pub video_paused: bool,
    pub video_stopped: bool,
    pub presentation_paused: bool,
}

#[napi(object)]
pub struct Frame {
    #[napi(ts_type = "bigint")]
    pub ssrc: i64,
    pub data: Buffer,
    pub frame_data: FrameData,
}

#[napi(object)]
pub struct FrameData {
    #[napi(ts_type = "bigint")]
    pub absolute_capture_timestamp_ms: i64,
    pub width: u16,
    pub height: u16,
    pub rotation: u16,
}

#[napi(object)]
pub struct RemoteSource {
    pub ssrc: u32,
    pub state: i32,
    pub device: i32,
}

#[napi(object)]
pub struct SegmentPartRequest {
    #[napi(ts_type = "bigint")]
    pub segment_id: i64,
    pub part_id: i32,
    pub limit: i32,
    #[napi(ts_type = "bigint")]
    pub timestamp: i64,
    pub quality_update: bool,
    pub channel_id: i32,
    pub quality: i32,
}

#[napi(object)]
pub struct AudioDescription {
    pub media_source: i32,
    pub input: String,
    pub sample_rate: u32,
    pub channel_count: u8,
    pub keep_open: bool,
}

#[napi(object)]
pub struct VideoDescription {
    pub media_source: i32,
    pub input: String,
    pub width: i16,
    pub height: i16,
    pub fps: u8,
    pub keep_open: bool,
}

#[napi(object)]
pub struct MediaDescription {
    pub microphone: Option<AudioDescription>,
    pub speaker: Option<AudioDescription>,
    pub camera: Option<VideoDescription>,
    pub screen: Option<VideoDescription>,
}

#[napi(object)]
pub struct SsrcGroup {
    pub semantics: String,
    pub ssrcs: Vec<u32>,
}

#[napi(object)]
pub struct DhConfig {
    pub g: i32,
    pub p: Buffer,
    pub random: Buffer,
}

#[napi(object)]
pub struct AuthParams {
    pub g_a_or_b: Buffer,
    #[napi(ts_type = "bigint")]
    pub key_fingerprint: i64,
}

#[napi(object)]
pub struct RtcServer {
    #[napi(ts_type = "bigint")]
    pub id: i64,
    pub ipv4: Option<String>,
    pub ipv6: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub port: u16,
    pub turn: bool,
    pub stun: bool,
    pub tcp: bool,
    pub peer_tag: Option<Buffer>,
}

#[napi(object)]
pub struct Protocol {
    pub min_layer: i32,
    pub max_layer: i32,
    pub udp_p2p: bool,
    pub udp_reflector: bool,
    pub library_versions: Vec<String>,
}

#[napi(object)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
}

#[napi(object)]
pub struct MediaDevices {
    pub microphone: Vec<DeviceInfo>,
    pub speaker: Vec<DeviceInfo>,
    pub camera: Vec<DeviceInfo>,
    pub screen: Vec<DeviceInfo>,
}

#[napi(object)]
pub struct CallInfo {
    pub capture: i32,
    pub playback: i32,
}

#[napi(object)]
pub struct LogMessage {
    pub level: i32,
    pub source: i32,
    pub file: String,
    pub line: u32,
    pub message: String,
}
