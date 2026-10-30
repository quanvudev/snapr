import { spawnSync } from "node:child_process";
import { copyFileSync, existsSync, mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { homedir, tmpdir } from "node:os";
import { join } from "node:path";

const REPO = "quanvudev/snapr-releases";
const root = join(import.meta.dir, "..");
const keyDir = join(homedir(), ".tauri");

const fail = (msg: string): never => {
  console.error(`release: ${msg}`);
  process.exit(1);
};

const run = (cmd: string, args: string[], env?: Record<string, string>) => {
  const r = spawnSync(cmd, args, { cwd: root, stdio: "inherit", env: { ...process.env, ...env } });
  if (r.status !== 0) fail(`${cmd} ${args[0]} failed`);
};

const capture = (cmd: string, args: string[]) => {
  const r = spawnSync(cmd, args, { cwd: root, encoding: "utf8" });
  return { ok: r.status === 0, out: (r.stdout ?? "").trim() };
};

const gh = ["gh", "C:\\Program Files\\GitHub CLI\\gh.exe"].find((p) => capture(p, ["--version"]).ok) ?? fail("gh CLI not found");

const conf = JSON.parse(readFileSync(join(root, "src-tauri/tauri.conf.json"), "utf8")).version as string;
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8")).version as string;
const cargo = /^version = "(.+)"/m.exec(readFileSync(join(root, "src-tauri/Cargo.toml"), "utf8"))?.[1];
if (conf !== pkg || conf !== cargo) fail(`versions differ: tauri.conf ${conf}, package.json ${pkg}, Cargo.toml ${cargo}`);
const version = conf;
const tag = `v${version}`;
const notesFile = process.argv[2];
const notes = notesFile ? readFileSync(notesFile, "utf8").trim() : `Snapr ${tag}`;

if (capture("git", ["status", "--porcelain"]).out) fail("working tree is not clean");
if (capture(gh, ["release", "view", tag, "-R", REPO]).ok) fail(`${tag} is already released on ${REPO}`);

const key = process.env.TAURI_SIGNING_PRIVATE_KEY ?? readFileSync(join(keyDir, "snapr.key"), "utf8").trim();
const password = process.env.TAURI_SIGNING_PRIVATE_KEY_PASSWORD ?? readFileSync(join(keyDir, "snapr.key.password"), "utf8").trim();

console.log(`release: building ${tag}`);
const signing = { TAURI_SIGNING_PRIVATE_KEY: key, TAURI_SIGNING_PRIVATE_KEY_PASSWORD: password };
run("bun", ["run", "tauri", "build"], signing);

const name = `Snapr_${version}_x64-setup.exe`;
const installer = join(root, "src-tauri/target/release/bundle/nsis", name);
if (!existsSync(installer) || !existsSync(`${installer}.sig`)) fail(`${name} or its .sig is missing`);

const platform = {
  signature: readFileSync(`${installer}.sig`, "utf8").trim(),
  url: `https://github.com/${REPO}/releases/download/${tag}/${name}`,
};
const out = mkdtempSync(join(tmpdir(), "snapr-release-"));
const exe = join(out, `Snapr_${version}_x64.exe`);
copyFileSync(join(root, "src-tauri/target/release/snapr-v2.exe"), exe);
run("bun", ["run", "tauri", "signer", "sign", exe], signing);
if (!existsSync(`${exe}.sig`)) fail(`${exe}.sig is missing`);
const latest = join(out, "latest.json");
writeFileSync(
  latest,
  JSON.stringify(
    {
      version,
      notes,
      pub_date: new Date().toISOString(),
      platforms: { "windows-x86_64": platform, "windows-x86_64-nsis": platform },
    },
    null,
    2,
  ),
);

console.log(`release: publishing ${tag} to ${REPO}`);
run(gh, ["release", "create", tag, "-R", REPO, "--target", "main", "--latest", "--title", `Snapr ${tag}`, "--notes", notes, installer, `${installer}.sig`, exe, `${exe}.sig`, latest]);
console.log(`release: done, https://github.com/${REPO}/releases/tag/${tag}`);
run("bun", ["scripts/scoop.ts", version, exe, gh]);
