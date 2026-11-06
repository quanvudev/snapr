import type { CompressHeight, CompressInfo, Quality, SubtitlePosition, SubtitleSize, SubtitleStyle, TaskSpec, Track } from "./api";
import type { T } from "./i18n";

export type Strings = (typeof T)["en"];

export const TASK_KINDS: Record<string, { label: keyof Strings }> = {
  gif: { label: "taskGif" },
  trim: { label: "taskTrim" },
  compress: { label: "taskCompress" },
  transcribe: { label: "taskTranscribe" },
  translate: { label: "taskTranslate" },
  subtitle: { label: "taskSubtitle" },
};

const TASK_ERRORS: Record<string, keyof Strings> = {
  NO_AUDIO: "noAudio",
  MODEL_MISSING: "modelMissing",
  TRANSCRIBING: "transcribing",
  NO_TRANSCRIPT: "noTranscript",
  NO_FONT: "noFont",
  ALREADY_ENGLISH: "alreadyEnglish",
  SAME_LANGUAGE: "sameLanguage",
  UNSUPPORTED_LANGUAGE: "unsupportedLanguage",
  LICENSE_REQUIRED: "licenseRequired",
};

const MODEL_ERRORS = ["MODEL_MISSING", "LICENSE_REQUIRED"];

export function needsModels(error: unknown) {
  const message = String(error);
  return MODEL_ERRORS.some((code) => message.includes(code));
}

export function taskError(t: Strings, error: unknown) {
  const message = String(error);
  const key = Object.keys(TASK_ERRORS).find((code) => message.includes(code));
  return key ? t[TASK_ERRORS[key]] : message;
}

export function transcribeSpec(path: string): TaskSpec {
  return { kind: "transcribe", path };
}

export function translateSpec(path: string, target: string | null = null): TaskSpec {
  return { kind: "translate", path, target };
}

export function trackSpec(path: string, track: Track): TaskSpec {
  return track === "original" ? transcribeSpec(path) : translateSpec(path, track);
}

export function trackKind(track: Track) {
  return track === "original" ? "transcribe" : "translate";
}

export function trackDetail(track: Track) {
  return track === "original" ? null : track;
}

export type CompressChoice = { quality: Quality; height: CompressHeight };

export const QUALITIES: { value: Quality; label: keyof Strings }[] = [
  { value: "low", label: "qualityLow" },
  { value: "medium", label: "qualityMedium" },
  { value: "high", label: "qualityHigh" },
];
export const COMPRESS_HEIGHTS: CompressHeight[] = [1080, 720, null];
export const COMPRESS_DEFAULT: CompressChoice = { quality: "medium", height: null };

const COMPRESS_KEY = "gallery.compress";

export function loadCompressChoice(): CompressChoice {
  try {
    const value = JSON.parse(localStorage.getItem(COMPRESS_KEY) ?? "null");
    if (value && QUALITIES.some((q) => q.value === value.quality) && COMPRESS_HEIGHTS.includes(value.height)) {
      return { quality: value.quality, height: value.height };
    }
  } catch {}
  return COMPRESS_DEFAULT;
}

export function saveCompressChoice(choice: CompressChoice) {
  try {
    localStorage.setItem(COMPRESS_KEY, JSON.stringify(choice));
  } catch {}
}

export function compressSpec(path: string, choice: CompressChoice): TaskSpec {
  return { kind: "compress", path, quality: choice.quality, height: choice.height };
}

export function compressHeightAvailable(info: CompressInfo, height: CompressHeight) {
  return info.estimates.some((e) => e.height === height);
}

export function compressEstimate(info: CompressInfo, choice: CompressChoice) {
  return info.estimates.find((e) => e.quality === choice.quality && e.height === choice.height)?.bytes ?? null;
}

export type GifPreset = { width: number | null; fps: number };

export type GifRange = { preset: GifPreset; startMs: number; endMs: number | null };

export const GIF_WIDTHS: (number | null)[] = [480, 640, 800, null];
export const GIF_FPS = [10, 15, 20];
export const GIF_DEFAULT: GifPreset = { width: 640, fps: 15 };
export const GIF_SLIDER_STEP_MS = 100;
export const BYTES_PER_PIXEL_ESTIMATE = 0.6;
export const MS_PER_SECOND = 1000;
export const PERCENT = 100;

const GIF_KEY = "gallery.gif";

export function loadGifPreset(): GifPreset {
  try {
    const value = JSON.parse(localStorage.getItem(GIF_KEY) ?? "null");
    if (value && GIF_WIDTHS.includes(value.width) && GIF_FPS.includes(value.fps)) {
      return { width: value.width, fps: value.fps };
    }
  } catch {}
  return GIF_DEFAULT;
}

