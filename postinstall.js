import { existsSync, mkdirSync, writeFileSync } from "node:fs";
import { chmod } from "node:fs/promises";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));

const REPO = "ArnabXD/ntgcalls-napi";

function getPlatformKey() {
  const { platform, arch } = process;
  if (platform === "linux" && arch === "x64") return "linux-x64";
  if (platform === "linux" && arch === "arm64") return "linux-arm64";
  if (platform === "darwin" && arch === "arm64") return "macos-arm64";
  if (platform === "win32" && arch === "x64") return "windows-x64";
  throw new Error(`Unsupported platform: ${platform} ${arch}`);
}

async function getLatestTag() {
  const res = await fetch(
    `https://api.github.com/repos/${REPO}/releases/latest`,
  );
  if (!res.ok) throw new Error(`GitHub API returned HTTP ${res.status}`);
  const data = await res.json();
  if (!data.tag_name) throw new Error("Could not determine latest release tag");
  return data.tag_name;
}

async function downloadZip(url) {
  const res = await fetch(url, { redirect: "follow" });
  if (!res.ok) throw new Error(`HTTP ${res.status} downloading ${url}`);
  return Buffer.from(await res.arrayBuffer());
}

async function extractZip(buffer, destDir) {
  mkdirSync(destDir, { recursive: true });

  // Write zip to a temp file then extract using platform tools
  const tmpZip = join(destDir, "_ntgcalls_tmp.zip");
  writeFileSync(tmpZip, buffer);

  const { execFileSync } = await import("node:child_process");

  try {
    if (process.platform === "win32") {
      execFileSync("powershell", [
        "-NoProfile",
        "-Command",
        `Expand-Archive -Force -Path '${tmpZip}' -DestinationPath '${destDir}'`,
      ]);
    } else {
      execFileSync("unzip", ["-o", tmpZip, "-d", destDir]);
    }
  } finally {
    // Clean up temp zip regardless of success/failure
    try {
      const { unlinkSync } = await import("node:fs");
      unlinkSync(tmpZip);
    } catch {}
  }
}

async function main() {
  const platformKey = getPlatformKey();
  const tag = await getLatestTag();

  const zipName = `ntgcalls-napi.${platformKey}.zip`;
  const url = `https://github.com/${REPO}/releases/download/${tag}/${zipName}`;

  console.log(`Downloading ${zipName} (release ${tag})...`);
  const buffer = await downloadZip(url);
  await extractZip(buffer, __dirname);

  const nodePath = join(__dirname, "ntgcalls.node");
  if (!existsSync(nodePath)) {
    throw new Error("Extraction succeeded but ntgcalls.node was not found");
  }

  if (process.platform !== "win32") {
    await chmod(nodePath, 0o755);
  }

  console.log("ntgcalls-napi: native addon ready.");
}

main().catch((err) => {
  console.error("ntgcalls-napi postinstall failed:", err.message);
  process.exit(1);
});
