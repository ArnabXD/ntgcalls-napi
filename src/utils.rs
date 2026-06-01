use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::ffi::{c_char, c_void, CStr};
use std::sync::{Mutex, OnceLock};

use crate::callbacks::{parse_device_info_vector, parse_string_vector};
use crate::ffi::*;
use crate::types::*;

static LOGGER_CALLBACK: OnceLock<Mutex<Option<ThreadsafeFunction<LogMessage, ()>>>> =
    OnceLock::new();

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
