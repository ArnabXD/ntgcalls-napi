import { createRequire } from "node:module";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const __dirname = dirname(fileURLToPath(import.meta.url));

const addonPath = join(__dirname, "ntgcalls.node");
const {
  NtgCalls,
  get_version,
  get_protocol,
  enable_g_lib_loop,
  get_media_devices,
  register_logger,
} = require(addonPath);

export {
  enable_g_lib_loop,
  get_media_devices,
  get_protocol,
  get_version,
  NtgCalls,
  register_logger,
};
