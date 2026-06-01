import { EventEmitter } from "node:events";
import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const __dirname = dirname(fileURLToPath(import.meta.url));

const addonPath = join(__dirname, "ntgcalls.node");
const {
  NtgCalls: NativeNtgCalls,
  get_version,
  get_protocol,
  enable_g_lib_loop,
  get_media_devices,
  register_logger,
} = require(addonPath);

class NtgCalls extends EventEmitter {
  constructor() {
    super();
    this.native = new NativeNtgCalls();

    // Register callbacks from the native thread safely
    this.native.on_stream_end((...args) => {
      const tuple = args[1];
      if (tuple) {
        this.emit("stream-end", tuple[0], tuple[1], tuple[2]);
      }
    });

    this.native.on_upgrade((...args) => {
      const tuple = args[1];
      if (tuple) {
        this.emit("upgrade", tuple[0], tuple[1]);
      }
    });

    this.native.on_connection_change((...args) => {
      const tuple = args[1];
      if (tuple) {
        this.emit("connection-change", tuple[0], tuple[1], tuple[2]);
      }
    });

    this.native.on_signaling_data((...args) => {
      const tuple = args[1];
      if (tuple) {
        this.emit("signaling-data", tuple[0], tuple[1]);
      }
    });

    this.native.on_frames((...args) => {
      const tuple = args[1];
      if (tuple) {
        this.emit("frames", tuple[0], tuple[1], tuple[2], tuple[3]);
      }
    });

    this.native.on_remote_source_change((...args) => {
      const tuple = args[1];
      if (tuple) {
        this.emit("remote-source-change", tuple[0], tuple[1]);
      }
    });

    this.native.on_request_broadcast_timestamp((...args) => {
      const val = args[1];
      if (val !== undefined && val !== null) {
        this.emit("request-broadcast-timestamp", val);
      }
    });

    this.native.on_request_broadcast_part((...args) => {
      const tuple = args[1];
      if (tuple) {
        this.emit("request-broadcast-part", tuple[0], tuple[1]);
      }
    });
  }

  // Helper to ensure chatId/userId is passed as a standard JS Number to the N-API FFI boundary.
  // Although the TS definitions declare it as bigint | number, napi-rs version 3 strictly
  // expects a JS Number at runtime for i64 arguments. Converting bigint to number is safe
  // since Telegram IDs fit well within Number.MAX_SAFE_INTEGER.
  id(value) {
    if (typeof value === "bigint") {
      return Number(value);
    }
    return value;
  }

  async create(chatId) {
    return this.native.create(this.id(chatId));
  }

  async connect(chatId, params, isPresentation = false) {
    return this.native.connect(this.id(chatId), params, isPresentation);
  }

  async init_presentation(chatId) {
    return this.native.init_presentation(this.id(chatId));
  }

  async stop_presentation(chatId) {
    return this.native.stop_presentation(this.id(chatId));
  }

  async set_stream_sources(chatId, streamMode, desc) {
    return this.native.set_stream_sources(this.id(chatId), streamMode, desc);
  }

  async set_audio_source(chatId, ffmpegCmd) {
    return this.native.set_audio_source(this.id(chatId), ffmpegCmd);
  }

  async pause(chatId) {
    return this.native.pause(this.id(chatId));
  }

  async resume(chatId) {
    return this.native.resume(this.id(chatId));
  }

  async mute(chatId) {
    return this.native.mute(this.id(chatId));
  }

  async unmute(chatId) {
    return this.native.unmute(this.id(chatId));
  }

  async stop(chatId) {
    return this.native.stop(this.id(chatId));
  }

  async time(chatId, mode = null) {
    return this.native.time(this.id(chatId), mode);
  }

  async get_state(chatId) {
    return this.native.get_state(this.id(chatId));
  }

  async get_connection_mode(chatId) {
    return this.native.get_connection_mode(this.id(chatId));
  }

  async create_p2p(userId) {
    return this.native.create_p2p(this.id(userId));
  }

  async init_exchange(userId, dhConfig, gAHash) {
    return this.native.init_exchange(this.id(userId), dhConfig, gAHash);
  }

  async exchange_keys(userId, gAOrB, fingerprint) {
    return this.native.exchange_keys(
      this.id(userId),
      gAOrB,
      this.id(fingerprint),
    );
  }

  async skip_exchange(userId, encryptionKey, isOutgoing) {
    return this.native.skip_exchange(
      this.id(userId),
      encryptionKey,
      isOutgoing,
    );
  }

  async connect_p2p(userId, rtcServers, versionsList, p2PAllowed) {
    const serversMapped = rtcServers.map((s) => ({
      ...s,
      id: this.id(s.id),
    }));
    return this.native.connect_p2p(
      this.id(userId),
      serversMapped,
      versionsList,
      p2PAllowed,
    );
  }

  async send_signaling_data(userId, data) {
    return this.native.send_signaling_data(this.id(userId), data);
  }

  async add_incoming_video(chatId, endpoint, ssrcGroupsList) {
    return this.native.add_incoming_video(
      this.id(chatId),
      endpoint,
      ssrcGroupsList,
    );
  }

  async remove_incoming_video(chatId, endpoint) {
    return this.native.remove_incoming_video(this.id(chatId), endpoint);
  }

  async send_external_frame(chatId, device, data, frameData) {
    const frameDataMapped = {
      ...frameData,
      absoluteCaptureTimestampMs: this.id(frameData.absoluteCaptureTimestampMs),
    };
    return this.native.send_external_frame(
      this.id(chatId),
      device,
      data,
      frameDataMapped,
    );
  }

  async send_broadcast_timestamp(chatId, timestamp) {
    return this.native.send_broadcast_timestamp(
      this.id(chatId),
      this.id(timestamp),
    );
  }

  async send_broadcast_part(
    chatId,
    segmentId,
    partId,
    status,
    qualityUpdate,
    data,
  ) {
    return this.native.send_broadcast_part(
      this.id(chatId),
      this.id(segmentId),
      partId,
      status,
      qualityUpdate,
      data,
    );
  }

  async cpu_usage() {
    return this.native.cpu_usage();
  }

  async calls() {
    return this.native.calls();
  }
}

export {
  enable_g_lib_loop,
  get_media_devices,
  get_protocol,
  get_version,
  NtgCalls,
  register_logger,
};
