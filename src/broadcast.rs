use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::ffi::c_void;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::callbacks::*;
use crate::ffi::*;

use crate::NtgCalls;

#[napi]
impl NtgCalls {
    #[napi(js_name = "send_broadcast_timestamp")]
    pub async fn send_broadcast_timestamp(
        &self,
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
        #[napi(ts_arg_type = "bigint")] timestamp: i64,
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
        #[napi(ts_arg_type = "bigint")] chat_id: i64,
        #[napi(ts_arg_type = "bigint")] segment_id: i64,
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
}
