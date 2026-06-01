use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use std::collections::HashMap;
use std::ffi::{c_char, c_void, CStr, CString};
use std::sync::atomic::Ordering;
use std::sync::Mutex;
use tokio::sync::oneshot;

use crate::ffi::*;
use crate::types::*;

// ── Shared Helper Memory Free ───────────────────────────────────────────────

pub(crate) unsafe fn get_async_error_message(code: i32, msg_ptr: *mut c_char) -> String {
    if !msg_ptr.is_null() {
        let c_str = CStr::from_ptr(msg_ptr);
        let msg = c_str.to_string_lossy().into_owned();
        free(msg_ptr as *mut c_void);
        msg
    } else {
        format!("NTgCalls async error: {}", code)
    }
}

pub(crate) unsafe fn parse_string_vector(data: *mut *mut c_char, size: i32) -> Vec<String> {
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

pub(crate) unsafe fn parse_device_info_vector(
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

// ── Async Context Definitions ────────────────────────────────────────────────

pub struct AsyncContext {
    pub tx: oneshot::Sender<std::result::Result<Option<String>, String>>,
    pub error_code: i32,
    pub error_message: *mut c_char,
    pub result_buffer: *mut c_char,
    pub _keep_alive: Option<Box<dyn std::any::Any + Send + Sync>>,
}

pub struct AsyncContextI64 {
    pub tx: oneshot::Sender<std::result::Result<i64, String>>,
    pub error_code: i32,
    pub error_message: *mut c_char,
    pub result: i64,
}

pub struct AsyncContextU32 {
    pub tx: oneshot::Sender<std::result::Result<u32, String>>,
    pub error_code: i32,
    pub error_message: *mut c_char,
    pub result: u32,
    pub _keep_alive: Option<Box<dyn std::any::Any + Send + Sync>>,
}

pub struct AsyncContextMediaState {
    pub tx: oneshot::Sender<std::result::Result<MediaState, String>>,
    pub error_code: i32,
    pub error_message: *mut c_char,
    pub result: ntg_media_state_struct,
}

pub struct AsyncContextI32 {
    pub tx: oneshot::Sender<std::result::Result<i32, String>>,
    pub error_code: i32,
    pub error_message: *mut c_char,
    pub result: i32,
}

pub struct AsyncContextBytes {
    pub tx: oneshot::Sender<std::result::Result<Vec<u8>, String>>,
    pub error_code: i32,
    pub error_message: *mut c_char,
    pub result_buffer: *mut u8,
    pub result_size: i32,
    pub _keep_alive: Option<Box<dyn std::any::Any + Send + Sync>>,
}

pub struct AsyncContextAuthParams {
    pub tx: oneshot::Sender<std::result::Result<AuthParams, String>>,
    pub error_code: i32,
    pub error_message: *mut c_char,
    pub result: ntg_auth_params_struct,
    pub _keep_alive: Option<Box<dyn std::any::Any + Send + Sync>>,
}

pub struct AsyncContextF64 {
    pub tx: oneshot::Sender<std::result::Result<f64, String>>,
    pub error_code: i32,
    pub error_message: *mut c_char,
    pub result: f64,
}

pub struct AsyncContextCalls {
    pub tx: oneshot::Sender<std::result::Result<HashMap<String, CallInfo>, String>>,
    pub error_code: i32,
    pub error_message: *mut c_char,
    pub result_buffer: *mut ntg_call_info_struct,
    pub result_size: i32,
}

// ── Raw Async Callbacks ──────────────────────────────────────────────────────

pub(crate) unsafe extern "C" fn rust_async_callback(user_data: *mut c_void) {
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

pub(crate) unsafe extern "C" fn rust_async_callback_i64(user_data: *mut c_void) {
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

pub(crate) unsafe extern "C" fn rust_async_callback_u32(user_data: *mut c_void) {
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

pub(crate) unsafe extern "C" fn rust_async_callback_mediastate(user_data: *mut c_void) {
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

pub(crate) unsafe extern "C" fn rust_async_callback_i32(user_data: *mut c_void) {
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

pub(crate) unsafe extern "C" fn rust_async_callback_bytes(user_data: *mut c_void) {
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

pub(crate) unsafe extern "C" fn rust_async_callback_authparams(user_data: *mut c_void) {
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

pub(crate) unsafe extern "C" fn rust_async_callback_f64(user_data: *mut c_void) {
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

pub(crate) unsafe extern "C" fn rust_async_callback_calls(user_data: *mut c_void) {
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

pub(crate) unsafe extern "C" fn raw_stream_end_callback(
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

pub(crate) unsafe extern "C" fn raw_upgrade_callback(
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

pub(crate) unsafe extern "C" fn raw_connection_callback(
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

pub(crate) unsafe extern "C" fn raw_signaling_callback(
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

pub(crate) unsafe extern "C" fn raw_frames_callback(
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

pub(crate) unsafe extern "C" fn raw_remote_source_callback(
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

pub(crate) unsafe extern "C" fn raw_broadcast_timestamp_callback(
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

pub(crate) unsafe extern "C" fn raw_broadcast_part_callback(
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
pub struct MediaDescriptionKeeper {
    pub microphone_input: Option<CString>,
    pub speaker_input: Option<CString>,
    pub camera_input: Option<CString>,
    pub screen_input: Option<CString>,
    pub microphone: Option<Box<ntg_audio_description_struct>>,
    pub speaker: Option<Box<ntg_audio_description_struct>>,
    pub camera: Option<Box<ntg_video_description_struct>>,
    pub screen: Option<Box<ntg_video_description_struct>>,
}

unsafe impl Send for MediaDescriptionKeeper {}
unsafe impl Sync for MediaDescriptionKeeper {}

#[allow(dead_code)]
pub struct ConnectP2PKeeper {
    pub servers_ipv4: Vec<CString>,
    pub servers_ipv6: Vec<CString>,
    pub servers_username: Vec<CString>,
    pub servers_password: Vec<CString>,
    pub servers_peer_tag: Vec<Option<Vec<u8>>>,
    pub servers: Vec<ntg_rtc_server_struct>,
    pub versions_strings: Vec<CString>,
    pub versions: Vec<*mut c_char>,
}

unsafe impl Send for ConnectP2PKeeper {}
unsafe impl Sync for ConnectP2PKeeper {}

#[allow(dead_code)]
pub struct AddIncomingVideoKeeper {
    pub endpoint: CString,
    pub semantics_strings: Vec<CString>,
    pub ssrcs_vecs: Vec<Vec<u32>>,
    pub ssrc_groups: Vec<ntg_ssrc_group_struct>,
}

unsafe impl Send for AddIncomingVideoKeeper {}
unsafe impl Sync for AddIncomingVideoKeeper {}

#[allow(dead_code)]
pub struct InitExchangeKeeper {
    pub p: Vec<u8>,
    pub random: Vec<u8>,
    pub dh_config: Box<ntg_dh_config_struct>,
    pub g_a_hash: Vec<u8>,
}

unsafe impl Send for InitExchangeKeeper {}
unsafe impl Sync for InitExchangeKeeper {}
