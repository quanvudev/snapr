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
export type WhisperModel = "base" | "small" | "turbo";
export type Runtime = "cpu" | "directml" | "cuda";
export type Acceleration = "auto" | Runtime;
export type TranscriptLanguage = "auto" | "vi" | "en";
export type Track = "original" | (string & {});
export type RedactStyle = "black" | "pixelate";
export type Redact = {
  auto_capture: boolean;
  engine: "windows" | "paddle";
  style: RedactStyle;
  email: boolean;
  phone: boolean;
  card: boolean;
  ipv4: boolean;
  secrets: boolean;
  patterns: string[];
};
export type Config = {
  output_dir: string;
  copy_to_clipboard: boolean;
  preview_seconds: number;
  record_fps: number;
  record_quality: Quality;
  record_compress: boolean;
  record_compress_height: number;
  record_transcribe: boolean;
  record_translate: boolean;
  hymt_license_accepted: boolean;
  whisper_model: WhisperModel;
  transcript_language: TranscriptLanguage;
  record_countdown: boolean;
  record_mic: string;
  record_camera: string;
  record_speaker: string;
  record_mode: RecordMode;
  text_capture: boolean;
  ocr_engine: OcrEngine;
  paddle_model: PaddleSize;
  ai_acceleration: Acceleration;
  update_check: boolean;
  padding: { enabled: boolean; size: number; image: string; round: boolean; radius: number; wallpaper: boolean };
  redact: Redact;
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
export type WhisperProgress = { model: WhisperModel; gpu: boolean; done: number; total: number };
export type WhisperStatus = {
  ready: WhisperModel[];
  bytes: Record<WhisperModel, number>;
  gpu_ready: WhisperModel[];
  gpu_bytes: Record<WhisperModel, number>;
  downloading: WhisperProgress | null;
};
export type WhisperDone = { model: WhisperModel; gpu: boolean; error: string | null };
export type Adapter = { index: number; name: string; vendor: number; memory: number };
export type GpuRuntime = Exclude<Runtime, "cpu">;
export type RuntimeStatus = { id: GpuRuntime; ready: boolean; bytes: number; disk: number };
export type AccelerationProgress = { runtime: GpuRuntime; done: number; total: number };
export type AccelerationDone = { runtime: GpuRuntime; error: string | null };
export type AccelerationStatus = {
  adapter: Adapter | null;
  recommended: Runtime;
  resolved: Runtime;
  loaded: Runtime | null;
  restart_needed: boolean;
  runtimes: RuntimeStatus[];
  downloading: AccelerationProgress | null;
  error: string | null;
};
export type TranslateProgress = { done: number; total: number };
export type TranslateStatus = { ready: boolean; bytes: number; downloading: TranslateProgress | null };
export type TranslateDone = { error: string | null };
export type Segment = { start_ms: number; end_ms: number; text: string };
export type UpdatePhase = "idle" | "checking" | "latest" | "available" | "blocked" | "downloading" | "installing" | "error";
export type UpdateStatus = { state: UpdatePhase; version: string | null; done: number; total: number; error: string | null };
export type InstalledApp = { name: string; path: string };
export type GalleryItem = {
  path: string;
  date_ms: number;
  size: number;
  kind: "photo" | "video" | "other";
  transcript: boolean;
  translations: string[];
  language: string | null;
};
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

export type Quality = "low" | "medium" | "high";
export type CompressHeight = 1080 | 720 | null;
export type SubtitlePosition = "bottom" | "top";
export type SubtitleSize = "small" | "medium" | "large";

export type TaskSpec =
  | {
      kind: "gif";
      path: string;
      start_ms: number;
      end_ms: number | null;
      width: number | null;
      fps: number;
    }
  | { kind: "compress"; path: string; quality: Quality; height: CompressHeight }
  | { kind: "transcribe"; path: string }
  | { kind: "translate"; path: string; target?: string | null }
  | { kind: "subtitle"; path: string; position: SubtitlePosition; size: SubtitleSize; style: SubtitleStyle; track: Track };
export type SubtitleStyle = { text_color: string; background: string; opacity: number };

export type CompressInfo = {
  width: number;
  height: number;
  fps: number;
  duration_ms: number;
  size: number;
  estimates: { quality: string; height: number | null; bytes: number }[];
};
export type QueueItem = {
  id: number;
  kind: string;
  name: string;
  state: "waiting" | "running";
  done: number;
  total: number;
  source: string | null;
  detail: string | null;
};
export type HistoryState = "done" | "failed" | "cancelled";
export type HistoryItem = {
  id: number;
  kind: string;
  name: string;
  source: string | null;
  detail: string | null;
  output: string | null;
  state: HistoryState;
  error: string | null;
  retryable: boolean;
};
export type QueueEnded = {
  id: number;
  kind: string;
  name: string;
  output: string | null;
  error: string | null;
  source: string | null;
  detail: string | null;
};

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
  queueAdd: (spec: TaskSpec) => invoke<number>("queue_add", { spec }),
  compressInfo: (path: string) => invoke<CompressInfo>("compress_info", { path }),
  queueList:() => invoke<QueueItem[]>("queue_list"),
  queueCancel: (id: number) => invoke("queue_cancel", { id }),
  queueCancelAll: () => invoke("queue_cancel_all"),
  queueHistory: () => invoke<HistoryItem[]>("queue_history"),
  queueHistoryClear: () => invoke("queue_history_clear"),
  queueRetry: (id: number) => invoke<number>("queue_retry", { id }),
  onQueueHistory: (cb: (items: HistoryItem[]) => void) =>
    listen<HistoryItem[]>("queue-history", (e) => cb(e.payload)),
  onQueue: (cb: (items: QueueItem[]) => void) => listen<QueueItem[]>("queue", (e) => cb(e.payload)),
  onQueueEnded: (cb: (ended: QueueEnded) => void) => listen<QueueEnded>("queue-ended", (e) => cb(e.payload)),
  paddleStatus: () => invoke<PaddleStatus>("paddle_status"),
  paddleDownload: (size: PaddleSize) => invoke("paddle_download", { size }),
  paddleCancel: () => invoke("paddle_cancel"),
  paddleDelete: (size: PaddleSize) => invoke("paddle_delete", { size }),
  onPaddleProgress: (cb: (p: PaddleProgress) => void) => listen<PaddleProgress>("paddle-progress", (e) => cb(e.payload)),
  onPaddleDone: (cb: (d: PaddleDone) => void) => listen<PaddleDone>("paddle-done", (e) => cb(e.payload)),
  whisperStatus: () => invoke<WhisperStatus>("whisper_status"),
  whisperDownload: (model: WhisperModel, gpu = false) => invoke("whisper_download", { model, gpu }),
  whisperCancel: () => invoke("whisper_cancel"),
  whisperDelete: (model: WhisperModel, gpu = false) => invoke("whisper_delete", { model, gpu }),
  accelerationStatus: () => invoke<AccelerationStatus>("acceleration_status"),
  accelerationDownload: (runtime: GpuRuntime) => invoke("acceleration_download", { runtime }),
  accelerationCancel: () => invoke("acceleration_cancel"),
  accelerationDelete: (runtime: GpuRuntime) => invoke("acceleration_delete", { runtime }),
  accelerationRestart: () => invoke("acceleration_restart"),
  onAccelerationProgress: (cb: (p: AccelerationProgress) => void) =>
    listen<AccelerationProgress>("acceleration-progress", (e) => cb(e.payload)),
  onAccelerationDone: (cb: (d: AccelerationDone) => void) =>
    listen<AccelerationDone>("acceleration-done", (e) => cb(e.payload)),
  onWhisperProgress: (cb: (p: WhisperProgress) => void) =>
    listen<WhisperProgress>("whisper-progress", (e) => cb(e.payload)),
  onWhisperDone: (cb: (d: WhisperDone) => void) => listen<WhisperDone>("whisper-done", (e) => cb(e.payload)),
  translateStatus: () => invoke<TranslateStatus>("translate_status"),
  translateDownload: () => invoke("translate_download"),
  redactValidate: (pattern: string) => invoke("redact_validate", { pattern }),
  translateCancel: () => invoke("translate_cancel"),
  translateDelete: () => invoke("translate_delete"),
  onTranslateProgress: (cb: (p: TranslateProgress) => void) =>
    listen<TranslateProgress>("translate-progress", (e) => cb(e.payload)),
  onTranslateDone: (cb: (d: TranslateDone) => void) => listen<TranslateDone>("translate-done", (e) => cb(e.payload)),
  transcriptRead: (path: string, track: Track = "original") => invoke<Segment[] | null>("transcript_read", { path, track }),
  transcriptSave: (path: string, track: Track, segments: Segment[]) => invoke("transcript_save", { path, track, segments }),
  transcriptDetect: (path: string) => invoke<string | null>("transcript_detect", { path }),
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
