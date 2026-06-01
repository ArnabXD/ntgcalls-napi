use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::oneshot;

// ── C API Function Signatures ──────────────────────────────────────────────

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_network_info_struct {
    pub kind: i32,
    pub state: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_audio_description_struct {
    pub media_source: i32,
    pub input: *const c_char,
    pub sample_rate: u32,
    pub channel_count: u8,
    pub keep_open: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_video_description_struct {
    pub media_source: i32,
    pub input: *const c_char,
    pub width: i16,
    pub height: i16,
    pub fps: u8,
    pub keep_open: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_auth_params_struct {
    pub g_a_or_b: *mut u8,
    pub size_gab: i32,
    pub key_fingerprint: i64,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_media_description_struct {
    pub microphone: *mut ntg_audio_description_struct,
    pub speaker: *mut ntg_audio_description_struct,
    pub camera: *mut ntg_video_description_struct,
    pub screen: *mut ntg_video_description_struct,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_call_info_struct {
    pub chat_id: i64,
    pub capture: i32,
    pub playback: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_media_state_struct {
    pub muted: bool,
    pub video_paused: bool,
    pub video_stopped: bool,
    pub presentation_paused: bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_rtc_server_struct {
    pub id: u64,
    pub ipv4: *mut c_char,
    pub ipv6: *mut c_char,
    pub username: *mut c_char,
    pub password: *mut c_char,
    pub port: u16,
    pub turn: bool,
    pub stun: bool,
    pub tcp: bool,
    pub peer_tag: *mut u8,
    pub peer_tag_size: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_protocol_struct {
    pub min_layer: i32,
    pub max_layer: i32,
    pub udp_p2p: bool,
    pub udp_reflector: bool,
    pub library_versions: *mut *mut c_char,
    pub library_versions_size: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_dh_config_struct {
    pub g: i32,
    pub p: *const u8,
    pub size_p: i32,
    pub random: *const u8,
    pub size_random: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_frame_data_struct {
    pub absolute_capture_timestamp_ms: i64,
    pub width: u16,
    pub height: u16,
    pub rotation: u16,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_remote_source_struct {
    pub ssrc: u32,
    pub state: i32,
    pub device: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_ssrc_group_struct {
    pub semantics: *mut c_char,
    pub ssrcs: *mut u32,
    pub size_ssrcs: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_device_info_struct {
    pub name: *mut c_char,
    pub metadata: *mut c_char,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_media_devices_struct {
    pub microphone: *mut ntg_device_info_struct,
    pub size_microphone: i32,
    pub speaker: *mut ntg_device_info_struct,
    pub size_speaker: i32,
    pub camera: *mut ntg_device_info_struct,
    pub size_camera: i32,
    pub screen: *mut ntg_device_info_struct,
    pub size_screen: i32,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_frame_struct {
    pub ssrc: i64,
    pub data: *mut u8,
    pub size_data: i32,
    pub frame_data: ntg_frame_data_struct,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_segment_part_request_struct {
    pub segment_id: i64,
    pub part_id: i32,
    pub limit: i32,
    pub timestamp: i64,
    pub quality_update: bool,
    pub channel_id: i32,
    pub quality: i32,
}

#[repr(C)]
pub struct NtgAsyncStruct {
    pub user_data: *mut c_void,
    pub error_code: *mut i32,
    pub error_message: *mut *mut c_char,
    pub promise: Option<unsafe extern "C" fn(*mut c_void)>,
}

unsafe impl Send for NtgAsyncStruct {}
unsafe impl Sync for NtgAsyncStruct {}

pub type NtgStreamCallback = unsafe extern "C" fn(
    pointer: usize,
    chat_id: i64,
    stream_type: i32,
    stream_device: i32,
    user_data: *mut c_void,
);

pub type NtgUpgradeCallback = unsafe extern "C" fn(
    pointer: usize,
    chat_id: i64,
    state: ntg_media_state_struct,
    user_data: *mut c_void,
);

pub type NtgConnectionCallback = unsafe extern "C" fn(
    pointer: usize,
    chat_id: i64,
    info: ntg_network_info_struct,
    user_data: *mut c_void,
);

pub type NtgSignalingCallback = unsafe extern "C" fn(
    pointer: usize,
    chat_id: i64,
    data: *mut u8,
    size: i32,
    user_data: *mut c_void,
);

pub type NtgFrameCallback = unsafe extern "C" fn(
    pointer: usize,
    chat_id: i64,
    mode: i32,
    device: i32,
    frames: *mut ntg_frame_struct,
    size: u64,
    user_data: *mut c_void,
);

pub type NtgRemoteSourceCallback = unsafe extern "C" fn(
    pointer: usize,
    chat_id: i64,
    source: ntg_remote_source_struct,
    user_data: *mut c_void,
);

pub type NtgBroadcastTimestampCallback =
    unsafe extern "C" fn(pointer: usize, chat_id: i64, user_data: *mut c_void);

pub type NtgBroadcastPartCallback = unsafe extern "C" fn(
    pointer: usize,
    chat_id: i64,
    request: ntg_segment_part_request_struct,
    user_data: *mut c_void,
);

#[derive(Copy, Clone)]
#[repr(C)]
pub struct ntg_log_message_struct {
    pub level: i32,
    pub source: i32,
    pub file: *mut c_char,
    pub line: u32,
    pub message: *mut c_char,
}

pub type NtgLogCallback = unsafe extern "C" fn(message: ntg_log_message_struct);

extern "C" {
    pub fn ntg_init() -> usize;
    pub fn ntg_destroy(ptr: usize) -> i32;

    pub fn ntg_create(
        ptr: usize,
        chat_id: i64,
        buffer: *mut *mut c_char,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_connect(
        ptr: usize,
        chat_id: i64,
        params: *const c_char,
        is_presentation: bool,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_set_stream_sources(
        ptr: usize,
        chat_id: i64,
        mode: i32,
        desc: ntg_media_description_struct,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_pause(ptr: usize, chat_id: i64, future: NtgAsyncStruct) -> i32;
    pub fn ntg_resume(ptr: usize, chat_id: i64, future: NtgAsyncStruct) -> i32;
    pub fn ntg_mute(ptr: usize, chat_id: i64, future: NtgAsyncStruct) -> i32;
    pub fn ntg_unmute(ptr: usize, chat_id: i64, future: NtgAsyncStruct) -> i32;
    pub fn ntg_stop(ptr: usize, chat_id: i64, future: NtgAsyncStruct) -> i32;
    pub fn ntg_time(
        ptr: usize,
        chat_id: i64,
        mode: i32,
        time: *mut i64,
        future: NtgAsyncStruct,
    ) -> i32;

    pub fn ntg_init_presentation(
        ptr: usize,
        chat_id: i64,
        buffer: *mut *mut c_char,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_stop_presentation(ptr: usize, chat_id: i64, future: NtgAsyncStruct) -> i32;
    pub fn ntg_get_state(
        ptr: usize,
        chat_id: i64,
        media_state: *mut ntg_media_state_struct,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_get_connection_mode(
        ptr: usize,
        chat_id: i64,
        mode: *mut i32,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_add_incoming_video(
        ptr: usize,
        chat_id: i64,
        endpoint: *const c_char,
        ssrc_groups: *mut ntg_ssrc_group_struct,
        size: i32,
        buffer: *mut u32,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_remove_incoming_video(
        ptr: usize,
        chat_id: i64,
        endpoint: *const c_char,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_create_p2p(ptr: usize, user_id: i64, future: NtgAsyncStruct) -> i32;
    pub fn ntg_init_exchange(
        ptr: usize,
        user_id: i64,
        dh_config: *mut ntg_dh_config_struct,
        g_a_hash: *const u8,
        size_g_a_hash: i32,
        buffer: *mut *mut u8,
        size: *mut i32,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_exchange_keys(
        ptr: usize,
        user_id: i64,
        g_a_or_b: *const u8,
        size_gab: i32,
        fingerprint: i64,
        buffer: *mut ntg_auth_params_struct,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_skip_exchange(
        ptr: usize,
        user_id: i64,
        encryption_key: *const u8,
        size: i32,
        is_outgoing: bool,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_connect_p2p(
        ptr: usize,
        user_id: i64,
        servers: *mut ntg_rtc_server_struct,
        servers_size: i32,
        versions: *mut *mut c_char,
        versions_size: i32,
        p2p_allowed: bool,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_send_signaling_data(
        ptr: usize,
        user_id: i64,
        buffer: *mut u8,
        size: i32,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_get_protocol(buffer: *mut ntg_protocol_struct) -> i32;
    pub fn ntg_send_external_frame(
        ptr: usize,
        chat_id: i64,
        device: i32,
        frame: *mut u8,
        frame_size: i32,
        frame_data: ntg_frame_data_struct,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_send_broadcast_timestamp(
        ptr: usize,
        chat_id: i64,
        timestamp: i64,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_send_broadcast_part(
        ptr: usize,
        chat_id: i64,
        segment_id: i64,
        part_id: i32,
        status: i32,
        quality_update: bool,
        frame: *const u8,
        frame_size: i32,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_get_media_devices(buffer: *mut ntg_media_devices_struct) -> i32;
    pub fn ntg_calls(
        ptr: usize,
        buffer: *mut *mut ntg_call_info_struct,
        size: *mut i32,
        future: NtgAsyncStruct,
    ) -> i32;
    pub fn ntg_get_version(buffer: *mut *mut c_char) -> i32;
    pub fn ntg_cpu_usage(ptr: usize, buffer: *mut f64, future: NtgAsyncStruct) -> i32;
    pub fn ntg_enable_g_lib_loop(enable: bool) -> i32;

    pub fn ntg_on_stream_end(ptr: usize, cb: NtgStreamCallback, user_data: *mut c_void) -> i32;
    pub fn ntg_on_upgrade(ptr: usize, cb: NtgUpgradeCallback, user_data: *mut c_void) -> i32;
    pub fn ntg_on_connection_change(
        ptr: usize,
        cb: NtgConnectionCallback,
        user_data: *mut c_void,
    ) -> i32;
    pub fn ntg_on_signaling_data(
        ptr: usize,
        cb: NtgSignalingCallback,
        user_data: *mut c_void,
    ) -> i32;
    pub fn ntg_on_frames(ptr: usize, cb: NtgFrameCallback, user_data: *mut c_void) -> i32;
    pub fn ntg_on_remote_source_change(
        ptr: usize,
        cb: NtgRemoteSourceCallback,
        user_data: *mut c_void,
    ) -> i32;
    pub fn ntg_on_request_broadcast_timestamp(
        ptr: usize,
        cb: NtgBroadcastTimestampCallback,
        user_data: *mut c_void,
    ) -> i32;
    pub fn ntg_on_request_broadcast_part(
        ptr: usize,
        cb: NtgBroadcastPartCallback,
        user_data: *mut c_void,
    ) -> i32;
    pub fn ntg_register_logger(callback: NtgLogCallback);
    pub fn free(ptr: *mut c_void);
}

// ── Shared Helper Memory Free ───────────────────────────────────────────────

unsafe fn get_async_error_message(code: i32, msg_ptr: *mut c_char) -> String {
    if !msg_ptr.is_null() {
        let c_str = CStr::from_ptr(msg_ptr);
        let msg = c_str.to_string_lossy().into_owned();
        free(msg_ptr as *mut c_void);
        msg
    } else {
        format!("NTgCalls async error: {}", code)
    }
}

unsafe fn parse_string_vector(data: *mut *mut c_char, size: i32) -> Vec<String> {
    let mut result = Vec::new();
    if !data.is_null() {
        for i in 0..size {
            let ptr = *data.offset(i as isize);
            if !ptr.is_null() {
                let c_str = CStr::from_ptr(ptr);
                result.push(c_str.to_string_lossy().into_owned());
                free(ptr as *mut c_void);
            }
        }
        free(data as *mut c_void);
    }
    result
}

unsafe fn parse_device_info_vector(
    devices: *mut ntg_device_info_struct,
    size: i32,
) -> Vec<DeviceInfo> {
    let mut result = Vec::new();
    if !devices.is_null() {
        for i in 0..size {
            let device = *devices.offset(i as isize);
            let name = if !device.name.is_null() {
                let s = CStr::from_ptr(device.name).to_string_lossy().into_owned();
                free(device.name as *mut c_void);
                s
            } else {
                String::new()
            };
            let id = if !device.metadata.is_null() {
                let s = CStr::from_ptr(device.metadata)
                    .to_string_lossy()
                    .into_owned();
                free(device.metadata as *mut c_void);
                s
            } else {
                String::new()
            };
            result.push(DeviceInfo { id, name });
        }
        free(devices as *mut c_void);
    }
    result
}

// ── NAPI Objects ─────────────────────────────────────────────────────────────

#[napi(object)]
pub struct MediaState {
    pub muted: bool,
    pub video_paused: bool,
    pub video_stopped: bool,
    pub presentation_paused: bool,
}

#[napi(object)]
pub struct Frame {
    pub ssrc: i64,
    pub data: Buffer,
    pub frame_data: FrameData,
}

#[napi(object)]
pub struct FrameData {
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
    pub segment_id: i64,
    pub part_id: i32,
    pub limit: i32,
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
    pub key_fingerprint: i64,
}

#[napi(object)]
pub struct RtcServer {
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

// ── Global Synchronous / Utility Functions ───────────────────────────────────

#[napi(js_name = "get_protocol")]
pub fn get_protocol() -> Result<Protocol> {
    let mut buffer = unsafe { std::mem::zeroed::<ntg_protocol_struct>() };
    let rc = unsafe { ntg_get_protocol(&mut buffer) };
    if rc != 0 {
        return Err(Error::from_reason(format!(
            "ntg_get_protocol returned {}",
            rc
        )));
    }
    let versions =
        unsafe { parse_string_vector(buffer.library_versions, buffer.library_versions_size) };
    Ok(Protocol {
        min_layer: buffer.min_layer,
        max_layer: buffer.max_layer,
        udp_p2p: buffer.udp_p2p,
        udp_reflector: buffer.udp_reflector,
        library_versions: versions,
    })
}

#[napi(js_name = "get_media_devices")]
pub fn get_media_devices() -> Result<MediaDevices> {
    let mut buffer = unsafe { std::mem::zeroed::<ntg_media_devices_struct>() };
    let rc = unsafe { ntg_get_media_devices(&mut buffer) };
    if rc != 0 {
        return Err(Error::from_reason(format!(
            "ntg_get_media_devices returned {}",
            rc
        )));
    }
    let mic = unsafe { parse_device_info_vector(buffer.microphone, buffer.size_microphone) };
    let spk = unsafe { parse_device_info_vector(buffer.speaker, buffer.size_speaker) };
    let cam = unsafe { parse_device_info_vector(buffer.camera, buffer.size_camera) };
    let scr = unsafe { parse_device_info_vector(buffer.screen, buffer.size_screen) };
    Ok(MediaDevices {
        microphone: mic,
        speaker: spk,
        camera: cam,
        screen: scr,
    })
}

#[napi(js_name = "enable_g_lib_loop")]
pub fn enable_g_lib_loop(enable: bool) {
    unsafe {
        ntg_enable_g_lib_loop(enable);
    }
}

#[napi(js_name = "get_version")]
pub fn get_version() -> Result<String> {
    let mut buffer: *mut c_char = std::ptr::null_mut();
    let rc = unsafe { ntg_get_version(&mut buffer) };
    if rc != 0 || buffer.is_null() {
        return Err(Error::from_reason("Failed to retrieve NTgCalls version"));
    }
    let version = unsafe {
        let c_str = CStr::from_ptr(buffer);
        let s = c_str.to_string_lossy().into_owned();
        free(buffer as *mut c_void);
        s
    };
    Ok(version)
}

static LOGGER_CALLBACK: OnceLock<Mutex<Option<ThreadsafeFunction<LogMessage, ()>>>> =
    OnceLock::new();

unsafe extern "C" fn raw_log_callback(msg: ntg_log_message_struct) {
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let file = if !msg.file.is_null() {
            CStr::from_ptr(msg.file).to_string_lossy().into_owned()
        } else {
            String::new()
        };
        let message = if !msg.message.is_null() {
            CStr::from_ptr(msg.message).to_string_lossy().into_owned()
        } else {
            String::new()
        };
        if let Some(mutex) = LOGGER_CALLBACK.get() {
            if let Ok(guard) = mutex.lock() {
                if let Some(tsfn) = guard.as_ref() {
                    tsfn.call(
                        Ok(LogMessage {
                            level: msg.level,
                            source: msg.source,
                            file,
                            line: msg.line,
                            message,
                        }),
                        ThreadsafeFunctionCallMode::NonBlocking,
                    );
                }
            }
        }
    }));
}

#[napi(js_name = "register_logger")]
pub fn register_logger(
    #[napi(ts_arg_type = "(message: LogMessage) => void")] cb: Function<LogMessage, ()>,
) -> Result<()> {
    let tsfn = cb
        .build_threadsafe_function()
        .callee_handled::<true>()
        .build()?;

    let mutex = LOGGER_CALLBACK.get_or_init(|| Mutex::new(None));
    let mut guard = mutex
        .lock()
        .map_err(|_| Error::from_reason("Mutex poisoned"))?;
    *guard = Some(tsfn);
    drop(guard);

    unsafe {
        ntg_register_logger(raw_log_callback);
    }

    Ok(())
}

// ── Async Context Definitions ────────────────────────────────────────────────

struct AsyncContext {
    tx: oneshot::Sender<std::result::Result<Option<String>, String>>,
    error_code: i32,
    error_message: *mut c_char,
    result_buffer: *mut c_char,
    _keep_alive: Option<Box<dyn std::any::Any + Send + Sync>>,
}

struct AsyncContextI64 {
    tx: oneshot::Sender<std::result::Result<i64, String>>,
    error_code: i32,
    error_message: *mut c_char,
    result: i64,
}

struct AsyncContextU32 {
    tx: oneshot::Sender<std::result::Result<u32, String>>,
    error_code: i32,
    error_message: *mut c_char,
    result: u32,
    _keep_alive: Option<Box<dyn std::any::Any + Send + Sync>>,
}

struct AsyncContextMediaState {
    tx: oneshot::Sender<std::result::Result<MediaState, String>>,
    error_code: i32,
    error_message: *mut c_char,
    result: ntg_media_state_struct,
}

struct AsyncContextI32 {
    tx: oneshot::Sender<std::result::Result<i32, String>>,
    error_code: i32,
    error_message: *mut c_char,
    result: i32,
}

struct AsyncContextBytes {
    tx: oneshot::Sender<std::result::Result<Vec<u8>, String>>,
    error_code: i32,
    error_message: *mut c_char,
    result_buffer: *mut u8,
    result_size: i32,
    _keep_alive: Option<Box<dyn std::any::Any + Send + Sync>>,
}

struct AsyncContextAuthParams {
    tx: oneshot::Sender<std::result::Result<AuthParams, String>>,
    error_code: i32,
    error_message: *mut c_char,
    result: ntg_auth_params_struct,
    _keep_alive: Option<Box<dyn std::any::Any + Send + Sync>>,
}

struct AsyncContextF64 {
    tx: oneshot::Sender<std::result::Result<f64, String>>,
    error_code: i32,
    error_message: *mut c_char,
    result: f64,
}

struct AsyncContextCalls {
    tx: oneshot::Sender<std::result::Result<HashMap<String, CallInfo>, String>>,
    error_code: i32,
    error_message: *mut c_char,
    result_buffer: *mut ntg_call_info_struct,
    result_size: i32,
}

// ── Raw Async Callbacks ──────────────────────────────────────────────────────

unsafe extern "C" fn rust_async_callback(user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::sync::atomic::fence(Ordering::Acquire);
        let context = Box::from_raw(user_data as *mut AsyncContext);
        if context.error_code == 0 {
            let mut res = None;
            if !context.result_buffer.is_null() {
                let c_str = CStr::from_ptr(context.result_buffer);
                res = Some(c_str.to_string_lossy().into_owned());
                free(context.result_buffer as *mut c_void);
            }
            let _ = context.tx.send(Ok(res));
        } else {
            let err_msg = get_async_error_message(context.error_code, context.error_message);
            let _ = context.tx.send(Err(err_msg));
        }
    }));
}

unsafe extern "C" fn rust_async_callback_i64(user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::sync::atomic::fence(Ordering::Acquire);
        let context = Box::from_raw(user_data as *mut AsyncContextI64);
        if context.error_code == 0 {
            let _ = context.tx.send(Ok(context.result));
        } else {
            let err_msg = get_async_error_message(context.error_code, context.error_message);
            let _ = context.tx.send(Err(err_msg));
        }
    }));
}

unsafe extern "C" fn rust_async_callback_u32(user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::sync::atomic::fence(Ordering::Acquire);
        let context = Box::from_raw(user_data as *mut AsyncContextU32);
        if context.error_code == 0 {
            let _ = context.tx.send(Ok(context.result));
        } else {
            let err_msg = get_async_error_message(context.error_code, context.error_message);
            let _ = context.tx.send(Err(err_msg));
        }
    }));
}

unsafe extern "C" fn rust_async_callback_mediastate(user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::sync::atomic::fence(Ordering::Acquire);
        let context = Box::from_raw(user_data as *mut AsyncContextMediaState);
        if context.error_code == 0 {
            let js_state = MediaState {
                muted: context.result.muted,
                video_paused: context.result.video_paused,
                video_stopped: context.result.video_stopped,
                presentation_paused: context.result.presentation_paused,
            };
            let _ = context.tx.send(Ok(js_state));
        } else {
            let err_msg = get_async_error_message(context.error_code, context.error_message);
            let _ = context.tx.send(Err(err_msg));
        }
    }));
}

unsafe extern "C" fn rust_async_callback_i32(user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::sync::atomic::fence(Ordering::Acquire);
        let context = Box::from_raw(user_data as *mut AsyncContextI32);
        if context.error_code == 0 {
            let _ = context.tx.send(Ok(context.result));
        } else {
            let err_msg = get_async_error_message(context.error_code, context.error_message);
            let _ = context.tx.send(Err(err_msg));
        }
    }));
}

unsafe extern "C" fn rust_async_callback_bytes(user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::sync::atomic::fence(Ordering::Acquire);
        let context = Box::from_raw(user_data as *mut AsyncContextBytes);
        if context.error_code == 0 {
            let mut bytes = Vec::new();
            if !context.result_buffer.is_null() {
                let slice =
                    std::slice::from_raw_parts(context.result_buffer, context.result_size as usize);
                bytes = slice.to_vec();
                free(context.result_buffer as *mut c_void);
            }
            let _ = context.tx.send(Ok(bytes));
        } else {
            let err_msg = get_async_error_message(context.error_code, context.error_message);
            let _ = context.tx.send(Err(err_msg));
        }
    }));
}

unsafe extern "C" fn rust_async_callback_authparams(user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::sync::atomic::fence(Ordering::Acquire);
        let context = Box::from_raw(user_data as *mut AsyncContextAuthParams);
        if context.error_code == 0 {
            let mut bytes = Vec::new();
            if !context.result.g_a_or_b.is_null() {
                let slice = std::slice::from_raw_parts(
                    context.result.g_a_or_b,
                    context.result.size_gab as usize,
                );
                bytes = slice.to_vec();
            }
            let js_params = AuthParams {
                g_a_or_b: Buffer::from(bytes),
                key_fingerprint: context.result.key_fingerprint,
            };
            let _ = context.tx.send(Ok(js_params));
        } else {
            let err_msg = get_async_error_message(context.error_code, context.error_message);
            let _ = context.tx.send(Err(err_msg));
        }
    }));
}

unsafe extern "C" fn rust_async_callback_f64(user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::sync::atomic::fence(Ordering::Acquire);
        let context = Box::from_raw(user_data as *mut AsyncContextF64);
        if context.error_code == 0 {
            let _ = context.tx.send(Ok(context.result));
        } else {
            let err_msg = get_async_error_message(context.error_code, context.error_message);
            let _ = context.tx.send(Err(err_msg));
        }
    }));
}

unsafe extern "C" fn rust_async_callback_calls(user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        std::sync::atomic::fence(Ordering::Acquire);
        let context = Box::from_raw(user_data as *mut AsyncContextCalls);
        if context.error_code == 0 {
            let mut map = HashMap::new();
            if !context.result_buffer.is_null() {
                for i in 0..context.result_size {
                    let call = *context.result_buffer.offset(i as isize);
                    map.insert(
                        call.chat_id.to_string(),
                        CallInfo {
                            capture: call.capture,
                            playback: call.playback,
                        },
                    );
                }
                free(context.result_buffer as *mut c_void);
            }
            let _ = context.tx.send(Ok(map));
        } else {
            let err_msg = get_async_error_message(context.error_code, context.error_message);
            let _ = context.tx.send(Err(err_msg));
        }
    }));
}

// ── Raw Community FFI Callbacks ──────────────────────────────────────────────

unsafe extern "C" fn raw_stream_end_callback(
    _pointer: usize,
    chat_id: i64,
    stream_type: i32,
    stream_device: i32,
    user_data: *mut c_void,
) {
    if user_data.is_null() || chat_id == 0 {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mutex_ptr =
            user_data as *const Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32), ()>>>;
        if let Ok(guard) = (*mutex_ptr).lock() {
            if let Some(tsfn) = guard.as_ref() {
                tsfn.call(
                    Ok((BigInt::from(chat_id), stream_type, stream_device)),
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }
        }
    }));
}

unsafe extern "C" fn raw_upgrade_callback(
    _pointer: usize,
    chat_id: i64,
    state: ntg_media_state_struct,
    user_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mutex_ptr =
            user_data as *const Mutex<Option<ThreadsafeFunction<(BigInt, MediaState), ()>>>;
        if let Ok(guard) = (*mutex_ptr).lock() {
            if let Some(tsfn) = guard.as_ref() {
                let js_state = MediaState {
                    muted: state.muted,
                    video_paused: state.video_paused,
                    video_stopped: state.video_stopped,
                    presentation_paused: state.presentation_paused,
                };
                tsfn.call(
                    Ok((BigInt::from(chat_id), js_state)),
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }
        }
    }));
}