export function saveGifPreset(preset: GifPreset) {
  try {
    localStorage.setItem(GIF_KEY, JSON.stringify(preset));
  } catch {}
}

export function gifEstimate(video: { w: number; h: number }, rangeMs: number, preset: GifPreset) {
  const width = Math.min(preset.width ?? video.w, video.w);
  const height = Math.round((width * video.h) / video.w);
  return width * height * preset.fps * (rangeMs / MS_PER_SECOND) * BYTES_PER_PIXEL_ESTIMATE;
}

export function gifSpec(path: string, preset: GifPreset, startMs: number, endMs: number | null): TaskSpec {
  return { kind: "gif", path, start_ms: startMs, end_ms: endMs, width: preset.width, fps: preset.fps };
}

export function trimSpec(path: string, startMs: number, endMs: number): TaskSpec {
  return { kind: "trim", path, start_ms: startMs, end_ms: endMs };
}

export type SubtitlePreset = { position: SubtitlePosition; size: SubtitleSize; style: SubtitleStyle };

export const SUBTITLE_POSITIONS: { value: SubtitlePosition; label: keyof Strings }[] = [
  { value: "bottom", label: "subtitleBottom" },
  { value: "top", label: "subtitleTop" },
];
export const SUBTITLE_SIZES: { value: SubtitleSize; label: keyof Strings }[] = [
  { value: "small", label: "subtitleSmall" },
  { value: "medium", label: "subtitleMedium" },
  { value: "large", label: "subtitleLarge" },
];
export const SUBTITLE_TEXT_COLORS: { value: string; label: keyof Strings }[] = [
  { value: "#ffffff", label: "colorWhite" },
  { value: "#ffd60a", label: "colorYellow" },
  { value: "#4cc9f0", label: "colorCyan" },
  { value: "#80ed99", label: "colorGreen" },
  { value: "#000000", label: "colorBlack" },
];
export const SUBTITLE_BACKGROUNDS: { value: string; label: keyof Strings }[] = [
  { value: "", label: "subtitleNoBackground" },
  { value: "#000000", label: "colorBlack" },
  { value: "#ffffff", label: "colorWhite" },
  { value: "#1e3a8a", label: "colorNavy" },
];
export const SUBTITLE_OPACITIES = [40, 60, 80, 100];
export const OUTLINE_RATIO = 0.06;
export const OUTLINE_STEPS = 16;
const LIGHT_LUMA = 128;
export const SUBTITLE_DEFAULT: SubtitlePreset = {
  position: "bottom",
  size: "medium",
  style: { text_color: "#ffffff", background: "#000000", opacity: 60 },
};

export function outlineColor(hex: string) {
  const [r, g, b] = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16));
  return (299 * r + 587 * g + 114 * b) / 1000 >= LIGHT_LUMA ? "#000000" : "#ffffff";
}
export const SUBTITLE_SIZE_RATIO: Record<SubtitleSize, number> = { small: 0.04, medium: 0.055, large: 0.075 };
export const MIN_TEXT_PX = 16;
export const EDGE_MARGIN_RATIO = 0.06;
export const MAX_WIDTH_RATIO = 0.9;
export const BOX_PADDING_X_RATIO = 0.5;
export const BOX_PADDING_Y_RATIO = 0.25;
export const LINE_HEIGHT_RATIO = 1.25;
export const BOX_RADIUS_RATIO = 0.25;

const SUBTITLE_KEY = "gallery.subtitle";

export function loadSubtitlePreset(): SubtitlePreset {
  try {
    const value = JSON.parse(localStorage.getItem(SUBTITLE_KEY) ?? "null");
    if (
      value &&
      SUBTITLE_POSITIONS.some((p) => p.value === value.position) &&
      SUBTITLE_SIZES.some((s) => s.value === value.size)
    ) {
      const style = value.style;
      const valid =
        style &&
        SUBTITLE_TEXT_COLORS.some((c) => c.value === style.text_color) &&
        SUBTITLE_BACKGROUNDS.some((c) => c.value === style.background) &&
        SUBTITLE_OPACITIES.includes(style.opacity);
      return { position: value.position, size: value.size, style: valid ? style : SUBTITLE_DEFAULT.style };
    }
  } catch {}
  return SUBTITLE_DEFAULT;
}

export function saveSubtitlePreset(preset: SubtitlePreset) {
  try {
    localStorage.setItem(SUBTITLE_KEY, JSON.stringify(preset));
  } catch {}
}

export function subtitleSpec(path: string, preset: SubtitlePreset, track: Track = "original"): TaskSpec {
  return { kind: "subtitle", path, position: preset.position, size: preset.size, style: preset.style, track };
}

export function percent(done: number, total: number) {
  return total > 0 ? Math.min(PERCENT, (done / total) * PERCENT) : 0;
}
