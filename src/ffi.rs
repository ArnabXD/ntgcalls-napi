use std::ffi::{c_char, c_void};

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