unsafe extern "C" fn raw_connection_callback(
    _pointer: usize,
    chat_id: i64,
    info: ntg_network_info_struct,
    user_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mutex_ptr =
            user_data as *const Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32), ()>>>;
        if let Ok(guard) = (*mutex_ptr).lock() {
            if let Some(tsfn) = guard.as_ref() {
                tsfn.call(
                    Ok((BigInt::from(chat_id), info.kind, info.state)),
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }
        }
    }));
}

unsafe extern "C" fn raw_signaling_callback(
    _pointer: usize,
    chat_id: i64,
    data: *mut u8,
    size: i32,
    user_data: *mut c_void,
) {
    if user_data.is_null() || data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mutex_ptr = user_data as *const Mutex<Option<ThreadsafeFunction<(BigInt, Buffer), ()>>>;
        if let Ok(guard) = (*mutex_ptr).lock() {
            if let Some(tsfn) = guard.as_ref() {
                let slice = std::slice::from_raw_parts(data, size as usize);
                let buf = Buffer::from(slice.to_vec());
                tsfn.call(
                    Ok((BigInt::from(chat_id), buf)),
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }
        }
    }));
}

unsafe extern "C" fn raw_frames_callback(
    _pointer: usize,
    chat_id: i64,
    mode: i32,
    device: i32,
    frames: *mut ntg_frame_struct,
    size: u64,
    user_data: *mut c_void,
) {
    if user_data.is_null() || frames.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mutex_ptr = user_data
            as *const Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32, Vec<Frame>), ()>>>;
        if let Ok(guard) = (*mutex_ptr).lock() {
            if let Some(tsfn) = guard.as_ref() {
                let mut js_frames = Vec::new();
                for i in 0..size {
                    let frame = *frames.offset(i as isize);
                    let data_slice =
                        std::slice::from_raw_parts(frame.data, frame.size_data as usize);
                    let buf = Buffer::from(data_slice.to_vec());
                    js_frames.push(Frame {
                        ssrc: frame.ssrc,
                        data: buf,
                        frame_data: FrameData {
                            absolute_capture_timestamp_ms: frame
                                .frame_data
                                .absolute_capture_timestamp_ms,
                            width: frame.frame_data.width,
                            height: frame.frame_data.height,
                            rotation: frame.frame_data.rotation,
                        },
                    });
                }
                tsfn.call(
                    Ok((BigInt::from(chat_id), mode, device, js_frames)),
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }
        }
    }));
}

