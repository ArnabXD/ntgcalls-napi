use napi::bindgen_prelude::*;
use napi_derive::napi;
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
    #[napi(js_name = "add_incoming_video")]
    pub async fn add_incoming_video(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
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
    pub async fn remove_incoming_video(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
        endpoint: String,
    ) -> Result<()> {
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

    #[napi(js_name = "send_external_frame")]
    pub async fn send_external_frame(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
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
}
