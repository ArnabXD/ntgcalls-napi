use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::ffi::{c_char, c_void, CString};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::callbacks::*;
use crate::ffi::*;
use crate::types::*;
use crate::NtgCalls;

#[napi]
impl NtgCalls {
    #[napi(js_name = "create_p2p")]
    pub async fn create_p2p(&self, #[napi(ts_arg_type = "bigint")] user_id: i64) -> Result<()> {
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
        #[napi(ts_arg_type = "bigint")] user_id: i64,
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
        #[napi(ts_arg_type = "bigint")] user_id: i64,
        g_a_or_b: Buffer,
        #[napi(ts_arg_type = "bigint")] fingerprint: i64,
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
        #[napi(ts_arg_type = "bigint")] user_id: i64,
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
        #[napi(ts_arg_type = "bigint")] user_id: i64,
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
    pub async fn send_signaling_data(
        &self,
        #[napi(ts_arg_type = "bigint")] user_id: i64,
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
}