unsafe extern "C" fn raw_remote_source_callback(
    _pointer: usize,
    chat_id: i64,
    source: ntg_remote_source_struct,
    user_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mutex_ptr =
            user_data as *const Mutex<Option<ThreadsafeFunction<(BigInt, RemoteSource), ()>>>;
        if let Ok(guard) = (*mutex_ptr).lock() {
            if let Some(tsfn) = guard.as_ref() {
                let js_source = RemoteSource {
                    ssrc: source.ssrc,
                    state: source.state,
                    device: source.device,
                };
                tsfn.call(
                    Ok((BigInt::from(chat_id), js_source)),
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }
        }
    }));
}

unsafe extern "C" fn raw_broadcast_timestamp_callback(
    _pointer: usize,
    chat_id: i64,
    user_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mutex_ptr = user_data as *const Mutex<Option<ThreadsafeFunction<BigInt, ()>>>;
        if let Ok(guard) = (*mutex_ptr).lock() {
            if let Some(tsfn) = guard.as_ref() {
                tsfn.call(
                    Ok(BigInt::from(chat_id)),
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }
        }
    }));
}

unsafe extern "C" fn raw_broadcast_part_callback(
    _pointer: usize,
    chat_id: i64,
    request: ntg_segment_part_request_struct,
    user_data: *mut c_void,
) {
    if user_data.is_null() {
        return;
    }
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mutex_ptr =
            user_data as *const Mutex<Option<ThreadsafeFunction<(BigInt, SegmentPartRequest), ()>>>;
        if let Ok(guard) = (*mutex_ptr).lock() {
            if let Some(tsfn) = guard.as_ref() {
                let js_request = SegmentPartRequest {
                    segment_id: request.segment_id,
                    part_id: request.part_id,
                    limit: request.limit,
                    timestamp: request.timestamp,
                    quality_update: request.quality_update,
                    channel_id: request.channel_id,
                    quality: request.quality,
                };
                tsfn.call(
                    Ok((BigInt::from(chat_id), js_request)),
                    ThreadsafeFunctionCallMode::NonBlocking,
                );
            }
        }
    }));
}

