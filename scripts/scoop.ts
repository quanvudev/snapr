import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";

const REPO = "quanvudev/snapr-releases";
const PATH = "bucket/snapr.json";
const DOWNLOAD = `https://github.com/${REPO}/releases/download`;

const fail = (msg: string): never => {
  console.error(`scoop: ${msg}`);
  process.exit(1);
};

const [version, exe, gh = "gh"] = process.argv.slice(2);
if (!version || !exe) fail("usage: bun scripts/scoop.ts <version> <exe> [gh]");

const uninstaller = [
  "$run = 'HKCU:\\Software\\Microsoft\\Windows\\CurrentVersion\\Run'",
  "$entry = (Get-ItemProperty $run -ErrorAction SilentlyContinue).snapr",
  "if ($entry -and $entry.Contains((Split-Path $dir))) { Remove-ItemProperty $run -Name snapr }",
];

const manifest = {
  version,
  description: "Screenshots, screen recording and text capture from the tray",
  homepage: `https://github.com/${REPO}`,
  license: "Proprietary",
  notes: "Close Snapr before running scoop update snapr.",
  architecture: {
    "64bit": {
      url: `${DOWNLOAD}/v${version}/Snapr_${version}_x64.exe#/snapr.exe`,
      hash: createHash("sha256").update(readFileSync(exe)).digest("hex"),
    },
  },
  bin: "snapr.exe",
  shortcuts: [["snapr.exe", "Snapr"]],
  uninstaller: { script: uninstaller },
  checkver: { github: `https://github.com/${REPO}` },
  autoupdate: {
    architecture: {
      "64bit": { url: `${DOWNLOAD}/v$version/Snapr_$version_x64.exe#/snapr.exe` },
    },
  },
};

const body = `${JSON.stringify(manifest, null, 4)}\n`;
if (process.env.SCOOP_DRY_RUN) {
  console.log(body);
  process.exit(0);
}

const current = spawnSync(gh, ["api", `repos/${REPO}/contents/${PATH}`, "-q", ".sha"], { encoding: "utf8" });
const sha = current.status === 0 ? current.stdout.trim() : "";
const args = ["api", "-X", "PUT", `repos/${REPO}/contents/${PATH}`, "-f", `message=scoop: snapr ${version}`, "-f", `content=${Buffer.from(body).toString("base64")}`];
if (sha) args.push("-f", `sha=${sha}`);
const put = spawnSync(gh, args, { stdio: ["ignore", "ignore", "inherit"] });
if (put.status !== 0) fail(`updating ${PATH} on ${REPO} failed`);
console.log(`scoop: ${PATH} on ${REPO} now names ${version}`);
