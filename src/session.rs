use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::collections::HashMap;
use std::ffi::{c_void, CString};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::callbacks::*;
use crate::ffi::*;
use crate::types::*;
use crate::NtgCalls;

#[napi]
impl NtgCalls {
    #[napi(js_name = "create")]
    pub async fn create(&self, #[napi(ts_arg_type = "bigint")] chat_id: i64) -> Result<String> {
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
    pub async fn connect(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
        params: String,
        is_presentation: bool,
    ) -> Result<()> {
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
    pub async fn init_presentation(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
    ) -> Result<String> {
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
    pub async fn stop_presentation(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
    ) -> Result<()> {
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
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
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
    pub async fn set_audio_source(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
        ffmpeg_cmd: String,
    ) -> Result<()> {
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
    pub async fn pause(&self, #[napi(ts_arg_type = "bigint")] chat_id: i64) -> Result<()> {
        self.run_simple_async_op(chat_id, |h, cid, a| unsafe { ntg_pause(h, cid, a) })
            .await
    }

    #[napi(js_name = "resume")]
    pub async fn resume(&self, #[napi(ts_arg_type = "bigint")] chat_id: i64) -> Result<()> {
        self.run_simple_async_op(chat_id, |h, cid, a| unsafe { ntg_resume(h, cid, a) })
            .await
    }

    #[napi(js_name = "mute")]
    pub async fn mute(&self, #[napi(ts_arg_type = "bigint")] chat_id: i64) -> Result<()> {
        self.run_simple_async_op(chat_id, |h, cid, a| unsafe { ntg_mute(h, cid, a) })
            .await
    }

    #[napi(js_name = "unmute")]
    pub async fn unmute(&self, #[napi(ts_arg_type = "bigint")] chat_id: i64) -> Result<()> {
        self.run_simple_async_op(chat_id, |h, cid, a| unsafe { ntg_unmute(h, cid, a) })
            .await
    }

    #[napi(js_name = "stop")]
    pub async fn stop(&self, #[napi(ts_arg_type = "bigint")] chat_id: i64) -> Result<()> {
        self.run_simple_async_op(chat_id, |h, cid, a| unsafe { ntg_stop(h, cid, a) })
            .await
    }

    #[napi(js_name = "time", ts_return_type = "Promise<bigint>")]
    pub async fn time(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
        mode: Option<i32>,
    ) -> Result<i64> {
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
    pub async fn get_state(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
    ) -> Result<MediaState> {
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
    pub async fn get_connection_mode(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
    ) -> Result<i32> {
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
}