// ── Keep Alive Structures for Async FFI Calls ────────────────────────────────

#[allow(dead_code)]
struct MediaDescriptionKeeper {
    microphone_input: Option<CString>,
    speaker_input: Option<CString>,
    camera_input: Option<CString>,
    screen_input: Option<CString>,
    microphone: Option<Box<ntg_audio_description_struct>>,
    speaker: Option<Box<ntg_audio_description_struct>>,
    camera: Option<Box<ntg_video_description_struct>>,
    screen: Option<Box<ntg_video_description_struct>>,
}

unsafe impl Send for MediaDescriptionKeeper {}
unsafe impl Sync for MediaDescriptionKeeper {}

#[allow(dead_code)]
struct ConnectP2PKeeper {
    servers_ipv4: Vec<CString>,
    servers_ipv6: Vec<CString>,
    servers_username: Vec<CString>,
    servers_password: Vec<CString>,
    servers_peer_tag: Vec<Option<Vec<u8>>>,
    servers: Vec<ntg_rtc_server_struct>,
    versions_strings: Vec<CString>,
    versions: Vec<*mut c_char>,
}

unsafe impl Send for ConnectP2PKeeper {}
unsafe impl Sync for ConnectP2PKeeper {}

#[allow(dead_code)]
struct AddIncomingVideoKeeper {
    endpoint: CString,
    semantics_strings: Vec<CString>,
    ssrcs_vecs: Vec<Vec<u32>>,
    ssrc_groups: Vec<ntg_ssrc_group_struct>,
}

unsafe impl Send for AddIncomingVideoKeeper {}
unsafe impl Sync for AddIncomingVideoKeeper {}

#[allow(dead_code)]
struct InitExchangeKeeper {
    p: Vec<u8>,
    random: Vec<u8>,
    dh_config: Box<ntg_dh_config_struct>,
    g_a_hash: Vec<u8>,
}

unsafe impl Send for InitExchangeKeeper {}
unsafe impl Sync for InitExchangeKeeper {}

// ── NtgCalls JS Class ────────────────────────────────────────────────────────

#[napi]
#[allow(clippy::type_complexity)]
pub struct NtgCalls {
    handle: usize,
    stream_end_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32), ()>>>>,
    upgrade_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, MediaState), ()>>>>,
    connection_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32), ()>>>>,
    signaling_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, Buffer), ()>>>>,
    frames_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32, Vec<Frame>), ()>>>>,
    remote_source_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, RemoteSource), ()>>>>,
    broadcast_timestamp_cb: Arc<Mutex<Option<ThreadsafeFunction<BigInt, ()>>>>,
    broadcast_part_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, SegmentPartRequest), ()>>>>,

    stream_end_cb_ptr: AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32), ()>>>>,
    upgrade_cb_ptr: AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, MediaState), ()>>>>,
    connection_cb_ptr: AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32), ()>>>>,
    signaling_cb_ptr: AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, Buffer), ()>>>>,
    frames_cb_ptr: AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32, Vec<Frame>), ()>>>>,
    remote_source_cb_ptr: AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, RemoteSource), ()>>>>,
    broadcast_timestamp_cb_ptr: AtomicPtr<Mutex<Option<ThreadsafeFunction<BigInt, ()>>>>,
    broadcast_part_cb_ptr:
        AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, SegmentPartRequest), ()>>>>,

    pending_ops: Arc<AtomicUsize>,
}

#[napi]
impl NtgCalls {
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        let handle = unsafe { ntg_init() };
        if handle == 0 {
            return Err(Error::from_reason("Failed to initialize NTgCalls handle"));
        }

