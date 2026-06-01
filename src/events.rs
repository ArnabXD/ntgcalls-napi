use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::ffi::c_void;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::callbacks::*;
use crate::ffi::*;
use crate::types::*;
use crate::NtgCalls;

#[napi]
impl NtgCalls {
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
}
