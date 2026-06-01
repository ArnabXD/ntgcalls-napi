pub mod broadcast;
pub mod callbacks;
pub mod events;
pub mod ffi;
pub mod p2p;
pub mod session;
pub mod types;
pub mod utils;
pub mod video;

use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;
use std::ffi::c_void;
use std::sync::atomic::{AtomicPtr, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

use crate::callbacks::*;
use crate::ffi::*;
use crate::types::*;

// ── NtgCalls JS Class ────────────────────────────────────────────────────────

#[napi]
#[allow(clippy::type_complexity)]
pub struct NtgCalls {
    pub(crate) handle: usize,
    pub(crate) stream_end_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32), ()>>>>,
    pub(crate) upgrade_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, MediaState), ()>>>>,
    pub(crate) connection_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32), ()>>>>,
    pub(crate) signaling_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, Buffer), ()>>>>,
    pub(crate) frames_cb:
        Arc<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32, Vec<Frame>), ()>>>>,
    pub(crate) remote_source_cb: Arc<Mutex<Option<ThreadsafeFunction<(BigInt, RemoteSource), ()>>>>,
    pub(crate) broadcast_timestamp_cb: Arc<Mutex<Option<ThreadsafeFunction<BigInt, ()>>>>,
    pub(crate) broadcast_part_cb:
        Arc<Mutex<Option<ThreadsafeFunction<(BigInt, SegmentPartRequest), ()>>>>,

    pub(crate) stream_end_cb_ptr:
        AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32), ()>>>>,
    pub(crate) upgrade_cb_ptr:
        AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, MediaState), ()>>>>,
    pub(crate) connection_cb_ptr:
        AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32), ()>>>>,
    pub(crate) signaling_cb_ptr: AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, Buffer), ()>>>>,
    pub(crate) frames_cb_ptr:
        AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, i32, i32, Vec<Frame>), ()>>>>,
    pub(crate) remote_source_cb_ptr:
        AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, RemoteSource), ()>>>>,
    pub(crate) broadcast_timestamp_cb_ptr: AtomicPtr<Mutex<Option<ThreadsafeFunction<BigInt, ()>>>>,
    pub(crate) broadcast_part_cb_ptr:
        AtomicPtr<Mutex<Option<ThreadsafeFunction<(BigInt, SegmentPartRequest), ()>>>>,

    pub(crate) pending_ops: Arc<AtomicUsize>,
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

    // ── Helper Async Executor ──────────────────────────────────────────────────

    pub(crate) async fn run_simple_async_op<F>(&self, chat_id: i64, op: F) -> Result<()>
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
