import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { openUrl } from "@tauri-apps/plugin-opener";

export type Hotkeys = { modifiers: string; fullscreen: string; window: string; area: string; text: string; record: string };
export type Config = {
  output_dir: string;
  copy_to_clipboard: boolean;
  preview_seconds: number;
  record_fps: number;
  text_capture: boolean;
  padding: { enabled: boolean; size: number; image: string; round: boolean; radius: number };
  language: "vi" | "en";
  hotkeys: Hotkeys;
};
export type Row = { combo: string; error: "invalid" | "needs_modifier" | "duplicate" | null; in_use: boolean };
export type Loaded = { config: Config; autostart: boolean; dir: string; first_run: boolean; ocr_available: boolean };
export type GalleryItem = { path: string; date_ms: number; kind: "photo" | "video" | "other" };

export const api = {
  load: () => invoke<Loaded>("load"),
  defaults: () => invoke<Config>("defaults"),
  resolveDir: (outputDir: string) => invoke<string>("resolve_dir", { outputDir }),
  check: (hotkeys: Hotkeys, text: boolean) => invoke<Row[]>("check", { hotkeys, text }),
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
  galleryDelete: (path: string) => invoke("gallery_delete", { path }),
  thumbSrc: (path: string) => convertFileSrc(path, "thumb"),
  mediaSrc: (path: string) => convertFileSrc(path, "media"),
  onSettingsOpen: (cb: (section: string) => void) => listen<string>("settings-open", (e) => cb(e.payload)),
  onGalleryChanged: (cb: () => void) => listen("gallery-changed", cb),
  hide: () => getCurrentWindow().hide(),
};
