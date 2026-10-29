import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";

export type RecordMode = "fullscreen" | "window" | "area";

export type HotkeyMode = "fixed" | "manual";
export type Hotkeys = { mode: HotkeyMode; modifiers: string; fullscreen: string; window: string; area: string; text: string; record: string; draw: string; ring: string; manual: string[] };
export type Theme = "system" | "light" | "dark";
export type BuiltinId = "fullscreen" | "window" | "area" | "text" | "record" | "draw" | "gallery" | "settings";
export type RingItem =
  | { kind: "builtin"; id: BuiltinId; key?: string | null }
  | { kind: "app"; label: string; path: string; args: string; admin: boolean; key?: string | null };
export type OcrEngine = "windows" | "paddle";
export type PaddleSize = "tiny" | "small" | "medium";
export type Config = {
  output_dir: string;
  copy_to_clipboard: boolean;
  preview_seconds: number;
  record_fps: number;
  record_countdown: boolean;
  record_mic: string;
  record_camera: string;
  record_speaker: string;
  record_mode: RecordMode;
  text_capture: boolean;
  ocr_engine: OcrEngine;
  paddle_model: PaddleSize;
  update_check: boolean;
  padding: { enabled: boolean; size: number; image: string; round: boolean; radius: number; wallpaper: boolean };
  language: "vi" | "en";
  theme: Theme;
  hotkeys: Hotkeys;
  ring: { enabled: boolean; keys: boolean; idle_close: number; items?: RingItem[] | null };
};
export type Row = { combo: string; error: "invalid" | "needs_modifier" | "duplicate" | null; in_use: boolean };
export type Loaded = { config: Config; autostart: boolean; dir: string; first_run: boolean; ocr_available: boolean; windows_ocr: boolean };
export type PaddleProgress = { size: PaddleSize; done: number; total: number };
export type PaddleStatus = { ready: PaddleSize[]; bytes: Record<PaddleSize, number>; downloading: PaddleProgress | null; runtime: boolean };
export type PaddleDone = { size: PaddleSize; error: string | null };
export type UpdatePhase = "idle" | "checking" | "latest" | "available" | "blocked" | "downloading" | "installing" | "error";
export type UpdateStatus = { state: UpdatePhase; version: string | null; done: number; total: number; error: string | null };
export type InstalledApp = { name: string; path: string };
export type GalleryItem = { path: string; date_ms: number; size: number; kind: "photo" | "video" | "other" };
export type BulkFailure = { path: string; error: string };

export type InputDevice = { id: string; name: string; default: boolean };
export type PanelPhase = "closed" | "setup" | "picking" | "countdown" | "recording";
export type PanelState = {
  phase: PanelPhase;
  mic: string;
  camera: string;
  speaker: string;
  mode: RecordMode;
  started_ms: number | null;
  paused_total_ms: number;
  paused_at_ms: number | null;
  camera_hidden: boolean;
  warning: "mic_lost" | null;
};

export type Recognized = { qr: string[]; text: string };

export const api = {
  load: () => invoke<Loaded>("load"),
  defaults: () => invoke<Config>("defaults"),
  resolveDir: (outputDir: string) => invoke<string>("resolve_dir", { outputDir }),
  check: (hotkeys: Hotkeys, text: boolean) => invoke<Row[]>("check", { hotkeys, text }),
  setListening: (on: boolean) => invoke<void>("set_listening", { on }),
  save: (config: Config, autostart: boolean) => invoke("save", { config, autostart }),
  openFolder: (outputDir: string) => invoke("open_folder", { outputDir }),
  openConfigFile: () => invoke("open_config_file"),
  openLanguageSettings: () => invoke("open_language_settings"),
  openSettings: () => invoke("open_settings"),
  openUrl,
  version: getVersion,
  galleryList: () => invoke<GalleryItem[]>("gallery_list"),
  galleryOpen: (path: string) => invoke("gallery_open", { path }),
  galleryReveal: (path: string) => invoke("gallery_reveal", { path }),
  galleryCopy: (path: string) => invoke("gallery_copy", { path }),
  galleryCopyFiles: (paths: string[]) => invoke("gallery_copy_files", { paths }),
  galleryPin: (path: string) => invoke("gallery_pin", { path }),
  galleryText: (path: string) => invoke<Recognized>("gallery_text", { path }),
  galleryPinMany: (paths: string[]) => invoke("gallery_pin_many", { paths }),
  galleryDelete: (path: string) => invoke("gallery_delete", { path }),
  galleryDeleteMany: (paths: string[]) => invoke<BulkFailure[]>("gallery_delete_many", { paths }),
  paddleStatus: () => invoke<PaddleStatus>("paddle_status"),
  paddleDownload: (size: PaddleSize) => invoke("paddle_download", { size }),
  paddleCancel: () => invoke("paddle_cancel"),
  paddleDelete: (size: PaddleSize) => invoke("paddle_delete", { size }),
  onPaddleProgress: (cb: (p: PaddleProgress) => void) => listen<PaddleProgress>("paddle-progress", (e) => cb(e.payload)),
  onPaddleDone: (cb: (d: PaddleDone) => void) => listen<PaddleDone>("paddle-done", (e) => cb(e.payload)),
  updateStatus: () => invoke<UpdateStatus>("update_status"),
  updateCheck: () => invoke("update_check"),
  updateInstall: () => invoke("update_install"),
  onUpdateStatus: (cb: (s: UpdateStatus) => void) => listen<UpdateStatus>("update-status", (e) => cb(e.payload)),
  listApps: () => invoke<InstalledApp[]>("list_apps"),
  ringIcon: (path: string) => invoke<ArrayBuffer>("ring_icon", { path }),
  thumbSrc: (path: string) => convertFileSrc(path, "thumb"),
  mediaSrc: (path: string) => convertFileSrc(path, "media"),
  onSettingsOpen: (cb: (section: string) => void) => listen<string>("settings-open", (e) => cb(e.payload)),
  onGalleryChanged: (cb: () => void) => listen("gallery-changed", cb),
  recordPanelState: () => invoke<PanelState>("record_panel_state"),
  recordInputs: () => invoke<InputDevice[]>("record_inputs"),
  recordOutputs: () => invoke<InputDevice[]>("record_outputs"),
  recordSelect: (mic: string, camera: string, speaker: string, mode: RecordMode) =>
    invoke("record_select", { mic, camera, speaker, mode }),
  recordPanelFit: (width: number, height: number) => invoke("record_panel_fit", { width, height }),
  recordStart: () => invoke("record_start"),
  recordClose: () => invoke("record_close"),
  recordPause: () => invoke("record_pause"),
  recordResume: () => invoke("record_resume"),
  recordStop: () => invoke("record_stop"),
  recordDiscard: () => invoke("record_discard"),
  recordCameraToggle: () => invoke("record_camera_toggle"),
  openCameraSettings: () => invoke("open_camera_settings"),
  onRecordPanel: (cb: (s: PanelState) => void) => listen<PanelState>("record-panel", (e) => cb(e.payload)),
  onRecordLevel: (cb: (level: number) => void) => listen<number>("record-level", (e) => cb(e.payload)),
  hide: async () => {
    await invoke("set_listening", { on: false });
    await getCurrentWindow().hide();
  },
  setWindowTheme: (theme: Theme) => getCurrentWindow().setTheme(theme === "system" ? null : theme),
};