        Ok(Self {
            handle,
            stream_end_cb: Arc::new(Mutex::new(None)),
            upgrade_cb: Arc::new(Mutex::new(None)),
            connection_cb: Arc::new(Mutex::new(None)),
            signaling_cb: Arc::new(Mutex::new(None)),
            frames_cb: Arc::new(Mutex::new(None)),
            remote_source_cb: Arc::new(Mutex::new(None)),
            broadcast_timestamp_cb: Arc::new(Mutex::new(None)),
            broadcast_part_cb: Arc::new(Mutex::new(None)),

            stream_end_cb_ptr: AtomicPtr::new(std::ptr::null_mut()),
            upgrade_cb_ptr: AtomicPtr::new(std::ptr::null_mut()),
            connection_cb_ptr: AtomicPtr::new(std::ptr::null_mut()),
            signaling_cb_ptr: AtomicPtr::new(std::ptr::null_mut()),
            frames_cb_ptr: AtomicPtr::new(std::ptr::null_mut()),
            remote_source_cb_ptr: AtomicPtr::new(std::ptr::null_mut()),
            broadcast_timestamp_cb_ptr: AtomicPtr::new(std::ptr::null_mut()),
            broadcast_part_cb_ptr: AtomicPtr::new(std::ptr::null_mut()),

            pending_ops: Arc::new(AtomicUsize::new(0)),
        })
    }

    #[napi(js_name = "on_stream_end")]
    pub fn on_stream_end(
        &self,
        #[napi(ts_arg_type = "(chatId: bigint, streamType: number, streamDevice: number) => void")]
        cb: Function<(BigInt, i32, i32), ()>,
    ) -> Result<()> {
        let tsfn = cb
            .build_threadsafe_function()
            .callee_handled::<true>()
            .build()?;

        let mut guard = self
            .stream_end_cb
            .lock()
            .map_err(|_| Error::from_reason("Mutex poisoned"))?;
        *guard = Some(tsfn);
        drop(guard);

        if self.stream_end_cb_ptr.load(Ordering::Acquire).is_null() {
            let leaked = Arc::into_raw(Arc::clone(&self.stream_end_cb)) as *mut _;
            if self
                .stream_end_cb_ptr
                .compare_exchange(
                    std::ptr::null_mut(),
                    leaked,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                unsafe {
                    ntg_on_stream_end(self.handle, raw_stream_end_callback, leaked as *mut c_void);
                }
            } else {
                unsafe {
                    drop(Arc::from_raw(leaked));
                }
            }
        }

        Ok(())
    }

    #[napi(js_name = "on_upgrade")]
    pub fn on_upgrade(
        &self,
        #[napi(ts_arg_type = "(chatId: bigint, state: MediaState) => void")] cb: Function<
            (BigInt, MediaState),
            (),
        >,
    ) -> Result<()> {
        let tsfn = cb
            .build_threadsafe_function()
            .callee_handled::<true>()
            .build()?;

        let mut guard = self
            .upgrade_cb
            .lock()
            .map_err(|_| Error::from_reason("Mutex poisoned"))?;
        *guard = Some(tsfn);
        drop(guard);

        if self.upgrade_cb_ptr.load(Ordering::Acquire).is_null() {
            let leaked = Arc::into_raw(Arc::clone(&self.upgrade_cb)) as *mut _;
            if self
                .upgrade_cb_ptr
                .compare_exchange(
                    std::ptr::null_mut(),
                    leaked,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                unsafe {
                    ntg_on_upgrade(self.handle, raw_upgrade_callback, leaked as *mut c_void);
                }
            } else {
                unsafe {
                    drop(Arc::from_raw(leaked));
                }
            }
        }

        Ok(())
    }

    #[napi(js_name = "on_connection_change")]
    pub fn on_connection_change(
        &self,
        #[napi(ts_arg_type = "(chatId: bigint, kind: number, state: number) => void")] cb: Function<
            (BigInt, i32, i32),
            (),
        >,
    ) -> Result<()> {
        let tsfn = cb
            .build_threadsafe_function()
            .callee_handled::<true>()
            .build()?;

        let mut guard = self
            .connection_cb
            .lock()
            .map_err(|_| Error::from_reason("Mutex poisoned"))?;
        *guard = Some(tsfn);
        drop(guard);

        if self.connection_cb_ptr.load(Ordering::Acquire).is_null() {
            let leaked = Arc::into_raw(Arc::clone(&self.connection_cb)) as *mut _;
            if self
                .connection_cb_ptr
                .compare_exchange(
                    std::ptr::null_mut(),
                    leaked,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                unsafe {
                    ntg_on_connection_change(
                        self.handle,
                        raw_connection_callback,
                        leaked as *mut c_void,
                    );
                }
            } else {
                unsafe {
                    drop(Arc::from_raw(leaked));
                }
            }
        }

        Ok(())
    }

    #[napi(js_name = "on_signaling_data")]
    pub fn on_signaling_data(
        &self,
        #[napi(ts_arg_type = "(chatId: bigint, data: Buffer) => void")] cb: Function<
            (BigInt, Buffer),
            (),
        >,
    ) -> Result<()> {
        let tsfn = cb
            .build_threadsafe_function()
            .callee_handled::<true>()
            .build()?;

        let mut guard = self
            .signaling_cb
            .lock()
            .map_err(|_| Error::from_reason("Mutex poisoned"))?;
        *guard = Some(tsfn);
        drop(guard);

        if self.signaling_cb_ptr.load(Ordering::Acquire).is_null() {
            let leaked = Arc::into_raw(Arc::clone(&self.signaling_cb)) as *mut _;
            if self
                .signaling_cb_ptr
                .compare_exchange(
                    std::ptr::null_mut(),
                    leaked,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                unsafe {
                    ntg_on_signaling_data(
                        self.handle,
                        raw_signaling_callback,
                        leaked as *mut c_void,
                    );
                }
            } else {
                unsafe {
                    drop(Arc::from_raw(leaked));
                }
            }
        }

        Ok(())
    }

    #[napi(js_name = "on_frames")]
    pub fn on_frames(
        &self,
        #[napi(
            ts_arg_type = "(chatId: bigint, mode: number, device: number, frames: Frame[]) => void"
        )]
        cb: Function<(BigInt, i32, i32, Vec<Frame>), ()>,
    ) -> Result<()> {
        let tsfn = cb
            .build_threadsafe_function()
            .callee_handled::<true>()
            .build()?;

        let mut guard = self
            .frames_cb
            .lock()
            .map_err(|_| Error::from_reason("Mutex poisoned"))?;
        *guard = Some(tsfn);
        drop(guard);

        if self.frames_cb_ptr.load(Ordering::Acquire).is_null() {
            let leaked = Arc::into_raw(Arc::clone(&self.frames_cb)) as *mut _;
            if self
                .frames_cb_ptr
                .compare_exchange(
                    std::ptr::null_mut(),
                    leaked,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                unsafe {
                    ntg_on_frames(self.handle, raw_frames_callback, leaked as *mut c_void);
                }
            } else {
                unsafe {
                    drop(Arc::from_raw(leaked));
                }
            }
        }

        Ok(())
    }

    #[napi(js_name = "on_remote_source_change")]
    pub fn on_remote_source_change(
        &self,
        #[napi(ts_arg_type = "(chatId: bigint, source: RemoteSource) => void")] cb: Function<
            (BigInt, RemoteSource),
            (),
        >,
    ) -> Result<()> {
        let tsfn = cb
            .build_threadsafe_function()
            .callee_handled::<true>()
            .build()?;

        let mut guard = self
            .remote_source_cb
            .lock()
            .map_err(|_| Error::from_reason("Mutex poisoned"))?;
        *guard = Some(tsfn);
        drop(guard);

        if self.remote_source_cb_ptr.load(Ordering::Acquire).is_null() {
            let leaked = Arc::into_raw(Arc::clone(&self.remote_source_cb)) as *mut _;
            if self
                .remote_source_cb_ptr
                .compare_exchange(
                    std::ptr::null_mut(),
                    leaked,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                unsafe {
                    ntg_on_remote_source_change(
                        self.handle,
                        raw_remote_source_callback,
                        leaked as *mut c_void,
                    );
                }
            } else {
                unsafe {
                    drop(Arc::from_raw(leaked));
                }
            }
        }

        Ok(())
    }

    #[napi(js_name = "on_request_broadcast_timestamp")]
    pub fn on_request_broadcast_timestamp(
        &self,
        #[napi(ts_arg_type = "(chatId: bigint) => void")] cb: Function<BigInt, ()>,
    ) -> Result<()> {
        let tsfn = cb
            .build_threadsafe_function()
            .callee_handled::<true>()
            .build()?;

        let mut guard = self
            .broadcast_timestamp_cb
            .lock()
            .map_err(|_| Error::from_reason("Mutex poisoned"))?;
        *guard = Some(tsfn);
        drop(guard);

        if self
            .broadcast_timestamp_cb_ptr
            .load(Ordering::Acquire)
            .is_null()
        {
            let leaked = Arc::into_raw(Arc::clone(&self.broadcast_timestamp_cb)) as *mut _;
            if self
                .broadcast_timestamp_cb_ptr
                .compare_exchange(
                    std::ptr::null_mut(),
                    leaked,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                unsafe {
                    ntg_on_request_broadcast_timestamp(
                        self.handle,
                        raw_broadcast_timestamp_callback,
                        leaked as *mut c_void,
                    );
                }
            } else {
                unsafe {
                    drop(Arc::from_raw(leaked));
                }
            }
        }

        Ok(())
    }

    #[napi(js_name = "on_request_broadcast_part")]
    pub fn on_request_broadcast_part(
        &self,
        #[napi(ts_arg_type = "(chatId: bigint, request: SegmentPartRequest) => void")] cb: Function<
            (BigInt, SegmentPartRequest),
            (),
        >,
    ) -> Result<()> {
        let tsfn = cb
            .build_threadsafe_function()
            .callee_handled::<true>()
            .build()?;

        let mut guard = self
            .broadcast_part_cb
            .lock()
            .map_err(|_| Error::from_reason("Mutex poisoned"))?;
        *guard = Some(tsfn);
        drop(guard);

        if self.broadcast_part_cb_ptr.load(Ordering::Acquire).is_null() {
            let leaked = Arc::into_raw(Arc::clone(&self.broadcast_part_cb)) as *mut _;
            if self
                .broadcast_part_cb_ptr
                .compare_exchange(
                    std::ptr::null_mut(),
                    leaked,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                unsafe {
                    ntg_on_request_broadcast_part(
                        self.handle,
                        raw_broadcast_part_callback,
                        leaked as *mut c_void,
                    );
                }
            } else {
                unsafe {
                    drop(Arc::from_raw(leaked));
                }
            }
        }

        Ok(())
    }

    // ── Public Async API ───────────────────────────────────────────────────────

    #[napi(js_name = "create")]
    pub async fn create(&self, chat_id: i64) -> Result<String> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: None,
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let rc = ntg_create(
                handle,
                chat_id,
                std::ptr::addr_of_mut!((*ctx).result_buffer),
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_create returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(Some(offer)) => Ok(offer),
            Ok(None) => Err(Error::from_reason("ntg_create did not return an offer SDP")),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "connect")]
    pub async fn connect(&self, chat_id: i64, params: String, is_presentation: bool) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let c_params = CString::new(params)
            .map_err(|_| Error::from_reason("Invalid connection params string"))?;

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: Some(Box::new(c_params)),
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let keeper = (*ctx)
                ._keep_alive
                .as_ref()
                .unwrap()
                .downcast_ref::<CString>()
                .unwrap();
            let rc = ntg_connect(handle, chat_id, keeper.as_ptr(), is_presentation, ntg_async);
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_connect returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "init_presentation")]
    pub async fn init_presentation(&self, chat_id: i64) -> Result<String> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: None,
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let rc = ntg_init_presentation(
                handle,
                chat_id,
                std::ptr::addr_of_mut!((*ctx).result_buffer),
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_init_presentation returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(Some(offer)) => Ok(offer),
            Ok(None) => Err(Error::from_reason(
                "ntg_init_presentation did not return an offer SDP",
            )),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "stop_presentation")]
    pub async fn stop_presentation(&self, chat_id: i64) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: None,
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let rc = ntg_stop_presentation(handle, chat_id, ntg_async);
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_stop_presentation returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "set_stream_sources")]
    pub async fn set_stream_sources(
        &self,
        chat_id: i64,
        stream_mode: i32,
        desc: MediaDescription,
    ) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let context = {
            let mut keeper = MediaDescriptionKeeper {
                microphone_input: None,
                speaker_input: None,
                camera_input: None,
                screen_input: None,
                microphone: None,
                speaker: None,
                camera: None,
                screen: None,
            };

            if let Some(ref m) = desc.microphone {
                let input = CString::new(m.input.clone())
                    .map_err(|_| Error::from_reason("Invalid microphone input string"))?;
                let audio_struct = Box::new(ntg_audio_description_struct {
                    media_source: m.media_source,
                    input: input.as_ptr(),
                    sample_rate: m.sample_rate,
                    channel_count: m.channel_count,
                    keep_open: m.keep_open,
                });
                keeper.microphone_input = Some(input);
                keeper.microphone = Some(audio_struct);
            }

            if let Some(ref s) = desc.speaker {
                let input = CString::new(s.input.clone())
                    .map_err(|_| Error::from_reason("Invalid speaker input string"))?;
                let audio_struct = Box::new(ntg_audio_description_struct {
                    media_source: s.media_source,
                    input: input.as_ptr(),
                    sample_rate: s.sample_rate,
                    channel_count: s.channel_count,
                    keep_open: s.keep_open,
                });
                keeper.speaker_input = Some(input);
                keeper.speaker = Some(audio_struct);
            }

            if let Some(ref c) = desc.camera {
                let input = CString::new(c.input.clone())
                    .map_err(|_| Error::from_reason("Invalid camera input string"))?;
                let video_struct = Box::new(ntg_video_description_struct {
                    media_source: c.media_source,
                    input: input.as_ptr(),
                    width: c.width,
                    height: c.height,
                    fps: c.fps,
                    keep_open: c.keep_open,
                });
                keeper.camera_input = Some(input);
                keeper.camera = Some(video_struct);
            }

            if let Some(ref s) = desc.screen {
                let input = CString::new(s.input.clone())
                    .map_err(|_| Error::from_reason("Invalid screen input string"))?;
                let video_struct = Box::new(ntg_video_description_struct {
                    media_source: s.media_source,
                    input: input.as_ptr(),
                    width: s.width,
                    height: s.height,
                    fps: s.fps,
                    keep_open: s.keep_open,
                });
                keeper.screen_input = Some(input);
                keeper.screen = Some(video_struct);
            }

            Box::into_raw(Box::new(AsyncContext {
                tx,
                error_code: 0,
                error_message: std::ptr::null_mut(),
                result_buffer: std::ptr::null_mut(),
                _keep_alive: Some(Box::new(keeper)),
            }))
        };

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let keeper_ref = (*ctx)
                ._keep_alive
                .as_mut()
                .unwrap()
                .downcast_mut::<MediaDescriptionKeeper>()
                .unwrap();

            let desc_struct = ntg_media_description_struct {
                microphone: keeper_ref
                    .microphone
                    .as_mut()
                    .map_or(std::ptr::null_mut(), |b| &mut **b as *mut _),
                speaker: keeper_ref
                    .speaker
                    .as_mut()
                    .map_or(std::ptr::null_mut(), |b| &mut **b as *mut _),
                camera: keeper_ref
                    .camera
                    .as_mut()
                    .map_or(std::ptr::null_mut(), |b| &mut **b as *mut _),
                screen: keeper_ref
                    .screen
                    .as_mut()
                    .map_or(std::ptr::null_mut(), |b| &mut **b as *mut _),
            };

            let rc = ntg_set_stream_sources(handle, chat_id, stream_mode, desc_struct, ntg_async);
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_set_stream_sources returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "set_audio_source")]
    pub async fn set_audio_source(&self, chat_id: i64, ffmpeg_cmd: String) -> Result<()> {
        let desc = MediaDescription {
            microphone: Some(AudioDescription {
                media_source: 2, // SHELL
                input: ffmpeg_cmd,
                sample_rate: 48000,
                channel_count: 1,
                keep_open: false,
            }),
            speaker: None,
            camera: None,
            screen: None,
        };
        self.set_stream_sources(chat_id, 0, desc).await
    }

    #[napi(js_name = "pause")]
    pub async fn pause(&self, chat_id: i64) -> Result<()> {
        self.run_simple_async_op(chat_id, |h, cid, a| unsafe { ntg_pause(h, cid, a) })
            .await
    }

    #[napi(js_name = "resume")]
    pub async fn resume(&self, chat_id: i64) -> Result<()> {
        self.run_simple_async_op(chat_id, |h, cid, a| unsafe { ntg_resume(h, cid, a) })
            .await
    }

    #[napi(js_name = "mute")]
    pub async fn mute(&self, chat_id: i64) -> Result<()> {
        self.run_simple_async_op(chat_id, |h, cid, a| unsafe { ntg_mute(h, cid, a) })
            .await
    }

    #[napi(js_name = "unmute")]
    pub async fn unmute(&self, chat_id: i64) -> Result<()> {
        self.run_simple_async_op(chat_id, |h, cid, a| unsafe { ntg_unmute(h, cid, a) })
            .await
    }

    #[napi(js_name = "stop")]
    pub async fn stop(&self, chat_id: i64) -> Result<()> {
        self.run_simple_async_op(chat_id, |h, cid, a| unsafe { ntg_stop(h, cid, a) })
            .await
    }

    #[napi(js_name = "time")]
    pub async fn time(&self, chat_id: i64, mode: Option<i32>) -> Result<i64> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let stream_mode = mode.unwrap_or(0);
        let (tx, rx) = oneshot::channel::<std::result::Result<i64, String>>();

        let context = Box::into_raw(Box::new(AsyncContextI64 {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result: 0,
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback_i64),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContextI64;
            let rc = ntg_time(
                handle,
                chat_id,
                stream_mode,
                std::ptr::addr_of_mut!((*ctx).result),
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_time returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        result.map_err(Error::from_reason)
    }

    #[napi(js_name = "get_state")]
    pub async fn get_state(&self, chat_id: i64) -> Result<MediaState> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<MediaState, String>>();

        let context = Box::into_raw(Box::new(AsyncContextMediaState {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result: unsafe { std::mem::zeroed() },
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback_mediastate),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContextMediaState;
            let rc = ntg_get_state(
                handle,
                chat_id,
                std::ptr::addr_of_mut!((*ctx).result),
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_get_state returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        result.map_err(Error::from_reason)
    }

    #[napi(js_name = "get_connection_mode")]
    pub async fn get_connection_mode(&self, chat_id: i64) -> Result<i32> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<i32, String>>();

        let context = Box::into_raw(Box::new(AsyncContextI32 {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result: 0,
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback_i32),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContextI32;
            let rc = ntg_get_connection_mode(
                handle,
                chat_id,
                std::ptr::addr_of_mut!((*ctx).result),
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_get_connection_mode returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        result.map_err(Error::from_reason)
    }

    #[napi(js_name = "add_incoming_video")]
    pub async fn add_incoming_video(
        &self,
        chat_id: i64,
        endpoint: String,
        ssrc_groups_list: Vec<SsrcGroup>,
    ) -> Result<u32> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<u32, String>>();

        let context = {
            let mut semantics_strings = Vec::new();
            let mut ssrcs_vecs = Vec::new();
            let mut groups = Vec::new();

            let endpoint_c = CString::new(endpoint)
                .map_err(|_| Error::from_reason("Invalid endpoint string"))?;

            for g in &ssrc_groups_list {
                let semantics_c = CString::new(g.semantics.clone())
                    .map_err(|_| Error::from_reason("Invalid semantics string"))?;
                semantics_strings.push(semantics_c);
                ssrcs_vecs.push(g.ssrcs.clone());
            }

            for i in 0..semantics_strings.len() {
                groups.push(ntg_ssrc_group_struct {
                    semantics: semantics_strings[i].as_ptr() as *mut _,
                    ssrcs: ssrcs_vecs[i].as_mut_ptr(),
                    size_ssrcs: ssrcs_vecs[i].len() as i32,
                });
            }

            let keeper = AddIncomingVideoKeeper {
                endpoint: endpoint_c,
                semantics_strings,
                ssrcs_vecs,
                ssrc_groups: groups,
            };

            Box::into_raw(Box::new(AsyncContextU32 {
                tx,
                error_code: 0,
                error_message: std::ptr::null_mut(),
                result: 0,
                _keep_alive: Some(Box::new(keeper)),
            }))
        };

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback_u32),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContextU32;
            let keeper_ref = (*ctx)
                ._keep_alive
                .as_mut()
                .unwrap()
                .downcast_mut::<AddIncomingVideoKeeper>()
                .unwrap();

            let rc = ntg_add_incoming_video(
                handle,
                chat_id,
                keeper_ref.endpoint.as_ptr(),
                keeper_ref.ssrc_groups.as_mut_ptr(),
                keeper_ref.ssrc_groups.len() as i32,
                std::ptr::addr_of_mut!((*ctx).result),
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_add_incoming_video returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        result.map_err(Error::from_reason)
    }

    #[napi(js_name = "remove_incoming_video")]
    pub async fn remove_incoming_video(&self, chat_id: i64, endpoint: String) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let endpoint_c =
            CString::new(endpoint).map_err(|_| Error::from_reason("Invalid endpoint string"))?;

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: Some(Box::new(endpoint_c)),
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let keeper = (*ctx)
                ._keep_alive
                .as_ref()
                .unwrap()
                .downcast_ref::<CString>()
                .unwrap();

            let rc = ntg_remove_incoming_video(handle, chat_id, keeper.as_ptr(), ntg_async);
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_remove_incoming_video returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "create_p2p")]
    pub async fn create_p2p(&self, user_id: i64) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: None,
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let rc = ntg_create_p2p(handle, user_id, ntg_async);
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_create_p2p returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "init_exchange")]
    pub async fn init_exchange(
        &self,
        user_id: i64,
        dh_config: DhConfig,
        g_a_hash: Buffer,
    ) -> Result<Buffer> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Vec<u8>, String>>();

        let context = {
            let p_vec = dh_config.p.to_vec();
            let random_vec = dh_config.random.to_vec();
            let dh_struct = Box::new(ntg_dh_config_struct {
                g: dh_config.g,
                p: p_vec.as_ptr(),
                size_p: p_vec.len() as i32,
                random: random_vec.as_ptr(),
                size_random: random_vec.len() as i32,
            });
            let g_a_hash_vec = g_a_hash.to_vec();

            let keeper = InitExchangeKeeper {
                p: p_vec,
                random: random_vec,
                dh_config: dh_struct,
                g_a_hash: g_a_hash_vec,
            };

            Box::into_raw(Box::new(AsyncContextBytes {
                tx,
                error_code: 0,
                error_message: std::ptr::null_mut(),
                result_buffer: std::ptr::null_mut(),
                result_size: 0,
                _keep_alive: Some(Box::new(keeper)),
            }))
        };

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback_bytes),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContextBytes;
            let keeper_ref = (*ctx)
                ._keep_alive
                .as_mut()
                .unwrap()
                .downcast_mut::<InitExchangeKeeper>()
                .unwrap();

            let rc = ntg_init_exchange(
                handle,
                user_id,
                &mut *keeper_ref.dh_config as *mut _,
                keeper_ref.g_a_hash.as_ptr(),
                keeper_ref.g_a_hash.len() as i32,
                std::ptr::addr_of_mut!((*ctx).result_buffer),
                std::ptr::addr_of_mut!((*ctx).result_size),
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_init_exchange returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        result.map(Buffer::from).map_err(Error::from_reason)
    }

    #[napi(js_name = "exchange_keys")]
    pub async fn exchange_keys(
        &self,
        user_id: i64,
        g_a_or_b: Buffer,
        fingerprint: i64,
    ) -> Result<AuthParams> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<AuthParams, String>>();

        let g_a_or_b_vec = g_a_or_b.to_vec();

        let context = Box::into_raw(Box::new(AsyncContextAuthParams {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result: unsafe { std::mem::zeroed() },
            _keep_alive: Some(Box::new(g_a_or_b_vec)),
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback_authparams),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContextAuthParams;
            let keeper = (*ctx)
                ._keep_alive
                .as_ref()
                .unwrap()
                .downcast_ref::<Vec<u8>>()
                .unwrap();

            let rc = ntg_exchange_keys(
                handle,
                user_id,
                keeper.as_ptr(),
                keeper.len() as i32,
                fingerprint,
                std::ptr::addr_of_mut!((*ctx).result),
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_exchange_keys returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        result.map_err(Error::from_reason)
    }

    #[napi(js_name = "skip_exchange")]
    pub async fn skip_exchange(
        &self,
        user_id: i64,
        encryption_key: Buffer,
        is_outgoing: bool,
    ) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let enc_key_vec = encryption_key.to_vec();

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: Some(Box::new(enc_key_vec)),
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let keeper = (*ctx)
                ._keep_alive
                .as_ref()
                .unwrap()
                .downcast_ref::<Vec<u8>>()
                .unwrap();

            let rc = ntg_skip_exchange(
                handle,
                user_id,
                keeper.as_ptr(),
                keeper.len() as i32,
                is_outgoing,
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_skip_exchange returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "connect_p2p")]
    pub async fn connect_p2p(
        &self,
        user_id: i64,
        rtc_servers: Vec<RtcServer>,
        versions_list: Vec<String>,
        p2p_allowed: bool,
    ) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let context = {
            let mut servers_ipv4 = Vec::new();
            let mut servers_ipv6 = Vec::new();
            let mut servers_username = Vec::new();
            let mut servers_password = Vec::new();
            let mut servers_peer_tag = Vec::new();
            let mut servers = Vec::new();

            for s in &rtc_servers {
                let ipv4 = s.ipv4.clone().unwrap_or_default();
                let c_ipv4 = CString::new(ipv4).map_err(|_| Error::from_reason("Invalid ipv4"))?;
                let ipv6 = s.ipv6.clone().unwrap_or_default();
                let c_ipv6 = CString::new(ipv6).map_err(|_| Error::from_reason("Invalid ipv6"))?;
                let username = s.username.clone().unwrap_or_default();
                let c_username =
                    CString::new(username).map_err(|_| Error::from_reason("Invalid username"))?;
                let password = s.password.clone().unwrap_or_default();
                let c_password =
                    CString::new(password).map_err(|_| Error::from_reason("Invalid password"))?;

                let tag_data = s.peer_tag.as_ref().map(|b| b.to_vec());

                servers_ipv4.push(c_ipv4);
                servers_ipv6.push(c_ipv6);
                servers_username.push(c_username);
                servers_password.push(c_password);
                servers_peer_tag.push(tag_data);
            }

            for i in 0..servers_ipv4.len() {
                servers.push(ntg_rtc_server_struct {
                    id: rtc_servers[i].id as u64,
                    ipv4: servers_ipv4[i].as_ptr() as *mut _,
                    ipv6: servers_ipv6[i].as_ptr() as *mut _,
                    username: servers_username[i].as_ptr() as *mut _,
                    password: servers_password[i].as_ptr() as *mut _,
                    port: rtc_servers[i].port,
                    turn: rtc_servers[i].turn,
                    stun: rtc_servers[i].stun,
                    tcp: rtc_servers[i].tcp,
                    peer_tag: servers_peer_tag[i]
                        .as_ref()
                        .map_or(std::ptr::null_mut(), |t| t.as_ptr() as *mut u8),
                    peer_tag_size: if servers_peer_tag[i].is_some() {
                        servers_peer_tag[i].as_ref().unwrap().len() as i32
                    } else {
                        0
                    },
                });
            }

            let mut versions_strings = Vec::new();
            let mut versions = Vec::new();
            for v in versions_list {
                let c_v =
                    CString::new(v).map_err(|_| Error::from_reason("Invalid version string"))?;
                versions_strings.push(c_v);
            }
            for v in &versions_strings {
                versions.push(v.as_ptr() as *mut c_char);
            }

            let keeper = ConnectP2PKeeper {
                servers_ipv4,
                servers_ipv6,
                servers_username,
                servers_password,
                servers_peer_tag,
                servers,
                versions_strings,
                versions,
            };

            Box::into_raw(Box::new(AsyncContext {
                tx,
                error_code: 0,
                error_message: std::ptr::null_mut(),
                result_buffer: std::ptr::null_mut(),
                _keep_alive: Some(Box::new(keeper)),
            }))
        };

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let keeper_ref = (*ctx)
                ._keep_alive
                .as_mut()
                .unwrap()
                .downcast_mut::<ConnectP2PKeeper>()
                .unwrap();

            let rc = ntg_connect_p2p(
                handle,
                user_id,
                keeper_ref.servers.as_mut_ptr(),
                keeper_ref.servers.len() as i32,
                keeper_ref.versions.as_mut_ptr(),
                keeper_ref.versions.len() as i32,
                p2p_allowed,
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_connect_p2p returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "send_signaling_data")]
    pub async fn send_signaling_data(&self, user_id: i64, data: Buffer) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let data_vec = data.to_vec();

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: Some(Box::new(data_vec)),
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let keeper = (*ctx)
                ._keep_alive
                .as_mut()
                .unwrap()
                .downcast_mut::<Vec<u8>>()
                .unwrap();

            let rc = ntg_send_signaling_data(
                handle,
                user_id,
                keeper.as_mut_ptr(),
                keeper.len() as i32,
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_send_signaling_data returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "send_external_frame")]
    pub async fn send_external_frame(
        &self,
        chat_id: i64,
        device: i32,
        data: Buffer,
        frame_data: FrameData,
    ) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let data_vec = data.to_vec();

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: Some(Box::new(data_vec)),
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let keeper = (*ctx)
                ._keep_alive
                .as_mut()
                .unwrap()
                .downcast_mut::<Vec<u8>>()
                .unwrap();

            let frame_data_struct = ntg_frame_data_struct {
                absolute_capture_timestamp_ms: frame_data.absolute_capture_timestamp_ms,
                width: frame_data.width,
                height: frame_data.height,
                rotation: frame_data.rotation,
            };

            let rc = ntg_send_external_frame(
                handle,
                chat_id,
                device,
                keeper.as_mut_ptr(),
                keeper.len() as i32,
                frame_data_struct,
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_send_external_frame returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "send_broadcast_timestamp")]
    pub async fn send_broadcast_timestamp(&self, chat_id: i64, timestamp: i64) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: None,
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let rc = ntg_send_broadcast_timestamp(handle, chat_id, timestamp, ntg_async);
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_send_broadcast_timestamp returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "send_broadcast_part")]
    pub async fn send_broadcast_part(
        &self,
        chat_id: i64,
        segment_id: i64,
        part_id: i32,
        status: i32,
        quality_update: bool,
        data: Buffer,
    ) -> Result<()> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let data_vec = data.to_vec();

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: Some(Box::new(data_vec)),
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContext;
            let keeper = (*ctx)
                ._keep_alive
                .as_mut()
                .unwrap()
                .downcast_mut::<Vec<u8>>()
                .unwrap();

            let rc = ntg_send_broadcast_part(
                handle,
                chat_id,
                segment_id,
                part_id,
                status,
                quality_update,
                keeper.as_ptr(),
                keeper.len() as i32,
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_send_broadcast_part returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => Err(Error::from_reason(err)),
        }
    }

    #[napi(js_name = "cpu_usage")]
    pub async fn cpu_usage(&self) -> Result<f64> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<f64, String>>();

        let context = Box::into_raw(Box::new(AsyncContextF64 {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result: 0.0,
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback_f64),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContextF64;
            let rc = ntg_cpu_usage(handle, std::ptr::addr_of_mut!((*ctx).result), ntg_async);
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_cpu_usage returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        result.map_err(Error::from_reason)
    }

    #[napi(js_name = "calls")]
    pub async fn calls(&self) -> Result<HashMap<String, CallInfo>> {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<HashMap<String, CallInfo>, String>>();

        let context = Box::into_raw(Box::new(AsyncContextCalls {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            result_size: 0,
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback_calls),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || unsafe {
            let ctx = context_addr as *mut AsyncContextCalls;
            let rc = ntg_calls(
                handle,
                std::ptr::addr_of_mut!((*ctx).result_buffer),
                std::ptr::addr_of_mut!((*ctx).result_size),
                ntg_async,
            );
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                let _ = Box::from_raw(ctx);
                return Err(Error::from_reason(format!(
                    "ntg_calls returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        result.map_err(Error::from_reason)
    }

    // ── Helper Async Executor ──────────────────────────────────────────────────

    async fn run_simple_async_op<F>(&self, chat_id: i64, op: F) -> Result<()>
    where
        F: FnOnce(usize, i64, NtgAsyncStruct) -> i32 + Send + 'static,
    {
        let handle = self.handle;
        let pending = Arc::clone(&self.pending_ops);
        let (tx, rx) = oneshot::channel::<std::result::Result<Option<String>, String>>();

        let context = Box::into_raw(Box::new(AsyncContext {
            tx,
            error_code: 0,
            error_message: std::ptr::null_mut(),
            result_buffer: std::ptr::null_mut(),
            _keep_alive: None,
        }));

        let ntg_async = NtgAsyncStruct {
            user_data: context as *mut c_void,
            error_code: unsafe { std::ptr::addr_of_mut!((*context).error_code) },
            error_message: unsafe { std::ptr::addr_of_mut!((*context).error_message) },
            promise: Some(rust_async_callback),
        };

        let context_addr = context as usize;

        pending.fetch_add(1, Ordering::Relaxed);
        let result_blocking = tokio::task::spawn_blocking(move || {
            let rc = op(handle, chat_id, ntg_async);
            pending.fetch_sub(1, Ordering::Release);
            if rc != 0 {
                unsafe {
                    let _ = Box::from_raw(context_addr as *mut AsyncContext);
                }
                return Err(Error::from_reason(format!(
                    "NTgCalls async operation returned error {}",
                    rc
                )));
            }
            Ok(())
        })
        .await
        .map_err(|_| Error::from_reason("Tokio spawn_blocking failed"))?;
        result_blocking?;

        let result = rx
            .await
            .map_err(|_| Error::from_reason("Async operation cancelled"))?;

        match result {
            Ok(_) => Ok(()),
            Err(err) => {
                if err.contains("error code: 1") {
                    Ok(())
                } else {
                    Err(Error::from_reason(err))
                }
            }
        }
    }
}

impl Drop for NtgCalls {
    fn drop(&mut self) {
        while self.pending_ops.load(Ordering::Acquire) > 0 {
            std::hint::spin_loop();
        }
        unsafe {
            ntg_destroy(self.handle);

            let p1 = self.stream_end_cb_ptr.load(Ordering::Acquire);
            if !p1.is_null() {
                drop(Arc::from_raw(p1 as *const _));
            }
            let p2 = self.connection_cb_ptr.load(Ordering::Acquire);
            if !p2.is_null() {
                drop(Arc::from_raw(p2 as *const _));
            }
            let p3 = self.upgrade_cb_ptr.load(Ordering::Acquire);
            if !p3.is_null() {
                drop(Arc::from_raw(p3 as *const _));
            }
            let p4 = self.signaling_cb_ptr.load(Ordering::Acquire);
            if !p4.is_null() {
                drop(Arc::from_raw(p4 as *const _));
            }
            let p5 = self.frames_cb_ptr.load(Ordering::Acquire);
            if !p5.is_null() {
                drop(Arc::from_raw(p5 as *const _));
            }
            let p6 = self.remote_source_cb_ptr.load(Ordering::Acquire);
            if !p6.is_null() {
                drop(Arc::from_raw(p6 as *const _));
            }
            let p7 = self.broadcast_timestamp_cb_ptr.load(Ordering::Acquire);
            if !p7.is_null() {
                drop(Arc::from_raw(p7 as *const _));
            }
            let p8 = self.broadcast_part_cb_ptr.load(Ordering::Acquire);
            if !p8.is_null() {
                drop(Arc::from_raw(p8 as *const _));
            }
        }
    }
}
